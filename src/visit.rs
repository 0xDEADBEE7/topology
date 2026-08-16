use crate::visitor::BlockVisitor;
use syn::{spanned::Spanned, visit::Visit, File, ImplItemFn, ItemFn, ItemStruct};

#[derive(Debug, Default, Clone)]
pub struct RawCounts {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub decisions: u32,
    pub cognitive: u32,
    pub n1: u32,
    pub n2: u32,
    pub dn1: u32,
    pub dn2: u32,
    pub has_docstring: bool,
}

struct FnVisitor {
    raw: Vec<RawCounts>,
    classes: Vec<crate::metrics::ClassMetrics>,
}

impl FnVisitor {
    fn record(
        &mut self,
        name: &str,
        line_start: usize,
        line_end: usize,
        body: &syn::Block,
        has_docstring: bool,
    ) {
        let mut bv = BlockVisitor::new();
        bv.visit_block(body);
        self.raw.push(RawCounts {
            name: name.to_owned(),
            line_start,
            line_end,
            decisions: 1 + bv.decisions,
            cognitive: bv.cognitive,
            n1: bv.n1,
            n2: bv.n2,
            dn1: bv.n1,
            dn2: bv.n2,
            has_docstring,
        });
    }
}

impl<'ast> Visit<'ast> for FnVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.record(
            &node.sig.ident.to_string(),
            node.sig.ident.span().start().line,
            node.span().end().line,
            &node.block,
            has_doc_attr(&node.attrs),
        );
        // do NOT recurse — nested fns are collected as separate entries
    }

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.classes.push(crate::metrics::ClassMetrics {
            name: node.ident.to_string(),
            line_start: node.ident.span().start().line,
            line_end: node.span().end().line,
        });
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.record(
            &node.sig.ident.to_string(),
            node.sig.ident.span().start().line,
            node.span().end().line,
            &node.block,
            has_doc_attr(&node.attrs),
        );
    }
}

/// Collect Rust functions and structs while preserving source locations.
pub fn collect(file: &File) -> (Vec<RawCounts>, Vec<crate::metrics::ClassMetrics>, bool) {
    let mut v = FnVisitor {
        raw: Vec::new(),
        classes: Vec::new(),
    };
    v.visit_file(file);
    (v.raw, v.classes, has_doc_attr(&file.attrs))
}

fn has_doc_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("doc"))
}
