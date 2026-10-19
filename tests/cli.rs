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
    let score = topo(&["score", path, "--detail"]);
    assert!(score.contains("Greeting"));
    assert!(
        !topo(&["score", path]).contains('\x1b'),
        "colour in piped output"
    );
    assert!(score.contains("App"));
    let outline = topo(&[path]);
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
    let table = topo(&["score", path, "--detail"]);
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
            ("register(…)", 0, 0),
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

/// Lists each declaration as `name exported parent [signature] description`.
fn declarations(path: &str) -> Vec<String> {
    let records = records(path);
    let text = |value: &serde_json::Value| value.as_str().unwrap_or("-").to_owned();
    let units = records.iter().filter(|r| r["type"] != "import");
    let rows = units.map(|r| {
        let (name, parent, signature) =
            (text(&r["name"]), text(&r["parent"]), text(&r["signature"]));
        format!(
            "{name} {} {parent} [{signature}] {}",
            r["exported"],
            text(&r["description"])
        )
    });
    rows.collect()
}

/// Docs come from the doc node the walker finds, headers stop at the body, and exports
/// follow each language's interface rules.
#[test]
fn declarations_carry_docs_signatures_exports_and_parents() {
    assert_eq!(
        declarations("tests/fixtures/typescript/declarations.ts"),
        [
            "- null - [-] Declares functions with docs, headers and exports.",
            "add true - [export function add(a: number, b: number): number] Adds two numbers.",
            "tagged true - [function tagged(value: number)] -",
            "local true - [const local = () =>] -",
            "Counter.increment false Counter [increment = () =>] Increments the count.",
            "Counter.reset false Counter [reset()] Resets the count.",
            "click false - [click: () =>] Handles a click.",
            "other false - [function other()] -",
            "footer true - [export function footer()] Renders the copyright footer.",
            "legacy true - [export function legacy()] -",
            "internal true - [export function internal()] -",
            "Store.handle false Store [@action handle = () =>] Handles a click.",
            "Store.onKey false Store [onKey = debounce((event) =>] -",
            "home false - [home: () =>] -",
            "about false - [about: () =>] -",
            "user false - [user: async (id) =>] Loads the user.",
            "wrapper false - [function wrapper()] -",
            "local false - [function local()] -",
            "Counter true - [export class Counter] -",
            "Store true - [export class Store] -",
        ]
    );
    assert_eq!(
        declarations("tests/fixtures/rust/declarations.rs"),
        [
            "- null - [-] Declares items with docs, headers and exports.",
            "build true - [pub fn build() -> u32] Builds a value across two lines.",
            "shared true - [pub(crate) fn shared()] -",
            "private false - [fn private()] -",
            "Holder::get true Holder [pub fn get(&self) -> u32] Reads the value.",
            "bold true - [pub fn bold()] **Bold** first words.",
            "aliased true - [pub fn aliased()] -",
            "Unit::new true Unit [pub fn new() -> Self] -",
            "generated true - [pub fn generated()] Joins generated docs.",
            "escaped true - [pub fn escaped()] First \"paragraph\".",
            "interleaved true - [pub fn interleaved()] Rebuilds from parts. Reads more detail.",
            "Holder true - [pub struct Holder] Holds a value.",
            "Unit true - [pub struct Unit;] -",
        ]
    );
    assert_eq!(
        declarations("tests/fixtures/python/declarations.py"),
        [
            "- null - [-] Declares functions with docs, headers and exports.",
            "fetch true - [def fetch(url):] Fetches a page.",
            "_private false - [def _private():] Single-quoted and joined.",
            "Shape.area false Shape [def area(self):] Measures it.",
            "config true - [def config():] /etc/topo.toml is read first.",
            "Shape true - [class Shape:] -",
        ]
    );
}

/// The score's Doc column and extract's descriptions read the same doc node.
#[test]
fn doc_column_matches_extracted_descriptions() {
    for language in ["typescript", "rust", "python"] {
        for entry in std::fs::read_dir(format!("tests/fixtures/{language}")).unwrap() {
            let path = entry.unwrap().path();
            let path = path.to_str().unwrap();
            if path.ends_with(".jsonl") {
                continue;
            }
            let functions = records(path)
                .into_iter()
                .filter(|r| r["type"] == "function");
            let described = functions.filter(|r| r["description"].is_string());
            let described: Vec<_> = described
                .map(|r| r["name"].as_str().unwrap().to_owned())
                .collect();
            assert_eq!(documented(path), described, "{path}");
        }
    }
}

/// Lists each function as `name start–end loc`.
fn extents(path: &str) -> Vec<String> {
    let records = records(path);
    let functions = records.iter().filter(|r| r["type"] == "function");
    let row = |r: &serde_json::Value| {
        let (lines, loc) = (&r["lines"], &r["metrics"]["loc"]);
        format!(
            "{} {}–{} {loc}",
            r["name"].as_str().unwrap(),
            lines[0],
            lines[1]
        )
    };
    functions.map(row).collect()
}

