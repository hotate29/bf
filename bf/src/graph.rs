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
