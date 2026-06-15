/// NexusTerminal Fix Engine
///
/// Infrastructure-driven scan → question → fix → verify → commit workflow.
/// The model's only job is generating a corrected file for a specific error.
/// Every other decision (what to check, when to stop, how to verify) is Rust.
///
/// This mirrors how Oz works: the agentic loop runs in infrastructure, not in
/// the LLM.  Small local models that can't reliably chain tool-calls can still
/// generate correct code for a single bounded task.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::Emitter;
use tracing::{info, warn};

use crate::model_router::{select_model, TaskKind};

// ── Public event type ─────────────────────────────────────────────────────────

/// Emitted to the frontend on every step of the fix process.
#[derive(Debug, Clone, Serialize)]
pub struct FixProgressEvent {
    pub session_id: String,
    /// "scanning" | "question" | "fixing" | "verifying" | "done" | "failed"
    pub stage: String,
    pub message: String,
    pub done: bool,
    pub errors_found: usize,
    pub errors_fixed: usize,
}

// ── Compiler error ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CompilerError {
    /// Absolute file path.
    file: String,
    line: u32,
    /// Short error message.
    message: String,
    /// Full rendered text shown to the model for context.
    rendered: String,
}

// ── Cargo JSON parsing ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CargoMessage {
    reason: Option<String>,
    message: Option<CargoMessageBody>,
}
#[derive(Deserialize)]
struct CargoMessageBody {
    level: Option<String>,
    message: Option<String>,
    spans: Option<Vec<CargoSpan>>,
    rendered: Option<String>,
}
#[derive(Deserialize)]
struct CargoSpan {
    file_name: Option<String>,
    line_start: Option<u32>,
    is_primary: Option<bool>,
}

fn parse_cargo_errors(json_output: &str, project_root: &str) -> Vec<CompilerError> {
    let mut errors = Vec::new();
    for line in json_output.lines() {
        let msg: CargoMessage = match serde_json::from_str(line) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if msg.reason.as_deref() != Some("compiler-message") {
            continue;
        }
        let body = match msg.message {
            Some(b) => b,
            None => continue,
        };
        if body.level.as_deref() != Some("error") {
            continue;
        }
        let spans = body.spans.unwrap_or_default();
        let primary = spans.iter().find(|s| s.is_primary == Some(true)).or_else(|| spans.first());
        let (file, line) = match primary {
            Some(s) => {
                let fname = s.file_name.clone().unwrap_or_default();
                let abs = if fname.starts_with('/') {
                    fname
                } else {
                    format!("{}/{}", project_root.trim_end_matches('/'), fname)
                };
                (abs, s.line_start.unwrap_or(1))
            }
            None => continue,
        };
        errors.push(CompilerError {
            file,
            line,
            message: body.message.unwrap_or_default(),
            rendered: body.rendered.unwrap_or_default(),
        });
    }
    errors
}

// ── TSC parsing ───────────────────────────────────────────────────────────────

fn parse_tsc_errors(output: &str, project_root: &str) -> Vec<CompilerError> {
    // Format: path/file.ts(10,5): error TS2322: Type 'string' is not assignable…
    let mut errors = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.contains("): error TS") {
            continue;
        }
        // Split at first '('
        let paren_pos = match line.find('(') {
            Some(p) => p,
            None => continue,
        };
        let rel_file = &line[..paren_pos];
        let rest = &line[paren_pos + 1..];
        let close = match rest.find(')') {
            Some(c) => c,
            None => continue,
        };
        let coords = &rest[..close];
        let line_num: u32 = coords
            .split(',')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let msg_part = rest[close + 1..].trim_start_matches(": error ").trim_start_matches(": ");
        let abs_file = if rel_file.starts_with('/') {
            rel_file.to_string()
        } else {
            format!("{}/{}", project_root.trim_end_matches('/'), rel_file)
        };
        errors.push(CompilerError {
            file: abs_file,
            line: line_num,
            message: msg_part.to_string(),
            rendered: line.to_string(),
        });
    }
    errors
}

// ── Project discovery ─────────────────────────────────────────────────────────

async fn find_cargo_dirs(root: &str) -> Vec<String> {
    let cmd = format!(
        "find '{}' -name 'Cargo.toml' \
         -not -path '*/target/*' -not -path '*/.git/*' -not -path '*/node_modules/*' \
         2>/dev/null | head -8",
        root
    );
    match tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| {
                Path::new(l.trim())
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
            })
            .collect(),
        Err(_) => vec![],
    }
}

