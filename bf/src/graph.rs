use std::{fmt::Debug, io::Write};

#[derive(Debug)]
pub struct Graph<'a, T> {
    nodes: Vec<&'a T>,
    edges: Vec<Vec<usize>>,
}
impl<T> Graph<'_, T> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
        }
    }
}

impl<T> Default for Graph<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph<'_, crate::parse::Node> {
    pub fn to_dot(&self, mut output: impl Write) {
        dot::render(self, &mut output).unwrap();
    }
}

impl<'a, T> Graph<'a, T> {
    pub fn push_node(&mut self, node: &'a T) {
        self.nodes.push(node);
        self.edges.push(vec![])
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges[from].push(to);
    }
    pub fn edges(&self, from: usize) -> &Vec<usize> {
        &self.edges[from]
    }
    pub fn node(&self, index: usize) -> &T {
        self.nodes[index]
    }
}

type Node = usize;
type Edge = (usize, usize);

impl dot::Labeller<'_, Node, Edge> for Graph<'_, crate::parse::Node> {
    fn graph_id(&self) -> dot::Id<'_> {
        dot::Id::new("example").unwrap()
    }

    fn node_id(&self, n: &Node) -> dot::Id<'_> {
        let node = self.nodes[*n];
        let s = match node {
            crate::parse::Node::Loop(_) => format!("Loop{n}"),
            crate::parse::Node::Instruction(_) => format!("Ins{n}"),
        };
        dot::Id::new(s).unwrap()
    }
    fn node_label(&self, n: &Node) -> dot::LabelText<'_> {
        let node = self.nodes[*n];
        let s = match node {
            crate::parse::Node::Loop(_) => format!("Loop{n}"),
            crate::parse::Node::Instruction(ins) => ins.to_string(),
        };
        dot::LabelText::LabelStr(s.into())
    }
}

impl<T: Debug> dot::GraphWalk<'_, Node, Edge> for Graph<'_, T> {
    fn nodes(&self) -> dot::Nodes<'_, Node> {
        (0..self.nodes.len()).collect()
    }

    fn edges(&self) -> dot::Edges<'_, Edge> {
        self.edges
            .iter()
            .enumerate()
            .flat_map(|(from, tos)| tos.iter().map(move |to| (from, *to)))
            .collect()
    }

    fn source(&self, edge: &Edge) -> Node {
        edge.0
    }

    fn target(&self, edge: &Edge) -> Node {
        edge.1
    }
}
