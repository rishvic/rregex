use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
};

use super::parsing::{ExprUnit, RegexOp};
use anyhow::{Error, Result};
use petgraph::{
    algo::tarjan_scc,
    dot::Dot,
    graphmap::{DiGraphMap, GraphMap},
    visit::Dfs,
    Directed,
};
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

impl TryFrom<ENfaEdge> for char {
    type Error = ENfaEdge;

    fn try_from(other: ENfaEdge) -> Result<Self, Self::Error> {
        match other {
            ENfaEdge::Char(c) => Ok(c),
            v => Err(v),
        }
    }
}

impl PartialEq<ENfaEdge> for ENfaEdge {
    fn eq(&self, other: &ENfaEdge) -> bool {
        match self {
            ENfaEdge::Epsilon => match other {
                ENfaEdge::Epsilon => true,
                ENfaEdge::Char(_) => false,
            },
            ENfaEdge::Char(c1) => match other {
                ENfaEdge::Epsilon => false,
                ENfaEdge::Char(c2) => c1.eq(c2),
            },
        }
    }
}

impl Eq for ENfaEdge {}

impl PartialOrd<ENfaEdge> for ENfaEdge {
    fn partial_cmp(&self, other: &ENfaEdge) -> Option<Ordering> {
        match self {
            ENfaEdge::Epsilon => match other {
                ENfaEdge::Epsilon => Some(Ordering::Equal),
                ENfaEdge::Char(_) => Some(Ordering::Less),
            },
            ENfaEdge::Char(c1) => match other {
                ENfaEdge::Epsilon => Some(Ordering::Greater),
                ENfaEdge::Char(c2) => c1.partial_cmp(c2),
            },
        }
    }
}

impl Ord for ENfaEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            ENfaEdge::Epsilon => match other {
                ENfaEdge::Epsilon => Ordering::Equal,
                ENfaEdge::Char(_) => Ordering::Less,
            },
            ENfaEdge::Char(c1) => match other {
                ENfaEdge::Epsilon => Ordering::Greater,
                ENfaEdge::Char(c2) => c1.cmp(c2),
            },
        }
    }
}

type ENfaGraph = GraphMap<u32, BTreeSet<ENfaEdge>, Directed>;

#[derive(Serialize, Debug, Clone)]
#[wasm_bindgen]
pub struct ENfa {
    graph: ENfaGraph,
    start: u32,
    fin: Vec<u32>,
}

impl ENfa {
    fn add_edge(&mut self, i: u32, j: u32, w: ENfaEdge) {
        if let Some(transition_chars) = self.graph.edge_weight_mut(i, j) {
            transition_chars.insert(w);
        } else {
            let mut edge_map = BTreeSet::new();
            edge_map.insert(w);
            self.graph.add_edge(i, j, edge_map);
        }
    }
}

fn gen_char_nfa(c: char) -> ENfa {
    let mut graph = GraphMap::with_capacity(2, 1);
    let start_node = graph.add_node(0);
    let final_node = graph.add_node(1);
    let mut edge_map = BTreeSet::new();
    edge_map.insert(ENfaEdge::Char(c));
    graph.add_edge(start_node, final_node, edge_map);
    ENfa {
        graph,
        start: start_node,
        fin: vec![final_node],
    }
}