async fn find_package_dirs(root: &str) -> Vec<String> {
    let cmd = format!(
        "find '{}' -name 'package.json' \
         -not -path '*/node_modules/*' -not -path '*/.git/*' \
         2>/dev/null | head -5",
        root
    );
    match tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| {
                Path::new(l.trim())
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
            })
            .collect(),
        Err(_) => vec![],
    }
}

// ── Compiler runner ───────────────────────────────────────────────────────────

/// Run cargo check and return (success, json_output).
async fn run_cargo_check(dir: &str) -> (bool, String) {
    match tokio::time::timeout(
        Duration::from_secs(120),
        tokio::process::Command::new("cargo")
            .args(["check", "--message-format=json"])
            .current_dir(dir)
            .output(),
    )
    .await
    {
        Ok(Ok(out)) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            (out.status.success(), combined)
        }
        Ok(Err(e)) => (false, format!("cargo check failed to run: {}", e)),
        Err(_) => (false, "cargo check timed out (120s)".to_string()),
    }
}

/// Run tsc --noEmit and return (success, plain_output).
async fn run_tsc_check(dir: &str) -> (bool, String) {
    match tokio::time::timeout(
        Duration::from_secs(90),
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg("npx tsc --noEmit 2>&1")
            .current_dir(dir)
            .output(),
    )
    .await
    {
        Ok(Ok(out)) => {
            let combined = String::from_utf8_lossy(&out.stdout).to_string();
            (out.status.success(), combined)
        }
        Ok(Err(e)) => (false, format!("tsc failed: {}", e)),
        Err(_) => (false, "tsc timed out (90s)".to_string()),
    }
}

// ── AI-driven file fix ────────────────────────────────────────────────────────

fn extract_code_block(text: &str, lang: &str) -> String {
    let fence = format!("```{}", lang);
    if let Some(start) = text.find(&fence) {
        let after = &text[start + fence.len()..];
        let after = after.trim_start_matches(|c: char| c != '\n').trim_start_matches('\n');
        if let Some(end) = after.find("```") {
            return after[..end].to_string();
        }
    }
    // Try plain ```
    if let Some(start) = text.find("```\n") {
        let after = &text[start + 4..];
        if let Some(end) = after.find("```") {
            return after[..end].to_string();
        }
    }
    String::new()
}

/// Ask the code-fix model to return a corrected version of the file.
/// Returns the fixed content, or None if the model response wasn't usable.
async fn ai_fix_file(
    file_path: &str,
    content: &str,
    errors: &[CompilerError],
    model: &str,
    ollama_url: &str,
) -> Option<String> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("txt");

    let error_block = errors
        .iter()
        .map(|e| format!("  Line {}: {}", e.line, e.message))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "You are a {lang} expert. The following file has compiler errors. \
Return the COMPLETE corrected file in a ```{ext} code block. \
Do not truncate. No explanations outside the code block.\n\n\
Compiler errors in {file}:\n{errors}\n\n\
```{ext}\n{content}\n```",
        lang = match ext {
            "rs" => "Rust",
            "ts" | "tsx" => "TypeScript",
            "js" | "jsx" => "JavaScript",
            "py" => "Python",
            "go" => "Go",
            _ => ext,
        },
        file = file_path,
        errors = error_block,
        content = if content.len() > 40_000 {
            &content[..40_000]
        } else {
            content
        },
    );

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": { "temperature": 0.05, "num_predict": 8192 }
    });

    let url = format!("{}/api/generate", ollama_url);
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .timeout(Duration::from_secs(180))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let data: serde_json::Value = resp.json().await.ok()?;
    let text = data["response"].as_str().unwrap_or("");
    if text.is_empty() {
        return None;
    }

    let fixed = extract_code_block(text, ext);
    if fixed.trim().is_empty() || fixed.trim() == content.trim() {
        None
    } else {
        Some(fixed)
    }
}

// ── Main public entry point ───────────────────────────────────────────────────

/// Emit a progress event to the frontend.
fn emit_progress<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    session_id: &str,
    stage: &str,
    message: &str,
    done: bool,
    found: usize,
    fixed: usize,
) {
    let _ = app.emit(
        "fix-progress",
        FixProgressEvent {
            session_id: session_id.to_string(),
            stage: stage.to_string(),
            message: message.to_string(),
            done,
            errors_found: found,
            errors_fixed: fixed,
        },
    );
}

