use super::tokens::{RegexTokenizer, Token};
use anyhow::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub enum RegexOp {
    Union,
    Star,
    Concat,
}

#[derive(Debug, Clone, Copy)]
enum OpStkItem {
    Op(RegexOp),
    Parens,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprUnit {
    Char(char),
    Op(RegexOp),
}

fn precedence(op: &RegexOp) -> i32 {
    match op {
        RegexOp::Union => 1,
        RegexOp::Concat => 2,
        RegexOp::Star => 3,
    }
}

fn add_operator(post_expr: &mut Vec<ExprUnit>, op_stk: &mut Vec<OpStkItem>, op: RegexOp) {
    while let Some(OpStkItem::Op(stk_op)) = op_stk.last() {
        if precedence(stk_op) < precedence(&op) {
            break;
        }
        if let Some(OpStkItem::Op(stk_op)) = op_stk.pop() {
            post_expr.push(ExprUnit::Op(stk_op));
        }
    }
    op_stk.push(OpStkItem::Op(op));
}

pub fn tokens_to_postfix(tokens: &mut RegexTokenizer) -> Result<Vec<ExprUnit>> {
    let mut post_expr = vec![];
    let mut op_stk = vec![];

    let mut last_token = false;
    for token in tokens {
        match token {
            Token::Char(c) => {
                if last_token {
                    add_operator(&mut post_expr, &mut op_stk, RegexOp::Concat);
                }
                post_expr.push(ExprUnit::Char(c));
                last_token = true;
            }

            Token::Star => {
                if !last_token {
                    return Err(Error::msg("Invalid expression: star after operator"));
                }
                add_operator(&mut post_expr, &mut op_stk, RegexOp::Star);
                if let Some(OpStkItem::Op(stk_op)) = op_stk.pop() {
                    post_expr.push(ExprUnit::Op(stk_op));
                }
                last_token = true;
            }

            Token::Pipe => {
                if !last_token {
                    return Err(Error::msg("Invalid expression: pipe after operator"));
                }
                add_operator(&mut post_expr, &mut op_stk, RegexOp::Union);
                last_token = false;
            }

            Token::OpenParens => {
                if last_token {
                    add_operator(&mut post_expr, &mut op_stk, RegexOp::Concat);
                }
                op_stk.push(OpStkItem::Parens);
                last_token = false;
            }

            Token::CloseParens => {
                if !last_token {
                    return Err(Error::msg(
                        "Invalid expression: Closed parentheses on incomplete expression",
                    ));
                }
                while let Some(OpStkItem::Op(_)) = op_stk.last() {
                    if let Some(OpStkItem::Op(op)) = op_stk.pop() {
                        post_expr.push(ExprUnit::Op(op));
                    }
                }
                if let Some(OpStkItem::Parens) = op_stk.last() {
                    op_stk.pop();
                } else {
                    return Err(Error::msg(
                        "Invalid expression: Inbalanced closed parentheses",
                    ));
                }
                last_token = true;
            }

            Token::Backslash => {
                if last_token {
                    add_operator(&mut post_expr, &mut op_stk, RegexOp::Concat);
                }
                post_expr.push(ExprUnit::Char('\\'));
                last_token = true;
            }
        }
    }

    while let Some(OpStkItem::Op(_)) = op_stk.last() {
        if let Some(OpStkItem::Op(op)) = op_stk.pop() {
            post_expr.push(ExprUnit::Op(op));
        }
    }
    if !op_stk.is_empty() {
        return Err(Error::msg(
            "Invalid expression: Inbalanced open parantheses",
        ));
    }

    Ok(post_expr)
}
