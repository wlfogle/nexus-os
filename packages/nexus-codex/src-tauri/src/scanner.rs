use std::path::PathBuf;

use ignore::WalkBuilder;

use crate::types::{Config, DocType};

/// Map a file extension to a [`DocType`], if it is a documentation file we care about.
pub(crate) fn doc_type_for_extension(ext: &str) -> Option<DocType> {
    match ext.to_ascii_lowercase().as_str() {
        "md" | "markdown" => Some(DocType::Markdown),
        "txt" => Some(DocType::Text),
        "pdf" => Some(DocType::Pdf),
        "rst" => Some(DocType::Rst),
        "adoc" | "asciidoc" => Some(DocType::Adoc),
        _ => None,
    }
}

/// Returns true if any component of `path` matches one of the excluded segments.
fn is_excluded(path: &str, excluded: &[String]) -> bool {
    excluded.iter().any(|seg| {
        if seg.is_empty() {
            return false;
        }
        // Match the segment as a path component (bounded by separators or ends).
        path.split('/').any(|c| c == seg) || path.contains(&format!("/{}/", seg))
    })
}

/// Walk every root in `config.scan_roots`, honouring `.gitignore`, and collect all
/// documentation files.
///
/// Skips:
///  - any path containing an excluded segment from `config.excluded_paths`
///  - files larger than `config.max_file_size_kb`
///
/// Returns a vector of `(path, doc_type, size_bytes)`.
pub fn scan_local(config: &Config) -> Vec<(PathBuf, DocType, u64)> {
    let max_bytes = config.max_file_size_kb.saturating_mul(1024);
    let mut results: Vec<(PathBuf, DocType, u64)> = Vec::new();

    for root in &config.scan_roots {
        let root_path = PathBuf::from(root);
        if !root_path.exists() {
            continue;
        }

        let walker = WalkBuilder::new(&root_path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .parents(true)
            .follow_links(false)
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Only files.
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }

            let path = entry.path();
            let path_str = path.to_string_lossy();

            if is_excluded(&path_str, &config.excluded_paths) {
                continue;
            }

            let ext = match path.extension().and_then(|e| e.to_str()) {
                Some(e) => e,
                None => continue,
            };

            let doc_type = match doc_type_for_extension(ext) {
                Some(dt) => dt,
                None => continue,
            };

            let size_bytes = match entry.metadata() {
                Ok(m) => m.len(),
                Err(_) => continue,
            };

            if max_bytes > 0 && size_bytes > max_bytes {
                continue;
            }

            results.push((path.to_path_buf(), doc_type, size_bytes));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_type_extensions() {
        assert_eq!(doc_type_for_extension("md"), Some(DocType::Markdown));
        assert_eq!(doc_type_for_extension("MD"), Some(DocType::Markdown));
        assert_eq!(doc_type_for_extension("markdown"), Some(DocType::Markdown));
        assert_eq!(doc_type_for_extension("txt"), Some(DocType::Text));
        assert_eq!(doc_type_for_extension("pdf"), Some(DocType::Pdf));
        assert_eq!(doc_type_for_extension("rst"), Some(DocType::Rst));
        assert_eq!(doc_type_for_extension("adoc"), Some(DocType::Adoc));
        assert_eq!(doc_type_for_extension("asciidoc"), Some(DocType::Adoc));
        assert_eq!(doc_type_for_extension("rs"), None);
        assert_eq!(doc_type_for_extension("py"), None);
        assert_eq!(doc_type_for_extension(""), None);
    }

    #[test]
    fn excluded_path_matching() {
        let excluded = vec![
            "node_modules".to_string(),
            "target".to_string(),
            ".git".to_string(),
        ];
        assert!(is_excluded("/home/user/project/node_modules/lodash/README.md", &excluded));
        assert!(is_excluded("/home/user/project/target/debug/foo", &excluded));
        assert!(is_excluded("/home/user/project/.git/config", &excluded));
        assert!(!is_excluded("/home/user/project/src/README.md", &excluded));
        assert!(!is_excluded("/home/user/my-target-project/README.md", &excluded));
    }
}
