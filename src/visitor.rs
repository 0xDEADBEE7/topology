use syn::{
    visit::{self, Visit},
    Expr, ExprBinary, ExprForLoop, ExprIf, ExprMatch, ExprWhile,
};

/// Walks inside a single function body accumulating raw counts.
pub struct BlockVisitor {
    pub depth: u32,
    pub decisions: u32,
    pub cognitive: u32,
    pub n1: u32,
    pub n2: u32,
}

impl BlockVisitor {
    pub fn new() -> Self {
        Self {
            depth: 0,
            decisions: 0,
            cognitive: 0,
            n1: 0,
            n2: 0,
        }
    }

    fn branch(&mut self) {
        self.decisions += 1;
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
        self.decisions += node.arms.len() as u32;
        self.cognitive += (1 + self.depth) * node.arms.len() as u32;
        self.depth += 1;
        visit::visit_expr_match(self, node);
        self.depth -= 1;
    }

    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        use syn::BinOp;
        match node.op {
            BinOp::And(_) | BinOp::Or(_) => {
                self.decisions += 1;
                self.n1 += 1;
            }
            _ => {
                self.n1 += 1;
            }
        }
        visit::visit_expr_binary(self, node);
    }

    fn visit_expr(&mut self, node: &'ast Expr) {
        match node {
            Expr::Try(_) => {
                self.decisions += 1;
                self.n1 += 1;
            }
            Expr::Lit(_) => {
                self.n2 += 1;
            }
            Expr::Path(_) => {
                self.n2 += 1;
            }
            _ => {}
        }
        visit::visit_expr(self, node);
    }
}
