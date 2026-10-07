//! The application catalog.
//!
//! Like the original KAppFinder, this tool is catalog-driven rather than
//! heuristic: it knows about a fixed set of programs and what a good menu
//! entry for each one looks like. A blind sweep of `$PATH` produces hundreds
//! of useless entries for things like `ls` and `awk`, so that mode exists
//! only as an explicit, opt-in report (`kappfinder-rs orphans`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// The compiled-in catalog, parsed at startup.
const BUILTIN_CATALOG: &str = include_str!("../data/catalog.toml");

/// Where a template came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Source {
    /// Shipped inside the binary.
    #[default]
    Builtin,
    /// Loaded from the user's catalog file.
    User,
    /// Synthesised on the fly for a binary with no template.
    Auto,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Builtin => "builtin",
            Source::User => "user",
            Source::Auto => "auto",
        }
    }
}

/// A recipe for one application's `.desktop` entry.
#[derive(Debug, Clone, Deserialize)]
pub struct AppTemplate {
    /// Executable name to look for in `$PATH`. Also the default filename stem.
    pub bin: String,
    /// `Name=` — the user-visible application name.
    pub name: String,
    /// `GenericName=` — what kind of program this is.
    #[serde(default)]
    pub generic_name: Option<String>,
    /// `Comment=` — the tooltip.
    #[serde(default)]
    pub comment: Option<String>,
    /// `Icon=` — an icon-theme name, not a path.
    #[serde(default)]
    pub icon: Option<String>,
    /// `Categories=` — first element should be a freedesktop main category.
    #[serde(default)]
    pub categories: Vec<String>,
    /// `Terminal=` — true for TUI/console programs.
    #[serde(default)]
    pub terminal: bool,
    /// `Keywords=` — extra search terms.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Appended verbatim after the binary path in `Exec=` (e.g. `%F`).
    #[serde(default)]
    pub exec_args: Option<String>,
    /// `MimeType=`.
    #[serde(default)]
    pub mime_types: Vec<String>,
    /// `StartupNotify=`. Defaults to `!terminal`.
    #[serde(default)]
    pub startup_notify: Option<bool>,
    /// Not read from TOML; set during loading.
    #[serde(skip)]
    pub source: Source,
}

impl AppTemplate {
    /// Build a reasonable default template for a binary the catalog does not
    /// know about. Used by `kappfinder-rs create <bin>`.
    pub fn auto(bin: &str) -> Self {
        AppTemplate {
            bin: bin.to_string(),
            name: prettify(bin),
            generic_name: None,
            comment: Some(format!("Run {bin}")),
            icon: Some("application-x-executable".to_string()),
            categories: vec!["Utility".to_string()],
            terminal: true,
            keywords: Vec::new(),
            exec_args: None,
            mime_types: Vec::new(),
            startup_notify: Some(false),
            source: Source::Auto,
        }
    }
}

/// Turn `foo-bar_baz` into `Foo Bar Baz`.
fn prettify(bin: &str) -> String {
    bin.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Deserialize)]
struct CatalogFile {
    #[serde(default)]
    app: Vec<AppTemplate>,
}

/// All known templates, keyed by binary name.
#[derive(Debug, Default)]
pub struct Catalog {
    pub apps: BTreeMap<String, AppTemplate>,
    /// The user catalog that was loaded, if any.
    pub user_file: Option<PathBuf>,
}

impl Catalog {
    /// Load the built-in catalog, then overlay the user catalog if it exists.
    /// A user entry with the same `bin` replaces the built-in one entirely.
    pub fn load(user_path: Option<&Path>) -> Result<Self> {
        let mut apps = BTreeMap::new();

        let builtin: CatalogFile = toml::from_str(BUILTIN_CATALOG)
            .context("the compiled-in catalog is malformed (this is a bug)")?;
        for mut app in builtin.app {
            app.source = Source::Builtin;
            apps.insert(app.bin.clone(), app);
        }

        let mut user_file = None;
        if let Some(path) = user_path {
            if path.is_file() {
                let text = fs::read_to_string(path)
                    .with_context(|| format!("reading user catalog {}", path.display()))?;
                let user: CatalogFile = toml::from_str(&text)
                    .with_context(|| format!("parsing user catalog {}", path.display()))?;
                for mut app in user.app {
                    app.source = Source::User;
                    apps.insert(app.bin.clone(), app);
                }
                user_file = Some(path.to_path_buf());
            }
        }

        Ok(Catalog { apps, user_file })
    }

