//! Collects Rust declarations and delegates function-body measurements.

use crate::metrics::{ClassMetrics, FnMetrics};
use crate::visitor::BlockVisitor;
use syn::{spanned::Spanned, visit::Visit, File, ImplItemFn, ItemFn, ItemStruct};

struct FnVisitor {
    fns: Vec<FnMetrics>,
    classes: Vec<ClassMetrics>,
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
        let mut bv = BlockVisitor::default();
        bv.visit_block(body);
        self.fns.push(FnMetrics {
            name: name.to_owned(),
            line_start,
            line_end,
            cognitive: bv.cognitive,
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
        self.classes.push(ClassMetrics {
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
pub fn collect(file: &File) -> (Vec<FnMetrics>, Vec<ClassMetrics>, bool) {
    let mut v = FnVisitor {
        fns: Vec::new(),
        classes: Vec::new(),
    };
    v.visit_file(file);
    (v.fns, v.classes, has_doc_attr(&file.attrs))
}

fn has_doc_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("doc"))
}
