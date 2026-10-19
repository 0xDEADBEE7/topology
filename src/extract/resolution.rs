//! Resolves relative imports to supported source files under the extraction root.

use crate::{
    lang::{Language, EXTENSIONS},
    metrics::Import,
};
use std::path::{Path, PathBuf};

/// Resolves a relative script specifier or Python module to a file under `root`.
pub(super) fn resolve_import(root: &Path, current: &str, import: &Import) -> Option<String> {
    let directory = root.join(current).parent()?.to_owned();
    let candidates = match Language::from_path(current)? {
        _ if !import.source.starts_with('.') => return None,
        Language::Python => python_candidates(directory, import),
        _ => script_candidates(&directory.join(&import.source)),
    };
    candidates.into_iter().find_map(|candidate| {
        let path = candidate
            .is_file()
            .then(|| candidate.canonicalize().ok())??;
        Some(path.strip_prefix(root).ok()?.to_string_lossy().into_owned())
    })
}

/// Lists files `from ..a.b import c` may name: each dot after the first climbs a package,
/// and `c` may be a submodule.
fn python_candidates(mut base: PathBuf, import: &Import) -> Vec<PathBuf> {
    let module = import.source.trim_start_matches('.');
    for _ in 1..import.source.len() - module.len() {
        base.pop();
    }
    base.extend(module.split('.').filter(|part| !part.is_empty()));
    let named = import.name.as_ref().map(|name| base.join(name));
    let modules = named.into_iter().chain([base]);
    modules
        .flat_map(|module| [module.with_extension("py"), module.join("__init__.py")])
        .collect()
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
