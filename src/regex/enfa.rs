use std::fmt;

use super::parsing::{ExprUnit, RegexOp};
use anyhow::{Error, Result};
use petgraph::{dot::Dot, graphmap::GraphMap, Directed};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone, Copy)]
enum ENfaEdge {
    Epsilon,
    Char(char),
}

impl fmt::Debug for ENfaEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ENfaEdge::Epsilon => 'ε',
                ENfaEdge::Char(c) => *c,
            }
        )
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ENfa {
    graph: GraphMap<u32, ENfaEdge, Directed>,
    start: u32,
    fin: Vec<u32>,
}

fn gen_char_nfa(c: char) -> ENfa {
    let mut graph = GraphMap::with_capacity(2, 1);
    let start_node = graph.add_node(0);
    let final_node = graph.add_node(1);
    graph.add_edge(start_node, final_node, ENfaEdge::Char(c));
    ENfa {
        graph,
        start: start_node,
        fin: vec![final_node],
    }
}

fn merge_graphmaps(
    g1: &mut GraphMap<u32, ENfaEdge, Directed>,
    g2: GraphMap<u32, ENfaEdge, Directed>,
) {
    let ord1 = u32::try_from(g1.node_count()).unwrap();
    let fin_ord = u32::try_from(g1.node_count() + g2.node_count()).unwrap();
    for i in ord1..fin_ord {
        g1.add_node(i);
    }
    for (i, j, w) in g2.all_edges() {
        g1.add_edge(ord1 + i, ord1 + j, *w);
    }
}

pub fn union_nfa(mut nfa1: ENfa, mut nfa2: ENfa) -> ENfa {
    let (effort_now, effort_after) = (
        nfa2.graph.node_count() + nfa2.graph.edge_count(),
        nfa1.graph.node_count() + nfa1.graph.edge_count(),
    );
    if effort_after < effort_now {
        (nfa1, nfa2) = (nfa2, nfa1);
    }
    let ord1 = u32::try_from(nfa1.graph.node_count()).unwrap();

    // Merge the two graphs and add a new start node.
    merge_graphmaps(&mut nfa1.graph, nfa2.graph);
    let start_node = u32::try_from(nfa1.graph.node_count()).unwrap();
    nfa1.graph.add_node(start_node);
    let fin_node = u32::try_from(nfa1.graph.node_count()).unwrap();
    nfa1.graph.add_node(fin_node);

    // Fix start node.
    let (start1, start2) = (nfa1.start, nfa2.start);
    nfa1.graph.add_edge(start_node, start1, ENfaEdge::Epsilon);
    nfa1.graph
        .add_edge(start_node, ord1 + start2, ENfaEdge::Epsilon);
    nfa1.start = start_node;

    // Add all end nodes edges.
    for i in nfa1.fin.into_iter() {
        nfa1.graph.add_edge(i, fin_node, ENfaEdge::Epsilon);
    }
    for j in nfa2.fin.into_iter() {
        nfa1.graph.add_edge(ord1 + j, fin_node, ENfaEdge::Epsilon);
    }
    nfa1.fin = vec![fin_node];

    nfa1
}

pub fn concat_nfa(mut nfa1: ENfa, mut nfa2: ENfa) -> ENfa {
    let (effort_now, effort_after) = (
        nfa2.graph.node_count() + nfa2.graph.edge_count() + nfa1.fin.len() + nfa2.fin.len(),
        nfa1.graph.node_count() + nfa1.graph.edge_count() + nfa1.fin.len(),
    );
    let swapped = effort_after < effort_now;
    if swapped {
        let ord2 = u32::try_from(nfa2.graph.node_count()).unwrap();
        merge_graphmaps(&mut nfa2.graph, nfa1.graph);
        for i in nfa1.fin {
            nfa2.graph.add_edge(ord2 + i, nfa2.start, ENfaEdge::Epsilon);
        }
        ENfa {
            graph: nfa2.graph,
            start: ord2 + nfa1.start,
            fin: nfa2.fin,
        }
    } else {
        let ord1 = u32::try_from(nfa1.graph.node_count()).unwrap();
        merge_graphmaps(&mut nfa1.graph, nfa2.graph);
        for i in nfa1.fin {
            nfa1.graph.add_edge(i, ord1 + nfa2.start, ENfaEdge::Epsilon);
        }
        ENfa {
            graph: nfa1.graph,
            start: nfa1.start,
            fin: nfa2.fin.into_iter().map(|x| ord1 + x).collect(),
        }
    }
}

pub fn star_nfa(nfa: &mut ENfa) {
    let start_node = u32::try_from(nfa.graph.node_count()).unwrap();
    nfa.graph.add_node(start_node);
    let fin_node = u32::try_from(nfa.graph.node_count()).unwrap();
    nfa.graph.add_node(fin_node);

    nfa.graph.add_edge(start_node, nfa.start, ENfaEdge::Epsilon);
    nfa.graph.add_edge(start_node, fin_node, ENfaEdge::Epsilon);
    for j in nfa.fin.iter() {
        nfa.graph.add_edge(*j, nfa.start, ENfaEdge::Epsilon);
        nfa.graph.add_edge(*j, fin_node, ENfaEdge::Epsilon);
    }

    nfa.start = start_node;
    nfa.fin = vec![fin_node];
}

pub fn gen_epsilon_nfa_from_expr(expr: &[ExprUnit]) -> Result<ENfa> {
    let mut graph_stk = vec![];

    if expr.is_empty() {
        return Err(Error::msg("Invalid postfix expression: empty expression"));
    }

    for unit in expr {
        match unit {
            ExprUnit::Char(c) => {
                graph_stk.push(gen_char_nfa(*c));
            }
            ExprUnit::Op(RegexOp::Union) => {
                if graph_stk.len() < 2 {
                    return Err(Error::msg(
                        "Invalid postfix expression: not enough operands for union",
                    ));
                }
                let op2 = graph_stk.pop().unwrap();
                let op1 = graph_stk.pop().unwrap();
                graph_stk.push(union_nfa(op1, op2));
            }
            ExprUnit::Op(RegexOp::Concat) => {
                if graph_stk.len() < 2 {
                    return Err(Error::msg(
                        "Invalid postfix expression: not enough operands for concat",
                    ));
                }
                let op2 = graph_stk.pop().unwrap();
                let op1 = graph_stk.pop().unwrap();
                graph_stk.push(concat_nfa(op1, op2));
            }
            ExprUnit::Op(RegexOp::Star) => {
                if graph_stk.len() < 1 {
                    return Err(Error::msg(
                        "Invalid postfix expression: not enough operands for kleene star",
                    ));
                }
                let mut op1 = graph_stk.pop().unwrap();
                star_nfa(&mut op1);
                graph_stk.push(op1);
            }
        }
    }

    if graph_stk.len() != 1 {
        return Err(Error::msg("Invalid postfix expression: too few operands"));
    }
    Ok(graph_stk.pop().unwrap())
}

#[wasm_bindgen]
pub struct FaRep {
    dot_str: String,
    start: u32,
    fin: Vec<u32>,
}

#[wasm_bindgen]
impl FaRep {
    pub fn get_dot_str(&self) -> String {
        String::from(&self.dot_str)
    }

    pub fn get_start(&self) -> u32 {
        self.start
    }

    pub fn get_fin(&self) -> Vec<u32> {
        self.fin.clone()
    }
}

impl ENfa {
    pub fn to_fa_rep(&self) -> FaRep {
        FaRep {
            dot_str: format!("{:?}", Dot::new(&self.graph)),
            start: self.start,
            fin: self.fin.clone(),
        }
    }
}
