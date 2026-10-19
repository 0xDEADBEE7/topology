//! Resolves repository-local imports to supported source paths.

use crate::lang::{Language, EXTENSIONS};
use std::path::{Path, PathBuf};

/// Resolve an import against files local to the repository.
pub(super) fn resolve_import(
    root: &Path,
    current: &str,
    language: &str,
    source: &str,
) -> Option<String> {
    let candidates = if language == "rust" {
        let module = source.strip_prefix("crate::")?.split("::").next()?;
        vec![
            root.join("src").join(format!("{module}.rs")),
            root.join("src").join(module).join("mod.rs"),
        ]
    } else if language == "python" {
        let module = source
            .rsplit_once('.')
            .map_or(source, |(module, _)| module)
            .replace('.', "/");
        let base = root.join(Path::new(&module));
        vec![base.with_extension("py"), base.join("__init__.py")]
    } else if language == "typescript" || language == "javascript" {
        if !source.starts_with('.') {
            return None;
        }
        let base = Path::new(current)
            .parent()
            .unwrap_or(Path::new("."))
            .join(source);
        script_candidates(&root.join(base))
    } else {
        return None;
    };
    candidates.into_iter().find_map(|candidate| {
        candidate
            .is_file()
            .then(|| candidate.canonicalize().ok())
            .flatten()
            .and_then(|path| {
                path.strip_prefix(root)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
            })
    })
}

/// Lists files a relative script import may name: the path itself, the path with each
/// script extension or index file, and TypeScript sources imported by their `.js` names.
fn script_candidates(base: &Path) -> Vec<PathBuf> {
    let scripts = || {
        EXTENSIONS
            .iter()
            .filter(|(_, language)| !matches!(language, Language::Rust | Language::Python))
    };
    let mut candidates = vec![base.to_owned()];
    for (extension, _) in scripts() {
        let mut file = base.as_os_str().to_owned();
        file.push(format!(".{extension}"));
        candidates.push(file.into());
    }
    candidates.extend(scripts().map(|(extension, _)| base.join(format!("index.{extension}"))));
    if Language::from_path(base) == Some(Language::JavaScript) {
        let typescript = scripts().filter(|(_, language)| *language != Language::JavaScript);
        candidates.extend(typescript.map(|(extension, _)| base.with_extension(extension)));
    }
    candidates
}
