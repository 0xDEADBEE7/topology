//! Exercises React assessment through the public command-line interface.

use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

fn topo(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_topo"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn react_files_support_scoring_inspection_and_extraction() {
    let path = "tests/fixtures/typescript/components.tsx";
    let score = topo(&["score", path, "--detail", "--no-colour"]);
    assert!(score.contains("Greeting"));
    assert!(
        !topo(&["score", path]).contains('\x1b'),
        "colour in piped output"
    );
    assert!(score.contains("App"));
    let outline = topo(&["--full-docstring", path]);
    assert!(outline.contains("Chooses the greeting content."));
    let records = topo(&["extract", "tests/fixtures/typescript"]);
    let values: Vec<serde_json::Value> = records
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for name in ["Greeting", "App"] {
        let record = values.iter().find(|r| r["name"] == name).expect(name);
        assert_eq!(record["language"], "typescript");
        assert_eq!(record["metrics"]["cognitive"], 1, "{name}");
        assert!(record["description"].is_string(), "{name} keeps its JSDoc");
    }
}

fn records(path: &str) -> Vec<serde_json::Value> {
    topo(&["extract", path])
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// Asserts the code lines and cognitive complexity of each named function.
fn assert_units(records: &[serde_json::Value], expected: &[(&str, u64, u64)]) {
    for (name, loc, cognitive) in expected {
        let record = records.iter().find(|r| r["name"] == *name).expect(name);
        let metrics = (&record["metrics"]["loc"], &record["metrics"]["cognitive"]);
        assert_eq!(metrics, (&(*loc).into(), &(*cognitive).into()), "{name}");
    }
}

/// Lists the functions the score table marks as documented.
fn documented(path: &str) -> Vec<String> {
    let table = topo(&["score", path, "--detail", "--no-colour"]);
    let rows = table.lines().filter(|row| row.ends_with("yes │"));
    let names = rows.filter_map(|row| row.strip_prefix("│   ")?.split(" (L").next());
    names.map(str::to_owned).collect()
}

/// Names, folds and rates React and plain functions as roadmap M3 and SonarSource 1.7 specify.
#[test]
fn react_semantics_name_fold_and_rate_functions() {
    let path = "tests/fixtures/typescript/semantics.tsx";
    let records = records(path);
    let functions: Vec<_> = records.iter().filter(|r| r["type"] == "function").collect();
    assert_units(
        &records,
        &[
            ("Button", 3, 0),
            ("Card", 1, 0),
            ("List", 8, 4),
            ("select", 1, 1),
            ("default", 1, 0),
            ("Store.increment", 3, 0),
            ("Store.reset", 3, 0),
            ("onSave", 1, 0),
            ("sumOfPrimes", 12, 7),
            ("getWords", 10, 1),
            ("classify", 9, 5),
            ("describe(\"store\")", 2, 0),
            ("it(\"increments\")", 5, 1),
            ("register(…)", 1, 0),
            ("stream.on(\"data\")", 3, 0),
            ("stream.on(…).on(\"end\")", 3, 0),
            ("pick", 1, 0),
            ("it.each(…)(\"picks %#\")", 4, 0),
            ("RowBase", 1, 0),
            ("memo(…)", 1, 0),
            ("….forEach(…)", 1, 0),
        ],
    );
    assert!(
        !functions.iter().any(|r| r["name"] == "Row"),
        "a comparator is not Row"
    );
    let classes = records.iter().filter(|r| r["type"] == "class");
    let classes: Vec<_> = classes.map(|r| r["name"].as_str().unwrap()).collect();
    assert_eq!(classes, ["Store"]);
    assert_eq!(
        functions.len(),
        23,
        "callbacks must fold, not appear as <anonymous>"
    );
    let ids: std::collections::HashSet<_> = records.iter().map(|r| &r["id"]).collect();
    assert_eq!(ids.len(), records.len(), "ids must be unique");
    let outline = topo(&[path]);
    assert!(outline.contains("\n    Store.increment (L25–27)"));
    assert!(outline.contains("\n  register(…) (L79–79)\n  register(…) (L79–79)\n"));
}

/// Rust and Python units follow the same naming, documentation and SonarSource rules.
#[test]
fn rust_and_python_semantics_name_and_rate_functions() {
    let path = "tests/fixtures/rust/semantics.rs";
    assert_units(
        &records(path),
        &[
            ("sum_of_primes", 12, 7),
            ("get_words", 8, 1),
            ("mixed", 11, 9),
            ("Shape::new", 4, 0),
            ("helper", 3, 0),
            ("Area::double", 3, 0),
            ("Shape::fmt", 3, 0),
            ("new", 3, 0),
            ("usage", 5, 0),
            ("Shape::area", 3, 0),
            ("count_down", 6, 6),
        ],
    );
    let docs = documented(path);
    assert_eq!(docs, ["sum_of_primes", "Shape::new", "count_down"]);
    assert!(topo(&[&format!("{path}::Shape::new")]).contains("pub fn new() -> Self {"));
    assert!(topo(&[&format!("{path}::new")]).contains("fn new() -> Shape {"));
    assert!(topo(&[&format!("{path}::Shape::area")]).contains("fn area(&self) -> u32 {"));
    let path = "tests/fixtures/python/semantics.py";
    assert_units(
        &records(path),
        &[
            ("sum_of_primes", 8, 6),
            ("get_words", 8, 1),
            ("mixed", 8, 8),
            ("Shape.area", 5, 1),
            ("page", 6, 0),
            ("Popen._execute_child", 2, 0),
            ("Popen.Inner.deep", 2, 0),
            ("fetch", 2, 0),
        ],
    );
    assert_eq!(documented(path), ["sum_of_primes", "Shape.area", "fetch"]);
    assert!(topo(&[&format!("{path}::Shape.area")]).contains("def area(self):"));
    assert!(topo(&[&format!("{path}::Popen.Inner.deep")]).contains("def deep(self):"));
}

/// Pins the extract record shapes; regenerate basic.jsonl only for a deliberate change.
#[test]
fn extract_output_matches_its_contract() {
    let expected = include_str!("fixtures/typescript/basic.jsonl");
    assert_eq!(
        topo(&["extract", "tests/fixtures/typescript/basic.ts"]),
        expected
    );
}

/// Size budget for `src` in files and code lines: lower it as the code shrinks, never raise it.
const MAX_FILES: usize = 21;
const MAX_LINES: u64 = 1_857;
const MAX_DEPENDENCIES: usize = 5;
/// Largest share of function lines in yellow functions, per mille; ratchets down.
const MAX_YELLOW_PER_MILLE: u64 = 0;

#[test]
fn closed_output_pipes_end_quietly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_topo"))
        .args(["extract", "src"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut first = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut first)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(first.starts_with('{'));
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    #[cfg(unix)]
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&output.status),
        Some(13)
    );
}

