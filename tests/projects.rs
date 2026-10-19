//! Verifies project discovery, local React imports, and failure reporting.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

/// A topo command whose git stops at the temporary directory, wherever TMPDIR points.
fn topo_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_topo"));
    command.env("GIT_CEILING_DIRECTORIES", std::env::temp_dir());
    command
}

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
        topo_command()
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
    project.write("src/App.tsx", "/** Application entry. */\nimport { Card } from './ui';\nimport { config } from './config.mjs';\nimport { pick } from './App.selection';\nimport { Button } from './Button';\nexport function App() { return <Card />; }");
    project.write("src/App.selection.ts", "export const pick = () => 1;");
    project.write("src/Button.tsx", "export const Button = () => 1;");
    project.write("src/Button/index.ts", "export const Button = () => 2;");
    project.write(
        "src/ui/index.tsx",
        "/** A greeting card. */\nexport const Card = () => <div />;",
    );
    project.write("src/config.mts", "export const config = () => true;");
    project.write(
        "src/menu.jsx",
        "export function Menu() { return <a>{ok && <b />}</a>; }",
    );
    project.write("src/legacy.cjs", "export const legacy = () => 1;");
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
        8
    );
    for (name, language, cognitive) in [
        ("config", "typescript", 0),
        ("Menu", "javascript", 1),
        ("legacy", "javascript", 0),
    ] {
        assert!(records.iter().any(|record| record["name"] == name
            && record["language"] == language
            && record["metrics"]["cognitive"] == cognitive));
    }
    let resolved = |source: &str| {
        records
            .iter()
            .find(|record| record["type"] == "import" && record["source"] == source)
            .map(|record| record["resolved_path"].clone())
    };
    assert_eq!(resolved("./ui"), Some("src/ui/index.tsx".into()));
    assert_eq!(resolved("./config.mjs"), Some("src/config.mts".into()));
    assert_eq!(
        resolved("./App.selection"),
        Some("src/App.selection.ts".into())
    );
    assert_eq!(resolved("./Button"), Some("src/Button.tsx".into()));
}

