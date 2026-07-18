use syn::{visit::Visit, File, ImplItemFn, ItemFn};
use proc_macro2::Span;
use crate::visitor::BlockVisitor;

#[derive(Debug, Default, Clone)]
pub struct RawCounts {
    pub name:       String,
    pub line_start: usize,
    pub line_end:   usize,
    pub decisions:  u32,
    pub cognitive:  u32,
    pub n1:         u32,
    pub n2:         u32,
    pub dn1:        u32,
    pub dn2:        u32,
}

struct FnVisitor(pub Vec<RawCounts>);

impl FnVisitor {
    fn record(&mut self, name: &str, span: Span, body: &syn::Block) {
        let mut bv = BlockVisitor::new();
        bv.visit_block(body);
        self.0.push(RawCounts {
            name:       name.to_owned(),
            line_start: span.start().line,
            line_end:   span.end().line,
            decisions:  1 + bv.decisions,
            cognitive:  bv.cognitive,
            n1: bv.n1, n2: bv.n2,
            dn1: bv.n1, dn2: bv.n2,
        });
    }
}

impl<'ast> Visit<'ast> for FnVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.record(&node.sig.ident.to_string(), node.sig.ident.span(), &node.block);
        // do NOT recurse — nested fns are collected as separate entries
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.record(&node.sig.ident.to_string(), node.sig.ident.span(), &node.block);
    }
}

pub fn collect(file: &File) -> Vec<RawCounts> {
    let mut v = FnVisitor(Vec::new());
    v.visit_file(file);
    v.0
}
