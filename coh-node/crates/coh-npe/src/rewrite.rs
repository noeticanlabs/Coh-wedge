//! NPE Equivalence and Rewrite Compression
//!
//! Uses `egg` (e-graphs) to find equivalence classes between proposals,
//! allowing the NPE to avoid exploring redundant paths.

#[cfg(feature = "npe-rewrite")]
use {crate::engine::NpeProposal, egg::*, std::fmt::Display};

// A simple mathematical language for NPE proposal equivalence
#[cfg(feature = "npe-rewrite")]
define_language! {
    pub enum MathLang {
        Num(i32),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Symbol(Symbol),
    }
}

#[cfg(feature = "npe-rewrite")]
pub struct NpeRewriter<L: Language + FromOp, N: Analysis<L> = ()> {
    rules: Vec<Rewrite<L, N>>,
    _marker: std::marker::PhantomData<(L, N)>,
}

#[cfg(feature = "npe-rewrite")]
impl<L: Language + Display + FromOp, N: Analysis<L> + Default> NpeRewriter<L, N> {
    pub fn new(rules: Vec<Rewrite<L, N>>) -> Self {
        Self {
            rules,
            _marker: std::marker::PhantomData,
        }
    }

    /// Check if two proposal contents are equivalent using the rewrite rules
    pub fn are_equivalent(&self, content_a: &str, content_b: &str) -> bool {
        // Parse the expressions
        let expr_a: RecExpr<L> = match content_a.parse() {
            Ok(e) => e,
            Err(_) => return false,
        };

        let expr_b: RecExpr<L> = match content_b.parse() {
            Ok(e) => e,
            Err(_) => return false,
        };

        // Create an e-graph and add both expressions
        let mut egraph: EGraph<L, N> = Default::default();
        let id_a = egraph.add_expr(&expr_a);
        let id_b = egraph.add_expr(&expr_b);

        // Run the rules
        let runner = Runner::default().with_egraph(egraph).run(&self.rules);

        // Check if they are in the same e-class
        runner.egraph.find(id_a) == runner.egraph.find(id_b)
    }

    /// Simplify a proposal's content, extracting the minimal representation
    pub fn simplify_proposal(&self, proposal: &NpeProposal) -> Option<String> {
        let expr: RecExpr<L> = proposal.content.parse().ok()?;

        let runner = Runner::default().with_expr(&expr).run(&self.rules);
        let root = runner.roots[0];

        // Extract the best (smallest) expression
        let extractor = Extractor::new(&runner.egraph, AstSize);
        let (_, best_expr) = extractor.find_best(root);

        Some(best_expr.to_string())
    }
}

#[cfg(test)]
#[cfg(feature = "npe-rewrite")]
mod tests {
    use super::*;

    fn create_test_rewriter() -> NpeRewriter<MathLang> {
        let rules = vec![
            rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
            rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
            rewrite!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
            rewrite!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
            rewrite!("add-0"; "(+ ?a 0)" => "?a"),
            rewrite!("mul-0"; "(* ?a 0)" => "0"),
            rewrite!("mul-1"; "(* ?a 1)" => "?a"),
        ];
        NpeRewriter::new(rules)
    }

    #[test]
    fn test_equivalence() {
        let rewriter = create_test_rewriter();

        // (+ a b) == (+ b a)
        assert!(rewriter.are_equivalent("(+ a b)", "(+ b a)"));

        // (+ a 0) == a
        assert!(rewriter.are_equivalent("(+ a 0)", "a"));

        // (* (+ a b) 1) == (+ b a)
        assert!(rewriter.are_equivalent("(* (+ a b) 1)", "(+ b a)"));

        // Not equivalent
        assert!(!rewriter.are_equivalent("(+ a b)", "(* a b)"));
    }

    #[test]
    fn test_simplify() {
        let rewriter = create_test_rewriter();

        let p = NpeProposal {
            id: "1".to_string(),
            content: "(+ (* x 1) 0)".to_string(), // Can be simplified to just "x"
            seed: 0,
            score: 0.0,
            content_hash: "hash".to_string(),
            depth: 0,
            parent_id: None,
            status: crate::engine::ProposalStatus::Generated,
            ..Default::default()
        };

        let simplified = rewriter.simplify_proposal(&p).unwrap();
        assert_eq!(simplified, "x");
    }
}
