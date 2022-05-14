use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
    fmt::Debug,
    io::Write,
};

#[derive(Debug)]
pub struct Graph<T> {
    nodes: BTreeMap<usize, T>,
    node_count: usize,
    edges: BTreeMap<usize, BTreeSet<usize>>,
}
impl<T> Graph<T> {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            node_count: 0,
            edges: BTreeMap::new(),
        }
    }
    pub fn nodes(&self) -> &BTreeMap<usize, T> {
        &self.nodes
    }
}

impl<T> Default for Graph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph<crate::parse::Node> {
    pub fn to_dot(&self, mut output: impl Write) {
        dot::render(self, &mut output).unwrap();
    }
}

impl<T> Graph<T> {
    pub fn push_node(&mut self, node: T) -> usize {
        let index = self.node_count;
        self.nodes.insert(index, node);
        self.edges.insert(index, BTreeSet::new());
        self.node_count += 1;
        index
    }
    pub fn remove_node(&mut self, index: usize) -> Vec<usize> {
        let mut edge_removed = Vec::new();

        self.nodes.remove(&index);
        self.edges.remove(&index);

        for (node_index, e) in &mut self.edges {
            if e.remove(&index) {
                edge_removed.push(*node_index);
            }
        }
        edge_removed
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.get_mut(&from).unwrap().insert(to);
    }
    pub fn remove_all_edge(&mut self, start: usize) -> bool {
        self.edges
            .get_mut(&start)
            .map(|edges| edges.clear())
            .is_some()
    }
    pub fn edges(&self, from: usize) -> &BTreeSet<usize> {
        &self.edges[&from]
    }
    pub fn node(&self, index: usize) -> Option<&T> {
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

pub fn dijkstra<T>(graph: &Graph<T>, start: usize) -> BTreeMap<usize, usize> {
    let mut q = BinaryHeap::from([Reverse((0, start))]);
    let mut costs = BTreeMap::from([(start, 0)]);

    while let Some(Reverse((cost, now_node))) = q.pop() {
        if cost > costs[&now_node] {
            continue;
        }

        for node in graph.edges(now_node) {
            let c = 1 + costs.get(&now_node).copied().unwrap_or_default();

            if *costs.get(node).unwrap_or(&usize::MAX) > c {
                costs.insert(*node, c);
                q.push(Reverse((c, *node)))
            }
        }
    }

    costs
}

type Node = usize;
type Edge = (usize, usize);

impl dot::Labeller<'_, Node, Edge> for Graph<crate::parse::Node> {
    fn graph_id(&self) -> dot::Id<'_> {
        dot::Id::new("example").unwrap()
    }

    fn node_id(&self, n: &Node) -> dot::Id<'_> {
        let node = &self.nodes[&*n];
        let s = match node {
            crate::parse::Node::Loop(_) => format!("Loop{n}"),
            crate::parse::Node::Instruction(_) => format!("Ins{n}"),
        };
        dot::Id::new(s).unwrap()
    }
    fn node_label(&self, n: &Node) -> dot::LabelText<'_> {
        let node = &self.nodes[&*n];
        let s = match node {
            crate::parse::Node::Loop(_) => format!("Loop{n}"),
            crate::parse::Node::Instruction(ins) => ins.to_string(),
        };
        dot::LabelText::LabelStr(s.into())
    }
}

impl<T: Debug> dot::GraphWalk<'_, Node, Edge> for Graph<T> {
    fn nodes(&self) -> dot::Nodes<Node> {
        self.nodes.keys().copied().collect()
    }

    fn edges(&self) -> dot::Edges<Edge> {
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