    pub fn get(&self, bin: &str) -> Option<&AppTemplate> {
        self.apps.get(bin)
    }

    pub fn len(&self) -> usize {
        self.apps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_catalog_parses() {
        let catalog = Catalog::load(None).expect("built-in catalog must parse");
        assert!(
            catalog.len() > 50,
            "expected a substantial catalog, got {}",
            catalog.len()
        );
    }

    #[test]
    fn builtin_entries_are_well_formed() {
        let catalog = Catalog::load(None).unwrap();
        for (bin, app) in &catalog.apps {
            assert!(!bin.is_empty(), "empty bin key");
            assert_eq!(bin, &app.bin, "map key must match app.bin");
            assert!(!app.name.is_empty(), "{bin}: Name must not be empty");
            assert!(
                !app.categories.is_empty(),
                "{bin}: at least one category required"
            );
            assert!(
                !bin.contains('/'),
                "{bin}: bin must be a bare name, not a path"
            );
        }
    }

    /// freedesktop.org menu-spec main categories, excluding `Audio` and
    /// `Video`. Those two are nominally main categories, but the registry
    /// requires them to appear *alongside* `AudioVideo`, so for the purpose
    /// of "how many sections will this land in" they behave as
    /// subcategories of it.
    const MAIN_CATEGORIES: &[&str] = &[
        "AudioVideo",
        "Development",
        "Education",
        "Game",
        "Graphics",
        "Network",
        "Office",
        "Science",
        "Settings",
        "System",
        "Utility",
    ];

    #[test]
    fn first_category_is_a_main_category() {
        let catalog = Catalog::load(None).unwrap();
        for (bin, app) in &catalog.apps {
            let first = &app.categories[0];
            assert!(
                MAIN_CATEGORIES.contains(&first.as_str()),
                "{bin}: '{first}' is not a freedesktop main category"
            );
        }
    }

    #[test]
    fn exactly_one_main_category_per_entry() {
        // Two main categories make the launcher appear in several menu
        // sections at once; desktop-file-validate warns about it.
        let catalog = Catalog::load(None).unwrap();
        for (bin, app) in &catalog.apps {
            let mains: Vec<&String> = app
                .categories
                .iter()
                .filter(|c| MAIN_CATEGORIES.contains(&c.as_str()))
                .collect();
            assert_eq!(
                mains.len(),
                1,
                "{bin}: expected exactly one main category, found {mains:?}"
            );
        }
    }

    #[test]
    fn audio_and_video_are_paired_with_audiovideo() {
        // menu-spec: "All applications in [Audio] should also be in AudioVideo"
        // — likewise for Video.
        let catalog = Catalog::load(None).unwrap();
        for (bin, app) in &catalog.apps {
            let has_sub = app
                .categories
                .iter()
                .any(|c| c == "Audio" || c == "Video");
            if has_sub {
                assert!(
                    app.categories.iter().any(|c| c == "AudioVideo"),
                    "{bin}: Audio/Video must be accompanied by AudioVideo"
                );
            }
        }
    }

    #[test]
    fn categories_have_no_duplicates() {
        let catalog = Catalog::load(None).unwrap();
        for (bin, app) in &catalog.apps {
            let mut sorted = app.categories.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(
                sorted.len(),
                app.categories.len(),
                "{bin}: duplicate categories"
            );
        }
    }

    #[test]
    fn prettify_titlecases() {
        assert_eq!(prettify("htop"), "Htop");
        assert_eq!(prettify("lazy-git"), "Lazy Git");
        assert_eq!(prettify("my_tool"), "My Tool");
    }

    #[test]
    fn auto_template_is_usable() {
        let t = AppTemplate::auto("somebin");
        assert_eq!(t.bin, "somebin");
        assert_eq!(t.source, Source::Auto);
        assert!(!t.categories.is_empty());
    }
}