fn merge_graphmaps(g1: &mut ENfaGraph, g2: ENfaGraph) {
    let ord1 = u32::try_from(g1.node_count()).unwrap();
    let fin_ord = u32::try_from(g1.node_count() + g2.node_count()).unwrap();
    for i in ord1..fin_ord {
        g1.add_node(i);
    }
    for (i, j, w) in g2.all_edges() {
        g1.add_edge(ord1 + i, ord1 + j, w.clone());
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
    nfa1.add_edge(start_node, start1, ENfaEdge::Epsilon);
    nfa1.add_edge(start_node, ord1 + start2, ENfaEdge::Epsilon);
    nfa1.start = start_node;

    // Add all end nodes edges.
    for i in nfa1.fin.clone() {
        nfa1.add_edge(i, fin_node, ENfaEdge::Epsilon);
    }
    for j in nfa2.fin {
        nfa1.add_edge(ord1 + j, fin_node, ENfaEdge::Epsilon);
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
            nfa2.add_edge(ord2 + i, nfa2.start, ENfaEdge::Epsilon);
        }
        ENfa {
            graph: nfa2.graph,
            start: ord2 + nfa1.start,
            fin: nfa2.fin,
        }
    } else {
        let ord1 = u32::try_from(nfa1.graph.node_count()).unwrap();
        merge_graphmaps(&mut nfa1.graph, nfa2.graph);
        for i in nfa1.fin.clone() {
            nfa1.add_edge(i, ord1 + nfa2.start, ENfaEdge::Epsilon);
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

    nfa.add_edge(start_node, nfa.start, ENfaEdge::Epsilon);
    nfa.add_edge(start_node, fin_node, ENfaEdge::Epsilon);
    for j in nfa.fin.clone().iter() {
        nfa.add_edge(*j, nfa.start, ENfaEdge::Epsilon);
        nfa.add_edge(*j, fin_node, ENfaEdge::Epsilon);
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

pub type NfaIx = u32;
pub type NfaGraph = DiGraphMap<NfaIx, BTreeSet<char>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct Nfa {
    graph: NfaGraph,
    start: NfaIx,
    fin: Vec<NfaIx>,
}

#[wasm_bindgen]
impl ENfa {
    pub fn to_fa_rep(&self) -> FaRep {
        FaRep {
            dot_str: format!("{:?}", Dot::new(&self.graph)),
            start: self.start,
            fin: self.fin.clone(),
        }
    }
}

impl ENfa {
    pub fn to_nfa(&self) -> Nfa {
        let mut epsilon_graph = DiGraphMap::new();
        for node in self.graph.nodes() {
            epsilon_graph.add_node(node);
        }
        self.graph
            .all_edges()
            .filter(|(_, _, w)| w.contains(&ENfaEdge::Epsilon))
            .for_each(|(i, j, _)| {
                epsilon_graph.add_edge(i, j, ());
            });

        let components = tarjan_scc(&epsilon_graph);
        let mut graph = NfaGraph::new();
        let mut id_to_comp = Vec::with_capacity(epsilon_graph.node_count());
        id_to_comp.resize(epsilon_graph.node_count(), 0u32);
        for (i, component) in components.iter().enumerate() {
            graph.add_node(u32::try_from(i).unwrap());
            for v in component {
                id_to_comp[usize::try_from(*v).unwrap()] = u32::try_from(i).unwrap();
            }
        }

        let mut comp_epsilon_graph: DiGraphMap<u32, ()> = DiGraphMap::new();
        for (i, j, w) in self.graph.all_edges() {
            let (comp1, comp2) = (
                id_to_comp[usize::try_from(i).unwrap()],
                id_to_comp[usize::try_from(j).unwrap()],
            );
            if comp1 == comp2 {
                continue;
            }

            for elem in w {
                match elem {
                    ENfaEdge::Char(c) => {
                        if let Some(transition_chars) = graph.edge_weight_mut(comp1, comp2) {
                            transition_chars.insert(*c);
                        } else {
                            let mut transition_chars = BTreeSet::new();
                            transition_chars.insert(*c);
                            graph.add_edge(comp1, comp2, transition_chars);
                        }
                    }
                    ENfaEdge::Epsilon => {
                        comp_epsilon_graph.add_edge(comp1, comp2, ());
                    }
                }
            }
        }

        let mut fin_comps = BTreeSet::new();
        for v in &self.fin {
            fin_comps.insert(id_to_comp[usize::try_from(*v).unwrap()]);
        }

        let mut graph_copy = graph.clone();
        for u in 0..u32::try_from(components.len()).unwrap() {
            for (_, v, _) in comp_epsilon_graph.edges(u) {
                if fin_comps.contains(&v) {
                    fin_comps.insert(u);
                }
                for (_, w, m) in graph_copy.edges(v) {
                    if let Some(transition_chars) = graph.edge_weight_mut(u, w) {
                        transition_chars.append(&mut m.clone());
                    } else {
                        graph.add_edge(u, w, m.clone());
                    }
                }
            }

            for (_, v, w) in graph.edges(u) {
                graph_copy.add_edge(u, v, w.clone());
            }
        }

        Nfa {
            graph,
            start: id_to_comp[usize::try_from(self.start).unwrap()],
            fin: fin_comps.into_iter().collect(),
        }
    }
}

fn merge_char_u32map_maps(a: &mut BTreeMap<char, BTreeSet<u32>>, b: BTreeMap<char, BTreeSet<u32>>) {
    for (c, b_ids) in b {
        match a.get_mut(&c) {
            None => {
                a.insert(c, b_ids);
            }
            Some(a_ids) => {
                for b_id in b_ids {
                    a_ids.insert(b_id);
                }
            }
        }
    }
}

#[wasm_bindgen]
impl Nfa {
    pub fn to_fa_rep(&self) -> FaRep {
        FaRep {
            dot_str: format!("{:?}", Dot::new(&self.graph)),
            start: self.start,
            fin: self.fin.clone(),
        }
    }
}

impl Nfa {
    pub fn remove_unreachable_nodes(&mut self) {
        let mut reachable_nodes = BTreeSet::new();
        let mut dfs = Dfs::new(&self.graph, self.start);
        while let Some(v) = dfs.next(&self.graph) {
            reachable_nodes.insert(v);
        }

        let mut reachable_map = vec![];
        reachable_map.resize(self.graph.node_count(), 0);

        for v in 0..u32::try_from(self.graph.node_count()).unwrap() {
            if !reachable_nodes.contains(&v) {
                self.graph.remove_node(v);
            }
        }

        for (i, v) in reachable_nodes.iter().enumerate() {
            reachable_map[usize::try_from(*v).unwrap()] = u32::try_from(i).unwrap();
        }

        let mut graph = NfaGraph::new();
        for v in 0..u32::try_from(reachable_nodes.len()).unwrap() {
            graph.add_node(v);
        }
        for (u, v, w) in self.graph.all_edges() {
            let (new_u, new_v) = (
                reachable_map[usize::try_from(u).unwrap()],
                reachable_map[usize::try_from(v).unwrap()],
            );
            graph.add_edge(new_u, new_v, w.clone());
        }

        self.graph = graph;
        self.start = reachable_map[usize::try_from(self.start).unwrap()];
        self.fin = self
            .fin
            .clone()
            .into_iter()
            .filter(|v| reachable_nodes.contains(v))
            .map(|v| reachable_map[usize::try_from(v).unwrap()])
            .collect();
    }

    pub fn reverse(self) -> Self {
        let mut graph = NfaGraph::new();
        for v in self.graph.nodes() {
            graph.add_node(v);
        }
        for (u, v, w) in self.graph.all_edges() {
            graph.add_edge(v, u, w.clone());
        }

        if self.fin.len() > 1 {
            let graph_copy = graph.clone();
            let new_node: u32 = u32::try_from(graph.node_count()).unwrap();
            graph.add_node(new_node);

            let mut fin = vec![self.start];
            if self.fin.contains(&self.start) {
                fin.push(new_node);
            }
            for u in self.fin.iter() {
                for (_, v, w) in graph_copy.edges(*u) {
                    if let Some(transition_chars) = graph.edge_weight_mut(new_node, v) {
                        for new_char in w {
                            transition_chars.insert(*new_char);
                        }
                    } else {
                        graph.add_edge(new_node, v, w.clone());
                    }
                }
            }

            Nfa {
                graph,
                start: new_node,
                fin,
            }
        } else {
            Nfa {
                graph,
                start: self.fin[0],
                fin: vec![self.start],
            }
        }
    }

    pub fn subset_construction(self) -> Self {
        let mut subset_to_id: BTreeMap<BTreeSet<u32>, usize> = BTreeMap::new();
        let mut id_to_next: Vec<BTreeMap<char, BTreeSet<u32>>> = vec![];
        let mut singular_next: Vec<BTreeMap<char, BTreeSet<u32>>> = vec![];
        singular_next.resize_with(self.graph.node_count(), || BTreeMap::new());

        for (u, v, map) in self.graph.all_edges() {
            let u = usize::try_from(u).unwrap();
            for c in map {
                if let Some(singular_subset) = singular_next[u].get_mut(c) {
                    singular_subset.insert(v);
                } else {
                    let mut singular_subset = BTreeSet::new();
                    singular_subset.insert(v);
                    singular_next[u].insert(*c, singular_subset);
                }
            }
        }

        let mut fin = vec![];

        let mut que = VecDeque::new();
        let mut initial_state = BTreeSet::new();
        initial_state.insert(self.start);
        que.push_back(initial_state);
        while !que.is_empty() {
            let subset = que.pop_front().unwrap();
            if subset_to_id.contains_key(&subset) {
                continue;
            }

            let mut moves = BTreeMap::new();
            for neigh_id in &subset {
                merge_char_u32map_maps(
                    &mut moves,
                    singular_next[usize::try_from(*neigh_id).unwrap()].clone(),
                )
            }
            for (_, next_subset) in &moves {
                que.push_back(next_subset.clone());
            }

            let id = subset_to_id.len();
            for fin_node in &self.fin {
                if subset.contains(fin_node) {
                    fin.push(u32::try_from(id).unwrap());
                    break;
                }
            }
            subset_to_id.insert(subset, id);
            id_to_next.push(moves);
        }

        let mut graph = NfaGraph::new();

        for (id, next_subsets) in id_to_next.iter().enumerate() {
            let id = u32::try_from(id).unwrap();
            graph.add_node(id);
            for (c, next_subset) in next_subsets {
                let next_id = u32::try_from(subset_to_id[next_subset]).unwrap();
                if let Some(transition_chars) = graph.edge_weight_mut(id, next_id) {
                    transition_chars.insert(*c);
                } else {
                    let mut transition_chars = BTreeSet::new();
                    transition_chars.insert(*c);
                    graph.add_edge(id, next_id, transition_chars);
                }
            }
        }

        Nfa {
            graph,
            start: 0,
            fin,
        }
    }

    pub fn minimized_dfa(self) -> Self {
        self.reverse()
            .subset_construction()
            .reverse()
            .subset_construction()
    }
}
