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
    let candidates = match language {
        "rust" => {
            let module = source.strip_prefix("crate::")?.split("::").next()?;
            let src = root.join("src");
            vec![
                src.join(format!("{module}.rs")),
                src.join(module).join("mod.rs"),
            ]
        }
        "python" => {
            let module = source.rsplit_once('.').map_or(source, |(module, _)| module);
            let base = root.join(Path::new(&module.replace('.', "/")));
            vec![base.with_extension("py"), base.join("__init__.py")]
        }
        "typescript" | "javascript" if source.starts_with('.') => {
            let directory = Path::new(current).parent().unwrap_or(Path::new("."));
            script_candidates(&root.join(directory.join(source)))
        }
        _ => return None,
    };
    candidates.into_iter().find_map(|candidate| {
        let path = candidate
            .is_file()
            .then(|| candidate.canonicalize().ok())??;
        Some(path.strip_prefix(root).ok()?.to_string_lossy().to_string())
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