/// Units start at the binding that names them, own the rows only they hold, and take
/// their names from their own declarations; blank Unicode rows are not code.
#[test]
fn units_start_at_their_bindings_and_own_their_rows() {
    let path = "tests/fixtures/typescript/ranges.tsx";
    let expected = [
        "viaPair 4–6 3",
        "viaAssignment 10–11 2",
        "render 14–14 1",
        "UserTable.visible 16–17 2",
        "UserTable.render 18–20 3",
        "factory 23–23 1",
        "describe 24–24 1",
        "Button 26–28 2",
        "withAuth 31–38 0",
        "WithAuth 31–38 6",
        "fetch(…).then(…) 40–45 3",
        "parse 41–43 3",
        "loaded 40–45 0",
    ];
    assert_eq!(extents(path), expected);
    let records = records(path);
    for (name, doc) in [
        ("viaPair", "Declared with an object key."),
        ("viaAssignment", "Declared by assignment."),
        ("UserTable.visible", "Declared as a class field."),
    ] {
        let record = records.iter().find(|r| r["name"] == name).expect(name);
        assert_eq!(record["description"], doc);
    }
    assert!(topo(&[path]).contains("\n  loaded (L40–45)\n    parse (L41–43)\n"));
    let expected = [
        "[u8; 4]::size 8–8 1",
        "(A, B)::size 12–12 1",
        "dyn Size + Send::twice 16–16 1",
        "*const u8::size 20–20 1",
        "Vec::size 24–27 3",
    ];
    assert_eq!(extents("tests/fixtures/rust/impls.rs"), expected);
    let path = "tests/fixtures/python/docstrings.py";
    let expected = [
        "label 4–6 2",
        "formatted 9–11 2",
        "raw_bytes 14–16 2",
        "plain 19–22 2",
    ];
    assert_eq!(extents(path), expected);
    assert_eq!(
        documented(path),
        ["plain"],
        "f-strings and bytes are not docstrings"
    );
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

/// Size budget for `src` in files and code lines: the size at release 0.2.0 rounded up to
/// the next 25. Lower it as the code shrinks; never raise it.
const MAX_FILES: usize = 9;
const MAX_LINES: u64 = 1_425;
const MAX_DEPENDENCIES: usize = 5;
/// Largest share of function lines in yellow functions, per mille; ratchets down.
const MAX_YELLOW_PER_MILLE: u64 = 0;

#[test]
fn closed_output_pipes_end_quietly() {
    // Far more output than a pipe buffer holds, so topo is still writing when it closes.
    let directory = std::env::temp_dir().join(format!("topology-pipe-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source: String = (0..5000)
        .map(|i| format!("export function f{i}() {{}}\n"))
        .collect();
    std::fs::write(directory.join("big.ts"), source).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_topo"))
        .args(["extract", directory.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut first = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut first)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    std::fs::remove_dir_all(&directory).unwrap();
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
        ("anonymous-default.ts", "helper"),
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

/// Help and the version print to stdout; exit 1 means nothing matched, 2 a usage error.
#[test]
fn commands_follow_the_exit_code_contract() {
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_topo"))
            .args(args)
            .output()
            .unwrap();
        let text = |bytes: Vec<u8>| String::from_utf8(bytes).unwrap();
        (
            output.status.code(),
            text(output.stdout),
            text(output.stderr),
        )
    };
    let (status, help, _) = run(&["--help"]);
    assert_eq!(status, Some(0));
    let forms = [
        "topo [<path>",
        "topo find",
        "topo score",
        "topo extract",
        "--version",
    ];
    for form in forms {
        assert!(help.contains(form), "usage lists {form}");
    }
    assert_eq!(run(&["score", "-h"]).0, Some(0), "help anywhere");
    let (status, version, _) = run(&["--version"]);
    assert_eq!((status, version.starts_with("topo ")), (Some(0), true));
    let (status, tree, _) = run(&[]);
    assert_eq!(
        (status, tree.contains("├─ src/")),
        (Some(0), true),
        "bare topo is topo ."
    );
    let found = run(&["find", "classify", "tests/fixtures/typescript"]);
    let path = "tests/fixtures/typescript/basic.ts::classify [function] (L1–7)";
    assert_eq!((found.0, found.1.contains(path)), (Some(0), true));
    let elsewhere = run(&["find", "classify", "tests/fixtures/rust"]);
    assert_eq!(elsewhere.0, Some(1), "find searches only its directory");
    let (status, candidates, _) = run(&["tests/fixtures/rust/impls.rs::size"]);
    assert_eq!(status, Some(1), "an ambiguous name lists its candidates");
    assert_eq!(candidates.lines().count(), 5, "{candidates}");
    assert!(candidates.contains("tests/fixtures/rust/impls.rs::Vec::size (L24–27)"));
    let missing = run(&["tests/fixtures/typescript/basic.ts::missing"]);
    assert_eq!(missing.0, Some(1));
    let (status, both, _) = run(&["tests/fixtures/typescript/semantics.tsx::register(…)"]);
    let shown = both.matches("::register(…) (L79–79)").count();
    assert_eq!((status, shown), (Some(0), 2), "every exact match is shown");
    let usages: [&[&str]; 7] = [
        &["src", "tests"],
        &["--bogus"],
        &["find"],
        &["find", "x", "src", "tests"],
        &["find", "--bogus"],
        &["extract", "src", "tests"],
        &["score", "--bogus"],
    ];
    for usage in usages {
        let (status, _, stderr) = run(usage);
        assert_eq!(status, Some(2), "{usage:?}");
        assert!(stderr.starts_with("error: "), "{usage:?}: {stderr}");
    }
}
