//! Resolves repository-local imports to supported source paths.

use std::path::Path;

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
        let base = root.join(base);
        let mut candidates = vec![base.clone()];
        for extension in crate::adapters::ECMA_EXTENSIONS {
            let mut file = base.clone().into_os_string();
            file.push(format!(".{extension}"));
            candidates.push(file.into());
            candidates.push(base.join(format!("index.{extension}")));
        }
        candidates
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
