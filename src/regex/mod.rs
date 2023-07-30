mod enfa;
mod parsing;
mod tokens;

use anyhow::Context;
use enfa::{gen_epsilon_nfa_from_expr, FaRep};
use parsing::tokens_to_postfix;
use tokens::RegexTokenizer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_debug_postexpr_string(expr: &str) -> String {
    return format!(
        "{:?}",
        tokens_to_postfix(&mut RegexTokenizer::from_string(expr))
    );
}

#[wasm_bindgen]
pub fn get_debug_graph_json(expr: &str) -> Result<FaRep, String> {
    let expr = tokens_to_postfix(&mut RegexTokenizer::from_string(expr))
        .context("Failed to convert to postfix");
    if expr.is_err() {
        return Err(expr.unwrap_err().to_string());
    }
    let expr = expr.unwrap();

    let enfa = gen_epsilon_nfa_from_expr(&expr[..]);
    if enfa.is_err() {
        return Err(enfa.unwrap_err().to_string());
    }
    let enfa = enfa.unwrap();

    let mut nfa = enfa.to_nfa();
    nfa.remove_unreachable_nodes();

    return Ok(nfa.to_fa_rep());
}
