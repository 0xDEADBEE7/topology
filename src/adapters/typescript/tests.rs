//! Regression coverage for TypeScript and React assessment.

use super::{Adapter, TypeScriptAdapter};

const SAMPLE: &str = include_str!("../../../tests/fixtures/typescript/basic.ts");

#[test]
fn parses_typescript_functions_and_decisions() {
    let metrics = TypeScriptAdapter
        .analyse("basic.ts", SAMPLE)
        .expect("fixture should parse");

    assert_eq!(metrics.language, "typescript");
    assert_eq!(metrics.loc, 11);
    assert_eq!(metrics.fns.len(), 2);
    assert_eq!(metrics.fns[0].name, "classify");
    assert_eq!(metrics.fns[0].line_start, 1);
    assert!(metrics.fns[0].cc >= 4);
    assert_eq!(metrics.fns[1].name, "render");
    assert!(metrics.fns[1].cc >= 2);
}

#[test]
fn assesses_react_components_and_jsdoc() {
    let source = include_str!("../../../tests/fixtures/typescript/components.tsx");
    let metrics = TypeScriptAdapter.analyse("components.tsx", source).unwrap();
    assert_eq!(metrics.fns.len(), 2);
    assert!(metrics.has_docstring);
    assert_eq!(metrics.fns[0].name, "Greeting");
    assert_eq!(metrics.fns[0].cc, 3);
    assert!(metrics.fns[0].has_docstring);
    assert_eq!(metrics.fns[1].name, "App");
    assert_eq!(metrics.fns[1].cc, 2);
    assert!(metrics.fns[1].has_docstring);
}

#[test]
fn nested_functions_have_independent_complexity() {
    for source in [
        "function outer() { function inner() { if (ok) return 1; } return inner; }",
        "const outer = () => { const inner = () => ok ? 1 : 0; return inner; };",
    ] {
        let metrics = TypeScriptAdapter.analyse("nested.ts", source).unwrap();
        assert_eq!(metrics.fns.len(), 2);
        assert_eq!(metrics.fns[0].cc, 1);
        assert_eq!(metrics.fns[1].cc, 2);
        assert_eq!(metrics.cc, 2);
    }
}

#[test]
fn hooks_and_methods_are_assessed() {
    let source = "export function useReady() { return useMemo(() => ok || fallback, []); }
        class View { render() { return <div>{ok ? 'yes' : 'no'}</div>; } }";
    let metrics = TypeScriptAdapter.analyse("hooks.tsx", source).unwrap();
    assert_eq!(metrics.classes[0].name, "View");
    assert_eq!(metrics.fns.len(), 3);
    assert_eq!(metrics.fns[0].cc, 1);
    assert_eq!(metrics.fns[1].cc, 2);
    assert_eq!(metrics.fns[2].name, "render");
    assert_eq!(metrics.fns[2].cc, 2);
}

#[test]
fn selects_grammar_by_extension() {
    let assertion = "const text = <string>value;";
    let errors = |path| {
        TypeScriptAdapter
            .analyse(path, assertion)
            .unwrap()
            .parse_errors
    };
    assert_eq!(errors("assertion.ts"), 0);
    assert!(errors("assertion.tsx") > 0);
}
