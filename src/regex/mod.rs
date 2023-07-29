mod parsing;
mod tokens;

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
