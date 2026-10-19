//! Maps file extensions to languages and holds the syntax tables that drive the walker.
//!
//! Adding a language means one `EXTENSIONS` row per extension, a `Language` variant, and
//! a `Spec` naming the grammar's node kinds for the constructs `walk` measures.

use std::path::Path;
use tree_sitter_typescript::{LANGUAGE_TSX, LANGUAGE_TYPESCRIPT};

/// A supported source language.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    Tsx,
    JavaScript,
}

/// Every supported extension; script extensions are listed in import-resolution order.
pub const EXTENSIONS: [(&str, Language); 10] = [
    ("rs", Language::Rust),
    ("py", Language::Python),
    ("ts", Language::TypeScript),
    ("tsx", Language::Tsx),
    ("mts", Language::TypeScript),
    ("cts", Language::TypeScript),
    ("js", Language::JavaScript),
    ("jsx", Language::JavaScript),
    ("mjs", Language::JavaScript),
    ("cjs", Language::JavaScript),
];

impl Language {
    /// Recognizes a supported source file by its extension.
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let extension = path.as_ref().extension()?.to_str()?;
        let known = EXTENSIONS.iter().find(|(known, _)| *known == extension);
        known.map(|(_, language)| *language)
    }

    /// Names the language in reports and extracted records.
    pub fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::TypeScript | Self::Tsx => "typescript",
            Self::JavaScript => "javascript",
        }
    }

    /// Returns the tree-sitter grammar and syntax table for languages the walker measures.
    pub fn grammar(self) -> Option<(tree_sitter::Language, &'static Spec)> {
        match self {
            Self::TypeScript => Some((LANGUAGE_TYPESCRIPT.into(), &ECMA)),
            Self::Tsx | Self::JavaScript => Some((LANGUAGE_TSX.into(), &ECMA)),
            Self::Rust | Self::Python => None,
        }
    }
}

/// Node kinds and tokens a grammar uses for the constructs the walker measures.
pub struct Spec {
    /// Executable function forms.
    pub functions: &'static [&'static str],
    /// Declarations listed as classes in outlines.
    pub classes: &'static [&'static str],
    /// Structures that add 1 plus nesting, with the fields nested one level deeper.
    pub structures: &'static [(&'static str, &'static [&'static str])],
    /// The `if` and `else` kinds; `else` and `else if` add 1 without nesting.
    pub conditional: [&'static str; 2],
    /// Jumps that add 1 when they name a label.
    pub jumps: &'static [&'static str],
    /// Logical operator tokens; each run of like operators adds 1.
    pub logical: &'static [&'static str],
    /// Expressions a binding sees through to reach the function it names.
    pub wrappers: &'static [&'static str],
    /// Calls whose callback implements the binding, as in `const Button = forwardRef(...)`.
    pub wrapper_calls: &'static [&'static str],
    /// Nodes that name the function they bind, with the field holding the name.
    pub bindings: &'static [(&'static str, &'static str)],
    /// Nodes between a declaration and the statement that introduces it.
    pub declarations: &'static [&'static str],
}

/// TypeScript, TSX and JavaScript, which shares the TSX grammar.
const ECMA: Spec = Spec {
    functions: &[
        "function_declaration",
        "function_expression",
        "arrow_function",
        "method_definition",
        "generator_function_declaration",
        "generator_function",
    ],
    classes: &["class_declaration", "abstract_class_declaration", "class"],
    structures: &[
        ("switch_statement", &["body"]),
        ("for_statement", &["body"]),
        ("for_in_statement", &["body"]),
        ("while_statement", &["body"]),
        ("do_statement", &["body"]),
        ("catch_clause", &["body"]),
        ("ternary_expression", &["consequence", "alternative"]),
    ],
    conditional: ["if_statement", "else_clause"],
    jumps: &["break_statement", "continue_statement"],
    logical: &["&&", "||"],
    wrappers: &[
        "parenthesized_expression",
        "as_expression",
        "satisfies_expression",
        "non_null_expression",
    ],
    wrapper_calls: &[
        "forwardRef",
        "memo",
        "useCallback",
        "useMemo",
        "lazy",
        "observer",
    ],
    bindings: &[
        ("variable_declarator", "name"),
        ("public_field_definition", "name"),
        ("pair", "key"),
        ("assignment_expression", "left"),
    ],
    declarations: &[
        "variable_declarator",
        "lexical_declaration",
        "variable_declaration",
        "export_statement",
        "arguments",
        "call_expression",
    ],
};
