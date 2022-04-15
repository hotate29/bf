use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    io::Write,
};

#[derive(Debug)]
pub struct Graph<'a, T> {
    nodes: BTreeMap<usize, &'a T>,
    node_count: usize,
    edges: BTreeMap<usize, BTreeSet<usize>>,
}
impl<T> Graph<'_, T> {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            node_count: 0,
            edges: BTreeMap::new(),
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
        self.nodes.insert(self.node_count, node);
        self.edges.insert(self.node_count, BTreeSet::new());
        self.node_count += 1;
    }
    pub fn remove_node(&mut self, index: usize) {
        self.nodes.remove(&index);
        self.edges.remove(&index);

        for e in self.edges.values_mut() {
            e.remove(&index);
        }
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.get_mut(&from).unwrap().insert(to);
    }
    pub fn edges(&self, from: usize) -> &BTreeSet<usize> {
        &self.edges[&from]
    }
    pub fn node(&self, index: usize) -> Option<&&T> {
        self.nodes.get(&index)
    }
    pub fn indegree(&self) -> BTreeMap<usize, usize> {
        let mut indegree = BTreeMap::new();

        for node in self.nodes.keys() {
            indegree.insert(*node, 0);
        }

        for to in self.edges.values().flatten() {
            *indegree.get_mut(to).unwrap() += 1;
        }

        indegree
    }
}

type Node = usize;
type Edge = (usize, usize);

impl dot::Labeller<'_, Node, Edge> for Graph<'_, crate::parse::Node> {
    fn graph_id(&self) -> dot::Id<'_> {
        dot::Id::new("example").unwrap()
    }

    fn node_id(&self, n: &Node) -> dot::Id<'_> {
        let node = self.nodes[&*n];
        let s = match node {
            crate::parse::Node::Loop(_) => format!("Loop{n}"),
            crate::parse::Node::Instruction(_) => format!("Ins{n}"),
        };
        dot::Id::new(s).unwrap()
    }
    fn node_label(&self, n: &Node) -> dot::LabelText<'_> {
        let node = self.nodes[&*n];
        let s = match node {
            crate::parse::Node::Loop(_) => format!("Loop{n}"),
            crate::parse::Node::Instruction(ins) => ins.to_string(),
        };
        dot::LabelText::LabelStr(s.into())
    }
}

impl<T: Debug> dot::GraphWalk<'_, Node, Edge> for Graph<'_, T> {
    fn nodes(&self) -> dot::Nodes<'_, Node> {
        self.nodes.keys().copied().collect()
    }

    fn edges(&self) -> dot::Edges<'_, Edge> {
        self.edges
            .iter()
            .flat_map(|(from, tos)| tos.iter().map(move |to| (*from, *to)))
            .collect()
    }

    fn source(&self, edge: &Edge) -> Node {
        edge.0
    }

    fn target(&self, edge: &Edge) -> Node {
        edge.1
    }
}
