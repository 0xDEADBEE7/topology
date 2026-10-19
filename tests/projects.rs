//! Verifies project discovery, local React imports, and failure reporting.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("topology-test-{}-{id}-{stamp}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn write(&self, path: &str, source: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    fn run(&self, command: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_topo"))
            .args([command, self.0.to_str().unwrap()])
            .output()
            .unwrap()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn project_commands_exclude_dependencies_and_resolve_tsx() {
    let project = Project::new();
    project.write("src/App.tsx", "/** Application entry. */\nimport { Card } from './ui';\nexport function App() { return <Card />; }");
    project.write(
        "src/ui/index.tsx",
        "/** A greeting card. */\nexport const Card = () => <div />;",
    );
    project.write("src/config.mts", "export const config = () => true;");
    for directory in ["node_modules", "dist", "coverage", ".next"] {
        project.write(&format!("{directory}/invalid.tsx"), "const broken = <;");
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&project.0, project.0.join("src/loop")).unwrap();
    let score = project.run("score");
    assert!(
        score.status.success(),
        "{}",
        String::from_utf8_lossy(&score.stderr)
    );
    assert!(score.stderr.is_empty());
    let extract = project.run("extract");
    assert!(extract.status.success());
    assert!(extract.stderr.is_empty());
    let records: Vec<serde_json::Value> = String::from_utf8(extract.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "file")
            .count(),
        3
    );
    assert!(records
        .iter()
        .any(|record| record["type"] == "import" && record["resolved_path"] == "src/ui/index.tsx"));
}

#[test]
fn scoring_rejects_empty_projects_and_measures_invalid_source() {
    let project = Project::new();
    assert!(!project.run("score").status.success());
    project.write("invalid.tsx", "const broken = <;");
    let output = project.run("score");
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid.tsx") && stderr.contains("syntax errors"));
}

#[test]
fn commands_skip_files_they_cannot_parse() {
    let project = Project::new();
    project.write("good.rs", "//! Good.\n/// Works.\npub fn works() {}\n");
    project.write("broken.rs", "fn broken( {");
    for command in ["score", "extract"] {
        let output = project.run(command);
        assert!(output.status.success(), "{command}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("broken.rs"));
    }
    let find = Command::new(env!("CARGO_BIN_EXE_topo"))
        .args(["find", "works"])
        .current_dir(&project.0)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&find.stdout).contains("good.rs::works"));
    assert!(String::from_utf8_lossy(&find.stderr).contains("broken.rs"));
}

#[test]
fn symbol_lookup_reads_only_the_named_file() {
    let project = Project::new();
    project.write(
        "lib.rs",
        "//! Library.\n/// Holds settings.\npub struct Config {\n    pub name: String,\n}\n/// Builds a config.\npub fn build() -> Config {\n    Config { name: String::new() }\n}\n",
    );
    project.write("broken.rs", "fn broken( {");
    let topo = |target: &str, directory: &Path| {
        let output = Command::new(env!("CARGO_BIN_EXE_topo"))
            .arg(target)
            .current_dir(directory)
            .output()
            .unwrap();
        assert!(
            output.status.success() && output.stderr.is_empty(),
            "{target}"
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let file = project.0.join("lib.rs").display().to_string();
    let body = "pub fn build() -> Config {\n    Config { name: String::new() }\n}";
    assert!(topo(&format!("{file}::build"), Path::new(".")).contains(body));
    assert!(topo("lib.rs::build", &project.0).contains(body));
    assert!(topo(&file, &project.0).contains("Config (L3–5)"));
}

#[test]
fn symbol_descriptions_include_preceding_rust_and_jsdoc_comments() {
    let project = Project::new();
    project.write(
        "sample.rs",
        "//! Rust fixture.\n/// Explains the Rust function.\npub fn sample() {}\n",
    );
    project.write("sample.ts", "/** Module description. */\n/**\n * Explains the component.\n */\nexport const sample = () => true;\n");
    let output = project.run("extract");
    assert!(output.status.success());
    let records: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(records
        .iter()
        .any(|record| record["description"] == "Explains the Rust function."));
    assert!(records.iter().any(|record| record["description"]
        .as_str()
        .is_some_and(|doc| doc.contains("Explains the component."))));
}
