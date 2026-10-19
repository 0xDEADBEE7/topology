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
    let score = topo(&["score", path, "--detail", "--all-metrics", "--no-colour"]);
    assert!(score.contains("Greeting"));
    assert!(
        !topo(&["score", path]).contains('\x1b'),
        "colour in piped output"
    );
    assert!(score.contains("App"));
    let outline = topo(&["--full-docstring", path]);
    assert!(outline.contains("Chooses the greeting content."));
    let single = topo(&["extract", "tests/fixtures/typescript/basic.ts"]);
    assert!(single.contains(r#""path":"basic.ts""#) && single.contains("classify"));
    let records = topo(&["extract", "tests/fixtures/typescript"]);
    let values: Vec<serde_json::Value> = records
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(values
        .iter()
        .any(|record| record["name"] == "Greeting" && record["language"] == "typescript"));
}

/// Size budget for `src`: lower these as the code shrinks, never raise them to pass.
const MAX_FILES: usize = 42;
const MAX_LINES: u64 = 2_650;
const MAX_DEPENDENCIES: usize = 6;

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
    assert_ne!(output.status.code(), Some(101));
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
    let (mut files, mut lines) = (0, 0);
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
        } else if record["type"] == "function" {
            assert!(metrics["cc"].as_u64().unwrap() < 15, "CC: {label}");
            assert!(
                metrics["cognitive"].as_u64().unwrap() < 15,
                "cognitive: {label}"
            );
            assert!(
                metrics["halstead"].as_f64().unwrap() < 1000.0,
                "Halstead: {label}"
            );
            assert!(metrics["loc"].as_u64().unwrap() <= 60, "LoC: {label}");
        }
    }
    assert!(
        (1..=MAX_FILES).contains(&files),
        "{files} files over budget"
    );
    assert!(lines <= MAX_LINES, "{lines} lines over budget");
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