#[test]
fn partially_parsed_files_keep_their_functions() {
    let output = Command::new(env!("CARGO_BIN_EXE_topo"))
        .args(["extract", "tests/fixtures/partial"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let records: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for (file, function) in [
        ("generic-import.ts", "load"),
        ("call-signatures.ts", "create"),
        ("jsx-entity.tsx", "Search"),
        ("anonymous-default.d.ts", "helper"),
        ("nul-template.ts", "separator"),
    ] {
        assert!(stderr.contains(file), "{file} parses now; drop the fixture");
        assert!(records
            .iter()
            .any(|record| record["path"] == file && record["name"] == function));
    }
}

#[test]
fn project_meets_its_structural_quality_bands() {
    let records = topo(&["extract", "src"]);
    let (mut files, mut lines, mut function_lines, mut yellow_lines) = (0, 0, 0, 0);
    for line in records.lines() {
        let record: serde_json::Value = serde_json::from_str(line).unwrap();
        let metrics = &record["metrics"];
        let label = format!("{} {}", record["path"], record["name"]);
        if record["type"] == "file" {
            files += 1;
            lines += metrics["loc"].as_u64().unwrap();
            assert!(
                record["description"]
                    .as_str()
                    .is_some_and(|doc| !doc.is_empty()),
                "module docs: {label}"
            );
            assert!(metrics["loc"].as_u64().unwrap() <= 400, "LoC: {label}");
            assert_eq!(metrics["parse_errors"], 0, "partial parse: {label}");
        } else if record["type"] == "function" {
            assert_ne!(metrics["band"], "red", "red function: {label}");
            function_lines += metrics["loc"].as_u64().unwrap();
            if metrics["band"] == "yellow" {
                yellow_lines += metrics["loc"].as_u64().unwrap();
            }
        }
    }
    assert!(
        1000 * yellow_lines <= MAX_YELLOW_PER_MILLE * function_lines,
        "{yellow_lines} of {function_lines} function lines are in yellow functions"
    );
    assert!(
        (1..=MAX_FILES).contains(&files),
        "{files} files over budget"
    );
    assert!(lines <= MAX_LINES, "{lines} code lines over budget");
    let dependencies = include_str!("../Cargo.toml")
        .split("[dependencies]")
        .nth(1)
        .unwrap_or_default()
        .lines()
        .take_while(|line| !line.starts_with('['))
        .filter(|line| line.contains('='))
        .count();
    assert!(
        dependencies <= MAX_DEPENDENCIES,
        "{dependencies} dependencies"
    );
}
