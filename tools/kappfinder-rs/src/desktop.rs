//! Reading and writing freedesktop.org Desktop Entry files.
//!
//! Only the `[Desktop Entry]` group is interpreted. Localised keys (`Name[de]`)
//! are deliberately ignored: this tool only needs the C-locale values.

use std::fs;
use std::path::Path;

use crate::catalog::AppTemplate;

/// Marker key written into every entry this tool generates, so that `remove`
/// can find its own output again without touching hand-written entries.
pub const GENERATED_KEY: &str = "X-KAppFinder-Generated";
/// Records which catalog the entry came from (builtin / user / auto).
pub const SOURCE_KEY: &str = "X-KAppFinder-Source";

/// Command wrappers that are not themselves the application being launched.
const WRAPPERS: &[&str] = &[
    "env",
    "sh",
    "bash",
    "dash",
    "zsh",
    "fish",
    "setsid",
    "nohup",
    "exec",
    "time",
    "nice",
    "ionice",
    "stdbuf",
    "dbus-run-session",
    "dbus-launch",
    "systemd-run",
    "systemd-cat",
    "gtk-launch",
    "pkexec",
    "sudo",
    "doas",
    "flatpak-spawn",
];

/// Shells whose `-c <command>` argument should be parsed recursively.
const SHELLS: &[&str] = &["sh", "bash", "dash", "zsh", "fish"];

/// A parsed `[Desktop Entry]` group. Only the fields this tool reasons about
/// are retained.
#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub entry_type: String,
    pub exec: Option<String>,
    pub try_exec: Option<String>,
    pub no_display: bool,
    pub hidden: bool,
    pub generated: bool,
}

impl DesktopEntry {
    /// Basename of the program this entry launches, if one can be determined.
    ///
    /// `TryExec` wins when present because it is unambiguous: it is a bare
    /// path with no arguments and no field codes.
    pub fn program(&self) -> Option<String> {
        if let Some(te) = &self.try_exec {
            let base = basename(te.trim());
            if !base.is_empty() {
                return Some(base);
            }
        }
        self.exec.as_deref().and_then(exec_program)
    }

    /// Whether this entry actually shows up in a menu.
    pub fn visible(&self) -> bool {
        !self.no_display && !self.hidden
    }
}

/// Parse a `.desktop` file. Returns `None` for unreadable or non-UTF-8 files
/// rather than failing the whole scan over one bad entry.
pub fn parse(path: &Path) -> Option<DesktopEntry> {
    let text = fs::read_to_string(path).ok()?;
    let mut entry = DesktopEntry {
        entry_type: String::new(),
        exec: None,
        try_exec: None,
        no_display: false,
        hidden: false,
        generated: false,
    };

    let mut in_group = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(stripped) = line.strip_prefix('[') {
            if let Some(group) = stripped.strip_suffix(']') {
                in_group = group == "Desktop Entry";
            }
            continue;
        }
        if !in_group {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        // Skip localised variants such as `Comment[fr]`.
        if key.contains('[') {
            continue;
        }
        let value = unescape_value(value.trim());
        match key {
            "Type" => entry.entry_type = value,
            "Exec" => entry.exec = Some(value),
            "TryExec" => entry.try_exec = Some(value),
            "NoDisplay" => entry.no_display = value.eq_ignore_ascii_case("true"),
            "Hidden" => entry.hidden = value.eq_ignore_ascii_case("true"),
            GENERATED_KEY => entry.generated = value.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }

    Some(entry)
}

/// Reverse the Desktop Entry value escaping rules (`\s \n \t \r \\`).
fn unescape_value(s: &str) -> String {
    if !s.contains('\\') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('s') => out.push(' '),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Apply Desktop Entry value escaping to a string destined for a file.
fn escape_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

/// Split an `Exec` value into argv tokens, honouring the spec's double-quote
/// and backslash rules.
pub fn tokenize_exec(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut started = false;
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            '\\' if quoted => {
                if let Some(next) = chars.next() {
                    cur.push(next);
                }
            }
            c if c.is_whitespace() && !quoted => {
                if started || !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                    started = false;
                }
            }
            c => cur.push(c),
        }
    }
    if started || !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

/// Extract the basename of the real program from an `Exec` value, seeing
/// through `env`, shell `-c`, and similar wrappers.
pub fn exec_program(exec: &str) -> Option<String> {
    let tokens = tokenize_exec(exec);
    program_from_tokens(&tokens, 0)
}

fn program_from_tokens(tokens: &[String], depth: usize) -> Option<String> {
    if depth > 4 {
        return None;
    }
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        // Field codes (%f, %U, ...) and empty tokens are never the program.
        if token.is_empty() || token.starts_with('%') {
            i += 1;
            continue;
        }
        // Leading `VAR=value` assignments and option flags are not the program.
        if is_assignment(token) || token.starts_with('-') {
            i += 1;
            continue;
        }

        let base = basename(token);
        if WRAPPERS.contains(&base.as_str()) {
            if SHELLS.contains(&base.as_str()) {
                // `sh -c "real-command ..."` — recurse into the script body.
                if let Some(offset) = tokens[i + 1..].iter().position(|a| a == "-c") {
                    let script_idx = i + 1 + offset + 1;
                    if let Some(script) = tokens.get(script_idx) {
                        let inner = tokenize_exec(script);
                        if let Some(found) = program_from_tokens(&inner, depth + 1) {
                            return Some(found);
                        }
                    }
                }
            }
            i += 1;
            continue;
        }

        return Some(base);
    }
    None
}