#[test]
fn extract_records_signatures_grouped_imports_and_module_paths() {
    let project = Project::new();
    project.write(
        "src/build.rs",
        "//! Builds.\nuse crate::{metrics, visit::collect};\n\npub fn build(\n    input: &str,\n) -> Result<(), Error> {\n    Ok(())\n}\n",
    );
    project.write("src/widget.ts", "export function widget() {}\n");
    let records: Vec<serde_json::Value> = String::from_utf8(project.run("extract").stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let named = |name: &str| records.iter().find(|r| r["name"] == name).expect(name);
    let signature = "pub fn build( input: &str, ) -> Result<(), Error>";
    assert_eq!(named("build")["signature"], signature);
    assert_eq!(named("build")["exported"], true);
    assert_eq!(named("widget")["exported"], true);
    let imported = |source: &str| {
        records
            .iter()
            .any(|r| r["type"] == "import" && r["source"] == source)
    };
    assert!(imported("crate::metrics") && imported("crate::visit::collect"));
}

#[test]
fn extract_reads_imports_from_syntax_and_resolves_relative_ones() {
    let project = Project::new();
    let script = "export * from './a';\nexport {\n  b,\n} from './b';\nimport './side.css';\nimport {\n  c,\n} from './c';\nconst lazy = import(/* chunk */ './lazy');\nconst r = require('./r');\nimport q = require('./q');\nimport React from 'react';\n";
    project.write("src/index.ts", script);
    for file in ["a.ts", "b.ts", "c.tsx", "lazy.ts", "r.js", "q.ts"] {
        project.write(&format!("src/{file}"), "export const x = 1;\n");
    }
    let python = "import os.path as p, json\nfrom .. import m\nfrom ...a import (b, c as d)\nfrom . import *\n";
    project.write("pkg/sub/mod.py", python);
    for file in ["pkg/m.py", "a.py", "pkg/sub/__init__.py"] {
        project.write(file, "");
    }
    project.write(
        "lib.rs",
        "use a::{b, c as d, e::{self, /* c */ f}, g::*};\n",
    );
    project.write("pkg/b.py", "");
    project.write("pkg/b/__init__.py", "");
    project.write("pkg/a.py", "from .b import c\n");
    project.write("open.ts", "import x from \"./ab\n;\nexport * from \"\n;\n");
    project.write("ab.ts", "");
    let records: Vec<serde_json::Value> = String::from_utf8(project.run("extract").stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .filter(|record: &serde_json::Value| record["type"] == "import")
        .collect();
    let imports = |path: &str| -> Vec<String> {
        let records = records.iter().filter(|r| r["path"] == path);
        records
            .map(|r| {
                let (name, resolved) = (&r["imported_name"], &r["resolved_path"]);
                format!(
                    "{} {} {name} {resolved}",
                    r["line"],
                    r["source"].as_str().unwrap()
                )
            })
            .collect()
    };
    let script = [
        "1 ./a null \"src/a.ts\"",
        "2 ./b null \"src/b.ts\"",
        "5 ./side.css null null",
        "6 ./c null \"src/c.tsx\"",
        "9 ./lazy null \"src/lazy.ts\"",
        "10 ./r null \"src/r.js\"",
        "11 ./q null \"src/q.ts\"",
        "12 react null null",
    ];
    assert_eq!(imports("src/index.ts"), script);
    let python = [
        "1 os.path null null",
        "1 json null null",
        "2 .. \"m\" \"pkg/m.py\"",
        "3 ...a \"b\" \"a.py\"",
        "3 ...a \"c\" \"a.py\"",
        "4 . \"*\" \"pkg/sub/__init__.py\"",
    ];
    assert_eq!(imports("pkg/sub/mod.py"), python);
    let rust = ["a::b", "a::c", "a::e", "a::e::f", "a::g::*"];
    let rust = rust.map(|source| format!("1 {source} null null"));
    assert_eq!(imports("lib.rs"), rust);
    let package = ["1 .b \"c\" \"pkg/b/__init__.py\""];
    assert_eq!(imports("pkg/a.py"), package, "a package shadows its module");
    assert!(
        imports("open.ts").is_empty(),
        "unterminated specifiers are skipped"
    );
}

#[test]
fn typescript_and_tsx_use_their_own_grammars() {
    let project = Project::new();
    for file in ["assertion.ts", "assertion.tsx"] {
        project.write(file, "const text = <string>value;");
    }
    let stderr = String::from_utf8(project.run("extract").stderr).unwrap();
    assert!(stderr.contains("assertion.tsx:") && !stderr.contains("assertion.ts:"));
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
fn commands_skip_unreadable_files_and_measure_broken_ones() {
    let project = Project::new();
    project.write("good.rs", "//! Good.\n/// Works.\npub fn works() {}\n");
    project.write("broken.rs", "fn broken( {");
    project.write("locked/hidden.rs", "fn hidden() {}");
    #[cfg(unix)]
    let lock = |mode| {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(project.0.join("locked"), fs::Permissions::from_mode(mode)).unwrap()
    };
    #[cfg(unix)]
    lock(0o000);
    let tree = topo_command().arg(&project.0).output();
    let outputs = [project.run("score"), project.run("extract"), tree.unwrap()];
    #[cfg(unix)]
    lock(0o755);
    for output in outputs {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stderr}");
        assert!(stderr.contains("locked") || cfg!(not(unix)), "{stderr}");
    }
    for command in ["score", "extract"] {
        let stderr = project.run(command).stderr;
        assert!(String::from_utf8_lossy(&stderr).contains("broken.rs: syntax errors"));
    }
    let find = topo_command()
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
        let output = topo_command()
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
fn tree_nests_a_directory_beside_its_same_named_file() {
    let project = Project::new();
    project.write("a.rs", "//! First file.\n");
    project.write("a/b.rs", "//! Nested file.\n");
    project.write("c.rs", "fn c() {}\n");
    let output = topo_command().arg(&project.0).output().unwrap();
    let tree = String::from_utf8(output.stdout).unwrap();
    let body: Vec<_> = tree.lines().skip(1).collect();
    assert_eq!(
        body,
        [
            "├─ a/",
            "│  └─ b.rs  #── Nested file.",
            "├─ a.rs  #───── First file.",
            "└─ c.rs",
        ]
    );
}

#[test]
fn tree_descriptions_skip_licenses_pragmas_and_headings() {
    let project = Project::new();
    project.write(
        "menu.tsx",
        "'use client';\n/**\n * Renders the menu.\n *\n * More.\n */\nexport function Menu() {}",
    );
    project.write(
        "license.ts",
        "/**\n * Copyright (c) Meta.\n */\n/** @jsx h */\nimport x from 'y';",
    );
    project.write(
        "tools.py",
        "#!/usr/bin/env python\n\"\"\"\nTools\n=====\n\nRuns tools.\n\"\"\"",
    );
    project.write("flags.rs", "//!\n//! Parses flags.\nfn main() {}");
    project.write(
        "bang.ts",
        "#!/usr/bin/env node\n//! Starts the CLI.\nmain();\n",
    );
    project.write("empty.ts", "/**\n */\nexport const x = 1;");
    project.write(
        "lint.ts",
        "/* eslint-disable */\n/**\n * Lints.\n * @param a The a.\n */",
    );
    project.write("opened.ts", "/**/\nexport const x = 1;\n/** Doc. */");
    project.write(
        "a_file_name_long_enough_to_push_every_other_description_off_the_screen.rs",
        "fn f() {}",
    );
    let output = topo_command().arg(&project.0).output().unwrap();
    let tree = String::from_utf8(output.stdout).unwrap();
    let described = |entry: &str, doc: &str| {
        tree.lines()
            .any(|line| line.starts_with(&format!("{entry}  #─")) && line.ends_with(doc))
    };
    assert!(described("├─ flags.rs", "─ Parses flags."), "{tree}");
    assert!(described("├─ bang.ts", "─ Starts the CLI."), "{tree}");
    assert!(described("├─ lint.ts", "─ Lints."), "{tree}");
    assert!(described("├─ menu.tsx", "─ Renders the menu."), "{tree}");
    assert!(described("└─ tools.py", "─ Runs tools."), "{tree}");
    for bare in ["├─ empty.ts", "├─ license.ts", "├─ opened.ts"] {
        assert!(tree.lines().any(|line| line == bare), "{tree}");
    }
}

#[test]
fn bands_rate_lines_and_cognitive_at_their_boundaries() {
    let project = Project::new();
    let mut source = String::new();
    for lines in [40, 41, 75, 76] {
        let body = "  x();\n".repeat(lines - 2);
        source += &format!("function lines{lines}() {{\n{body}}}\n");
    }
    for ifs in [10, 11, 15, 16] {
        source += &format!(
            "function cognitive{ifs}() {{ {}}}\n",
            "if (x) {} ".repeat(ifs)
        );
    }
    project.write("bands.ts", &source);
    let records: Vec<serde_json::Value> = String::from_utf8(project.run("extract").stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for (name, band) in [
        ("lines40", "green"),
        ("lines41", "yellow"),
        ("lines75", "yellow"),
        ("lines76", "red"),
        ("cognitive10", "green"),
        ("cognitive11", "yellow"),
        ("cognitive15", "yellow"),
        ("cognitive16", "red"),
    ] {
        let band_of = |record: &&serde_json::Value| record["name"] == name;
        let record = records.iter().find(band_of).expect(name);
        assert_eq!(record["metrics"]["band"], band, "{name}");
    }
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

#[test]
fn find_matches_qualified_names_and_the_tree_reads_a_linked_root() {
    let project = Project::new();
    project.write(
        "src/metrics.rs",
        "pub struct FileInfo {}\nimpl FileInfo {\n    pub fn units(&self) {}\n}\n",
    );
    let topo = |args: &[&str]| {
        let output = topo_command()
            .args(args)
            .current_dir(&project.0)
            .output()
            .unwrap();
        String::from_utf8(output.stdout).unwrap()
    };
    let found = "1 matches for '{query}':\nsrc/metrics.rs::FileInfo [struct] (L1–1)\n";
    for query in [
        "FileInfo",
        "src/metrics::FileInfo",
        "src/metrics.rs::FileInfo",
    ] {
        assert_eq!(topo(&["find", query]), found.replace("{query}", query));
    }
    let method = "src/metrics.rs::FileInfo::units [function] (L3–3)";
    assert!(topo(&["find", "units"]).contains(method));
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(project.0.join("src"), project.0.join("linked")).unwrap();
        assert!(topo(&["linked"]).contains("metrics.rs"));
    }
}

#[test]
fn discovery_keeps_sources_and_skips_generated_and_installed_code() {
    let project = Project::new();
    for file in [
        "src/bin/tool.rs",
        "build/build.ts",
        "src/App.selection.ts",
        "real/r.rs",
        "src/types.d.ts",
        "src/app.min.js",
        "venv/lib/v.py",
        "lib/site-packages/s.py",
        "__pycache__/c.py",
        ".hidden/h.rs",
        "README.md",
    ] {
        project.write(file, "export function f() {}\n");
    }
    project.write("real/r.rs", "pub fn f() {}\n");
    #[cfg(unix)]
    std::os::unix::fs::symlink(project.0.join("real"), project.0.join("linked")).unwrap();
    let topo = |args: &[&str]| {
        let output = topo_command()
            .args(args)
            .current_dir(&project.0)
            .output()
            .unwrap();
        (
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        )
    };
    let files = |stdout: String| {
        let records = stdout
            .lines()
            .map(|line| serde_json::from_str(line).unwrap());
        let files = records.filter(|record: &serde_json::Value| record["type"] == "file");
        files
            .map(|record| record["path"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    let scanned = [
        "build/build.ts",
        "real/r.rs",
        "src/App.selection.ts",
        "src/bin/tool.rs",
    ];
    assert_eq!(files(topo(&["extract"]).0), scanned);
    let (table, stderr) = topo(&["score", "src", "./src", "src/types.d.ts", "README.md"]);
    assert!(
        stderr.contains("README.md: skipped: unsupported language"),
        "{stderr}"
    );
    for file in ["src/App.selection.ts", "src/bin/tool.rs", "src/types.d.ts"] {
        assert_eq!(table.matches(&format!("│ {file} ")).count(), 1, "{table}");
    }
    assert!(
        topo(&["score"]).0.contains("│ build/build.ts "),
        "score defaults to ."
    );
    #[cfg(unix)]
    {
        assert_eq!(files(topo(&["extract", "linked"]).0), ["r.rs"]);
        assert!(topo(&["score", "linked"]).0.contains("│ linked/r.rs "));
        assert!(topo(&["find", "f", "linked"])
            .0
            .contains("linked/r.rs::f [function]"));
    }
}

#[test]
fn scans_follow_git_ignores_and_score_skips_tests_unless_asked() {
    let project = Project::new();
    for file in [
        "src/real.ts",
        "src/a.test.ts",
        "src/b.spec.tsx",
        "src/c.stories.tsx",
        "src/__tests__/d.ts",
        "src/__mocks__/e.ts",
        "tests/f.rs",
        "e2e/g.ts",
        "pkg/test_h.py",
        "pkg/i_test.py",
        "pkg/conftest.py",
        "ignored/j.rs",
    ] {
        project.write(file, "export function f() {}\n");
    }
    project.write(".gitignore", "ignored/\n");
    let topo = |args: &[&str]| {
        let output = topo_command()
            .args(args)
            .current_dir(&project.0)
            .output()
            .unwrap();
        String::from_utf8(output.stdout).unwrap()
    };
    let rows = |table: String| table.lines().filter(|row| row.starts_with("│ ")).count() - 1;
    assert_eq!(
        rows(topo(&["score"])),
        2,
        "real.ts and ignored/j.rs outside git"
    );
    let git = Command::new("git")
        .arg("init")
        .arg("-q")
        .current_dir(&project.0)
        .status();
    if !git.is_ok_and(|status| status.success()) {
        return;
    }
    assert!(topo(&["score"]).contains("│ src/real.ts "));
    assert_eq!(rows(topo(&["score"])), 1, "git ignores ignored/");
    assert_eq!(rows(topo(&["score", "--tests"])), 11);
    assert_eq!(
        rows(topo(&["score", "tests"])),
        1,
        "a named folder is scored"
    );
    assert_eq!(
        topo(&["extract"])
            .lines()
            .filter(|l| l.contains("\"type\":\"file\""))
            .count(),
        11
    );
}
