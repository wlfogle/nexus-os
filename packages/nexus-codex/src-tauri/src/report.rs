use std::collections::HashSet;
use std::path::Path;

use anyhow::{anyhow, Result};
use chrono::Utc;

use crate::types::{
    Config, DocResult, DocSource, DocStatus, DocType, Report, ReportSummary,
};

/// Build the full [`Report`] from a set of per-document results.
///
/// Computes the [`ReportSummary`] by counting statuses, distinct repositories per
/// source, and PDF documents.
pub fn build_report(
    scan_id: &str,
    model_used: &str,
    config: &Config,
    results: Vec<DocResult>,
    duration_secs: f64,
) -> Report {
    let mut summary = ReportSummary {
        total: results.len(),
        current: 0,
        stale: 0,
        outdated: 0,
        orphaned: 0,
        needs_review: 0,
        local_repos_scanned: 0,
        github_repos_scanned: 0,
        pdfs_scanned: 0,
        scan_duration_secs: duration_secs,
    };

    let mut local_repos: HashSet<String> = HashSet::new();
    let mut github_repos: HashSet<String> = HashSet::new();

    for r in &results {
        match r.status {
            DocStatus::Current => summary.current += 1,
            DocStatus::Stale => summary.stale += 1,
            DocStatus::Outdated => summary.outdated += 1,
            DocStatus::Orphaned => summary.orphaned += 1,
            DocStatus::NeedsReview => summary.needs_review += 1,
        }

        if r.doc_type == DocType::Pdf {
            summary.pdfs_scanned += 1;
        }

        if let Some(repo) = &r.repo {
            match r.source {
                DocSource::Local => {
                    local_repos.insert(repo.clone());
                }
                DocSource::Github => {
                    github_repos.insert(repo.clone());
                }
            }
        }
    }

    summary.local_repos_scanned = local_repos.len();
    summary.github_repos_scanned = github_repos.len();

    Report {
        scan_id: scan_id.to_string(),
        generated_at: Utc::now().to_rfc3339(),
        model_used: model_used.to_string(),
        config: config.clone(),
        results,
        summary,
    }
}

/// Human-readable label for a status.
fn status_label(status: &DocStatus) -> &'static str {
    match status {
        DocStatus::Current => "Current",
        DocStatus::Stale => "Stale",
        DocStatus::Outdated => "Outdated",
        DocStatus::Orphaned => "Orphaned",
        DocStatus::NeedsReview => "Needs Review",
    }
}

/// Resolve an output path to an absolute string after writing.
fn absolute_path(output_path: &str) -> String {
    match std::fs::canonicalize(output_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => Path::new(output_path).to_string_lossy().to_string(),
    }
}