fn is_assignment(token: &str) -> bool {
    let Some((lhs, _)) = token.split_once('=') else {
        return false;
    };
    !lhs.is_empty()
        && !lhs.starts_with('-')
        && !lhs.contains('/')
        && lhs
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Final path component of a possibly-qualified program name.
pub fn basename(s: &str) -> String {
    s.rsplit('/').next().unwrap_or(s).to_string()
}

/// Quote a single argument for use inside an `Exec` value.
///
/// Literal `%` must be doubled, and reserved characters force double quoting
/// with backslash escapes. The result is then run through [`escape_value`]
/// when written, which doubles those backslashes again — exactly what the
/// spec requires.
fn exec_quote(arg: &str) -> String {
    let pct = arg.replace('%', "%%");
    let reserved = " \t\n\"'\\<>~|&;$*?#()`";
    let needs_quotes = pct.is_empty() || pct.chars().any(|c| reserved.contains(c));
    if !needs_quotes {
        return pct;
    }
    let mut out = String::with_capacity(pct.len() + 2);
    out.push('"');
    for c in pct.chars() {
        if matches!(c, '"' | '\\' | '$' | '`') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// Render a complete `.desktop` file for `template`, launching `bin_path`.
pub fn render(template: &AppTemplate, bin_path: &Path) -> String {
    let bin = bin_path.to_string_lossy();
    let mut out = String::with_capacity(512);

    out.push_str("# Generated by kappfinder-rs.\n");
    out.push_str("# Edit freely; `kappfinder-rs remove` only deletes files that still\n");
    out.push_str("# carry the X-KAppFinder-Generated marker below.\n");
    out.push_str("[Desktop Entry]\n");
    out.push_str("Type=Application\n");
    // The Version key names the *spec revision* the file claims to follow.
    // desktop-file-validate only recognises 1.0 and 1.1, so claim 1.0 for
    // maximum compatibility — nothing here needs a later revision.
    out.push_str("Version=1.0\n");

    push_kv(&mut out, "Name", &template.name);
    if let Some(v) = &template.generic_name {
        push_kv(&mut out, "GenericName", v);
    }
    if let Some(v) = &template.comment {
        push_kv(&mut out, "Comment", v);
    }

    let mut exec = exec_quote(&bin);
    if let Some(args) = &template.exec_args {
        let args = args.trim();
        if !args.is_empty() {
            exec.push(' ');
            exec.push_str(args);
        }
    }
    push_kv(&mut out, "Exec", &exec);
    // TryExec is a bare path: no quoting, no field codes.
    push_kv(&mut out, "TryExec", &bin);

    if let Some(icon) = &template.icon {
        push_kv(&mut out, "Icon", icon);
    }
    out.push_str(if template.terminal {
        "Terminal=true\n"
    } else {
        "Terminal=false\n"
    });

    if !template.categories.is_empty() {
        push_kv(&mut out, "Categories", &join_semi(&template.categories));
    }
    if !template.keywords.is_empty() {
        push_kv(&mut out, "Keywords", &join_semi(&template.keywords));
    }
    if !template.mime_types.is_empty() {
        push_kv(&mut out, "MimeType", &join_semi(&template.mime_types));
    }

    let startup_notify = template.startup_notify.unwrap_or(!template.terminal);
    out.push_str(if startup_notify {
        "StartupNotify=true\n"
    } else {
        "StartupNotify=false\n"
    });

    out.push_str(GENERATED_KEY);
    out.push_str("=true\n");
    push_kv(&mut out, SOURCE_KEY, template.source.as_str());

    out
}

fn push_kv(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push('=');
    out.push_str(&escape_value(value));
    out.push('\n');
}

/// Desktop Entry list values are semicolon separated *and* semicolon terminated.
fn join_semi(items: &[String]) -> String {
    let mut s = String::new();
    for item in items {
        s.push_str(item);
        s.push(';');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_quoted_exec() {
        let t = tokenize_exec(r#"/usr/bin/foo "a b" --flag"#);
        assert_eq!(t, vec!["/usr/bin/foo", "a b", "--flag"]);
    }

    #[test]
    fn finds_plain_program() {
        assert_eq!(exec_program("/usr/bin/htop").as_deref(), Some("htop"));
        assert_eq!(exec_program("htop %U").as_deref(), Some("htop"));
    }

    #[test]
    fn sees_through_env_wrapper() {
        assert_eq!(
            exec_program("env LANG=C /usr/bin/gkrellm").as_deref(),
            Some("gkrellm")
        );
    }

    #[test]
    fn sees_through_shell_dash_c() {
        assert_eq!(
            exec_program(r#"sh -c "exec /usr/bin/ncdu /home""#).as_deref(),
            Some("ncdu")
        );
    }

    #[test]
    fn ignores_leading_flags_and_field_codes() {
        assert_eq!(exec_program("%U").as_deref(), None);
        assert_eq!(exec_program("--help").as_deref(), None);
    }

    #[test]
    fn escapes_and_unescapes_round_trip() {
        let original = "line1\nline2\\end";
        assert_eq!(unescape_value(&escape_value(original)), original);
    }

    #[test]
    fn quotes_paths_with_spaces() {
        assert_eq!(exec_quote("/opt/my app/bin"), "\"/opt/my app/bin\"");
        assert_eq!(exec_quote("/usr/bin/htop"), "/usr/bin/htop");
        assert_eq!(exec_quote("100%"), "100%%");
    }

    #[test]
    fn list_values_are_semicolon_terminated() {
        let v = vec!["System".to_string(), "Monitor".to_string()];
        assert_eq!(join_semi(&v), "System;Monitor;");
    }

    #[test]
    fn detects_assignments() {
        assert!(is_assignment("LANG=C"));
        assert!(!is_assignment("/usr/bin/x=y"));
        assert!(!is_assignment("--opt=1"));
        assert!(!is_assignment("plainword"));
    }
}