/// Run the full scan → fix → verify loop for a given path.
///
/// Emits `fix-progress` events throughout; sends a final `agent-done` event
/// so the frontend AI panel shows a completion message.
pub async fn scan_and_fix<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    session_id: String,
    scan_path: String,
    ollama_url: String,
) {
    info!("fix_engine: starting scan_and_fix for {}", scan_path);

    // ── 1. Discover projects ──────────────────────────────────────────────────
    emit_progress(&app, &session_id, "scanning", &format!("Discovering projects in {}…", scan_path), false, 0, 0);

    let cargo_dirs = find_cargo_dirs(&scan_path).await;
    let package_dirs = find_package_dirs(&scan_path).await;

    if cargo_dirs.is_empty() && package_dirs.is_empty() {
        emit_progress(&app, &session_id, "done",
            "No Rust or TypeScript projects found.", true, 0, 0);
        let _ = app.emit("agent-done", crate::agent::AgentDoneEvent {
            session_id: session_id.clone(),
            answer: "No Rust or TypeScript projects found in that path.".to_string(),
        });
        return;
    }

    // ── 2. Scan all projects ──────────────────────────────────────────────────
    let code_model = select_model(TaskKind::CodeFix, &ollama_url).await;
    info!("fix_engine: using code model {}", code_model);

    let mut all_errors: Vec<(String /* project_type */, String /* project_dir */, Vec<CompilerError>)> = Vec::new();

    for dir in &cargo_dirs {
        let short = Path::new(dir)
            .strip_prefix(&scan_path)
            .unwrap_or(Path::new(dir))
            .display()
            .to_string();
        let label = if short.is_empty() { dir.clone() } else { short };
        emit_progress(&app, &session_id, "scanning", &format!("cargo check: {}…", label), false, 0, 0);
        let (ok, output) = run_cargo_check(dir).await;
        if !ok {
            let errors = parse_cargo_errors(&output, dir);
            info!("fix_engine: {} cargo errors in {}", errors.len(), dir);
            if !errors.is_empty() {
                all_errors.push(("rust".to_string(), dir.clone(), errors));
            } else {
                // cargo failed but produced no parseable errors (build script, etc.)
                all_errors.push(("rust".to_string(), dir.clone(), vec![CompilerError {
                    file: dir.clone(),
                    line: 1,
                    message: "Build failed (check output for details)".to_string(),
                    rendered: output.chars().take(2000).collect(),
                }]));
            }
        }
    }

    for dir in &package_dirs {
        let short = Path::new(dir)
            .strip_prefix(&scan_path)
            .unwrap_or(Path::new(dir))
            .display()
            .to_string();
        let label = if short.is_empty() { dir.clone() } else { short };
        emit_progress(&app, &session_id, "scanning", &format!("tsc: {}…", label), false, 0, 0);
        let (ok, output) = run_tsc_check(dir).await;
        if !ok {
            let errors = parse_tsc_errors(&output, dir);
            info!("fix_engine: {} tsc errors in {}", errors.len(), dir);
            if !errors.is_empty() {
                all_errors.push(("ts".to_string(), dir.clone(), errors));
            }
        }
    }

    // Flatten errors for counting
    let total_errors: usize = all_errors.iter().map(|(_, _, e)| e.len()).sum();

    if total_errors == 0 {
        emit_progress(&app, &session_id, "done",
            &format!("All {} Rust + {} TS projects pass — no errors found!",
                cargo_dirs.len(), package_dirs.len()),
            true, 0, 0);
        let _ = app.emit("agent-done", crate::agent::AgentDoneEvent {
            session_id: session_id.clone(),
            answer: format!("✅ Scanned {} Rust + {} TS projects — all pass.",
                cargo_dirs.len(), package_dirs.len()),
        });
        return;
    }

    emit_progress(&app, &session_id, "fixing",
        &format!("Found {} error(s) across {} project(s). Starting fixes with {}…",
            total_errors, all_errors.len(), code_model),
        false, total_errors, 0);

    // ── 3. Fix loop ───────────────────────────────────────────────────────────
    let mut total_fixed = 0usize;
    let mut fix_log: Vec<String> = Vec::new();
    const MAX_ITERATIONS: usize = 5;

    // Group errors by file so we only fix each file once per iteration
    for (proj_type, proj_dir, errors) in &all_errors {
        // Collect unique files in this project
        let mut files_with_errors: std::collections::HashMap<String, Vec<CompilerError>> =
            std::collections::HashMap::new();
        for err in errors {
            files_with_errors
                .entry(err.file.clone())
                .or_default()
                .push(err.clone());
        }

        for iteration in 0..MAX_ITERATIONS {
            // Re-check to get fresh errors
            let (check_ok, check_output) = if proj_type == "rust" {
                run_cargo_check(proj_dir).await
            } else {
                run_tsc_check(proj_dir).await
            };

            if check_ok {
                info!("fix_engine: {} passed after {} iteration(s)", proj_dir, iteration);
                fix_log.push(format!("✅ {} — all errors fixed", proj_dir));
                break;
            }

            // Parse current errors
            let current_errors = if proj_type == "rust" {
                parse_cargo_errors(&check_output, proj_dir)
            } else {
                parse_tsc_errors(&check_output, proj_dir)
            };

            if current_errors.is_empty() {
                // Build failed but no parseable errors — skip
                fix_log.push(format!("⚠️  {} — build failed, could not parse errors", proj_dir));
                break;
            }

            // Group by file
            let mut by_file: std::collections::HashMap<String, Vec<CompilerError>> =
                std::collections::HashMap::new();
            for err in current_errors {
                by_file.entry(err.file.clone()).or_default().push(err);
            }

            let mut any_fixed = false;
            for (file_path, file_errors) in &by_file {
                if !PathBuf::from(file_path).exists() {
                    continue; // synthetic or generated path
                }
                let content = match tokio::fs::read_to_string(file_path).await {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("fix_engine: cannot read {}: {}", file_path, e);
                        continue;
                    }
                };

                let short_name = Path::new(file_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| file_path.clone());

                emit_progress(&app, &session_id, "fixing",
                    &format!("Fixing {} ({} error(s)) with {}…",
                        short_name, file_errors.len(), code_model),
                    false, total_errors, total_fixed);

                if let Some(fixed_content) = ai_fix_file(
                    file_path, &content, file_errors, &code_model, &ollama_url
                ).await {
                    // Write backup
                    let backup = format!("{}.bak", file_path);
                    if let Err(e) = tokio::fs::write(&backup, &content).await {
                        warn!("fix_engine: backup failed for {}: {}", file_path, e);
                        continue;
                    }
                    if let Err(e) = tokio::fs::write(file_path, &fixed_content).await {
                        warn!("fix_engine: write failed for {}: {}", file_path, e);
                        // Restore backup
                        let _ = tokio::fs::copy(&backup, file_path).await;
                        continue;
                    }
                    info!("fix_engine: wrote fix to {}", file_path);
                    fix_log.push(format!("🔧 Fixed: {}", file_path));
                    total_fixed += 1;
                    any_fixed = true;
                } else {
                    fix_log.push(format!("⚠️  Could not auto-fix: {} — manual review needed", file_path));
                }
            }

            if !any_fixed {
                fix_log.push(format!("⚠️  {} — no AI fixes possible after {} tries", proj_dir, iteration + 1));
                break;
            }
        }

        // Final check for this project
        emit_progress(&app, &session_id, "verifying",
            &format!("Verifying {}…", proj_dir), false, total_errors, total_fixed);
    }

    // ── 4. Final verification pass ────────────────────────────────────────────
    let mut pass_count = 0usize;
    let mut fail_count = 0usize;

    for dir in &cargo_dirs {
        let (ok, _) = run_cargo_check(dir).await;
        if ok { pass_count += 1; } else { fail_count += 1; }
    }
    for dir in &package_dirs {
        let (ok, _) = run_tsc_check(dir).await;
        if ok { pass_count += 1; } else { fail_count += 1; }
    }

    // ── 5. Git commit if all pass ─────────────────────────────────────────────
    if fail_count == 0 && total_fixed > 0 {
        let commit_msg = format!(
            "fix: auto-fixed {} file(s) via NexusTerminal fix engine\n\nCo-Authored-By: NexusAI <nexusai@nexusos.dev>",
            total_fixed
        );
        let _ = tokio::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&scan_path)
            .output()
            .await;
        let _ = tokio::process::Command::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(&scan_path)
            .output()
            .await;
        fix_log.push(format!("✅ Committed {} fix(es)", total_fixed));
    }

    // ── 6. Final report ───────────────────────────────────────────────────────
    let summary = if fail_count == 0 {
        format!(
            "✅ All {} project(s) pass. Fixed {} file(s).\n\n{}",
            pass_count + fail_count,
            total_fixed,
            fix_log.join("\n")
        )
    } else {
        format!(
            "⚠️  {}/{} projects pass. Fixed {} file(s). {} still failing — manual review needed.\n\n{}",
            pass_count,
            pass_count + fail_count,
            total_fixed,
            fail_count,
            fix_log.join("\n")
        )
    };

    emit_progress(&app, &session_id, "done", &summary, true, total_errors, total_fixed);
    let _ = app.emit("agent-done", crate::agent::AgentDoneEvent {
        session_id: session_id.clone(),
        answer: summary,
    });

    info!("fix_engine: done — fixed {}/{} errors", total_fixed, total_errors);
}
