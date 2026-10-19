//! Measures cognitive complexity within Rust function bodies.

use syn::{
    visit::{self, Visit},
    ExprForLoop, ExprIf, ExprMatch, ExprWhile,
};

#[derive(Default)]
/// Walks inside a single function body accumulating cognitive increments.
pub struct BlockVisitor {
    depth: u32,
    pub cognitive: u32,
}

impl BlockVisitor {
    fn branch(&mut self) {
        self.cognitive += 1 + self.depth;
    }
}

impl<'ast> Visit<'ast> for BlockVisitor {
    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        self.branch();
        self.depth += 1;
        visit::visit_expr_if(self, node);
        self.depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast ExprWhile) {
        self.branch();
        self.depth += 1;
        visit::visit_expr_while(self, node);
        self.depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast ExprForLoop) {
        self.branch();
        self.depth += 1;
        visit::visit_expr_for_loop(self, node);
        self.depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        self.cognitive += (1 + self.depth) * node.arms.len() as u32;
        self.depth += 1;
        visit::visit_expr_match(self, node);
        self.depth -= 1;
    }
}
