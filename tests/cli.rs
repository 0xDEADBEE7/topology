//! Exercises React assessment through the public command-line interface.

use std::process::Command;

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
    assert!(score.contains("App"));
    let outline = topo(&["--full-docstring", path]);
    assert!(outline.contains("Chooses the greeting content."));
    let records = topo(&["extract", "tests/fixtures/typescript"]);
    let values: Vec<serde_json::Value> = records
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(values
        .iter()
        .any(|record| record["name"] == "Greeting" && record["language"] == "typescript"));
}

#[test]
fn project_meets_its_structural_quality_bands() {
    let records = topo(&["extract", "src"]);
    let mut files = 0;
    for line in records.lines() {
        let record: serde_json::Value = serde_json::from_str(line).unwrap();
        let metrics = &record["metrics"];
        if metrics.is_null() {
            continue;
        }
        let label = format!("{} {}", record["path"], record["name"]);
        assert!(metrics["cc"].as_u64().unwrap() < 15, "CC: {label}");
        assert!(
            metrics["cognitive"].as_u64().unwrap() < 15,
            "cognitive: {label}"
        );
        assert!(
            metrics["halstead"].as_f64().unwrap() < 1000.0,
            "Halstead: {label}"
        );
        let limit = if record["type"] == "file" {
            files += 1;
            assert!(
                record["description"]
                    .as_str()
                    .is_some_and(|doc| !doc.is_empty()),
                "module docs: {label}"
            );
            assert!(metrics["nmi"].as_f64().unwrap() >= 20.0, "NMI: {label}");
            299
        } else {
            60
        };
        assert!(metrics["loc"].as_u64().unwrap() <= limit, "LoC: {label}");
    }
    assert!(files > 0);
}