/// Render and write a markdown report to `output_path`.
///
/// Documents are grouped by status in priority order:
/// Outdated → Stale → Orphaned → Needs Review → Current.
pub fn export_markdown(report: &Report, output_path: &str) -> Result<String> {
    let mut md = String::new();
    let s = &report.summary;

    md.push_str("# Nexus Codex — Documentation Intelligence Report\n\n");
    md.push_str(&format!("- **Scan ID:** {}\n", report.scan_id));
    md.push_str(&format!("- **Generated:** {}\n", report.generated_at));
    md.push_str(&format!("- **Model:** {}\n", report.model_used));
    md.push_str(&format!(
        "- **Duration:** {:.1}s\n\n",
        s.scan_duration_secs
    ));

    md.push_str("## Summary\n\n");
    md.push_str("| Metric | Count |\n");
    md.push_str("| --- | --- |\n");
    md.push_str(&format!("| Total documents | {} |\n", s.total));
    md.push_str(&format!("| Current | {} |\n", s.current));
    md.push_str(&format!("| Stale | {} |\n", s.stale));
    md.push_str(&format!("| Outdated | {} |\n", s.outdated));
    md.push_str(&format!("| Orphaned | {} |\n", s.orphaned));
    md.push_str(&format!("| Needs review | {} |\n", s.needs_review));
    md.push_str(&format!(
        "| Local repos scanned | {} |\n",
        s.local_repos_scanned
    ));
    md.push_str(&format!(
        "| GitHub repos scanned | {} |\n",
        s.github_repos_scanned
    ));
    md.push_str(&format!("| PDFs scanned | {} |\n\n", s.pdfs_scanned));

    let order = [
        DocStatus::Outdated,
        DocStatus::Stale,
        DocStatus::Orphaned,
        DocStatus::NeedsReview,
        DocStatus::Current,
    ];

    for status in &order {
        let docs: Vec<&DocResult> = report
            .results
            .iter()
            .filter(|r| &r.status == status)
            .collect();
        if docs.is_empty() {
            continue;
        }

        md.push_str(&format!(
            "## {} ({})\n\n",
            status_label(status),
            docs.len()
        ));

        for doc in docs {
            md.push_str(&format!("### `{}`\n\n", doc.path));
            if let Some(repo) = &doc.repo {
                md.push_str(&format!("- **Repo:** {}\n", repo));
            }
            if let Some(url) = &doc.repo_url {
                md.push_str(&format!("- **Repo URL:** {}\n", url));
            }
            md.push_str(&format!("- **Confidence:** {:.2}\n", doc.confidence));
            md.push_str(&format!(
                "- **Staleness:** {:.2}\n",
                doc.staleness_score
            ));
            if let Some(commit) = &doc.last_commit_date {
                md.push_str(&format!("- **Last commit:** {}\n", commit));
            }
            if let Some(age) = doc.related_code_age_days {
                md.push_str(&format!("- **Related code age:** {} days\n", age));
            }
            md.push_str(&format!("- **Reason:** {}\n", doc.reason));
            md.push_str(&format!("- **Evidence:** {}\n", doc.evidence));
            if let Some(rewrite) = &doc.suggested_rewrite {
                md.push_str("\n**Suggested rewrite:**\n\n");
                md.push_str("```\n");
                md.push_str(rewrite);
                md.push_str("\n```\n");
            }
            md.push('\n');
        }
    }

    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("failed to create output directory: {e}"))?;
        }
    }

    std::fs::write(output_path, md)
        .map_err(|e| anyhow!("failed to write markdown report: {e}"))?;

    Ok(absolute_path(output_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DocResult, DocSource, DocStatus, DocType};

    fn make_doc(status: DocStatus, source: DocSource, repo: Option<&str>, is_pdf: bool) -> DocResult {
        DocResult {
            path: "/tmp/test.md".to_string(),
            repo: repo.map(|s| s.to_string()),
            repo_url: None,
            source,
            doc_type: if is_pdf { DocType::Pdf } else { DocType::Markdown },
            status,
            confidence: 0.9,
            staleness_score: 0.5,
            reason: "test".to_string(),
            evidence: "test evidence".to_string(),
            suggested_rewrite: None,
            last_modified: None,
            last_commit_date: None,
            related_code_age_days: None,
            file_size_bytes: 1024,
        }
    }

    #[test]
    fn summary_counts_correctly() {
        let results = vec![
            make_doc(DocStatus::Current,    DocSource::Local,  Some("repo-a"), false),
            make_doc(DocStatus::Stale,      DocSource::Local,  Some("repo-a"), false),
            make_doc(DocStatus::Outdated,   DocSource::Github, Some("repo-b"), false),
            make_doc(DocStatus::Orphaned,   DocSource::Local,  None,           true),
            make_doc(DocStatus::NeedsReview,DocSource::Github, Some("repo-c"), true),
        ];
        let config = Config::default();
        let report = build_report("test-id", "llama3", &config, results, 42.0);
        let s = &report.summary;
        assert_eq!(s.total, 5);
        assert_eq!(s.current, 1);
        assert_eq!(s.stale, 1);
        assert_eq!(s.outdated, 1);
        assert_eq!(s.orphaned, 1);
        assert_eq!(s.needs_review, 1);
        assert_eq!(s.local_repos_scanned, 1); // only "repo-a"
        assert_eq!(s.github_repos_scanned, 2); // "repo-b" and "repo-c"
        assert_eq!(s.pdfs_scanned, 2);
        assert!((s.scan_duration_secs - 42.0).abs() < 0.001);
    }

    #[test]
    fn export_markdown_round_trip() {
        let config = Config::default();
        let results = vec![make_doc(DocStatus::Stale, DocSource::Local, Some("test-repo"), false)];
        let report = build_report("scan-001", "qwen2.5", &config, results, 1.5);
        let path = "/tmp/nexus_codex_test_report.md";
        let written = export_markdown(&report, path).expect("export_markdown failed");
        let content = std::fs::read_to_string(path).expect("could not read output");
        assert!(content.contains("Nexus Codex"));
        assert!(content.contains("scan-001"));
        assert!(content.contains("qwen2.5"));
        assert!(content.contains("Stale"));
        assert!(!written.is_empty());
    }

    #[test]
    fn export_json_is_valid() {
        let config = Config::default();
        let results = vec![make_doc(DocStatus::Current, DocSource::Github, Some("repo"), false)];
        let report = build_report("scan-002", "mistral", &config, results, 0.5);
        let path = "/tmp/nexus_codex_test_report.json";
        export_json(&report, path).expect("export_json failed");
        let content = std::fs::read_to_string(path).expect("could not read output");
        let parsed: serde_json::Value =
            serde_json::from_str(&content).expect("output is not valid JSON");
        assert_eq!(parsed["scan_id"], "scan-002");
        assert_eq!(parsed["model_used"], "mistral");
        assert_eq!(parsed["summary"]["total"], 1);
    }
}

/// Serialize the report to pretty JSON and write it to `output_path`.
pub fn export_json(report: &Report, output_path: &str) -> Result<String> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| anyhow!("failed to serialize report: {e}"))?;

    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("failed to create output directory: {e}"))?;
        }
    }

    std::fs::write(output_path, json)
        .map_err(|e| anyhow!("failed to write JSON report: {e}"))?;

    Ok(absolute_path(output_path))
}
