use std::collections::HashMap;
use std::iter::FromIterator;

use generational_arena::Arena;
use generational_arena::Index;

use crate::edge::Edge;
use crate::node::Node;

pub struct Graph<N, E> {
    arena: Arena<Node<N, E>>,
}

impl<N, E> Graph<N, E> {
    fn empty() -> Graph<N, E> {
        Graph {
            arena: Arena::new(),
        }
    }

    pub fn from_nodes_and_edge_list(nodes: Vec<N>, edges: Vec<(usize, usize, E)>) -> Graph<N, E> {
        let mut arena = Arena::new();
        let mut label_indices = HashMap::new();
        for (i, label) in nodes.into_iter().enumerate() {
            let index = arena.insert_with(|index| Graph::node_from_index(label, index));
            label_indices.insert(i, index);
        }
        for (index_0, index_1, edge_data) in edges.into_iter() {
            let index_0 = label_indices[&index_0];
            let index_1 = label_indices[&index_1];
            arena.get_mut(index_0).unwrap().edges.push(Edge {
                index: index_1,
                data: edge_data,
            });
        }
        Graph { arena }
    }

    pub fn node_from_index(label: N, index: Index) -> Node<N, E> {
        Node {
            data: label,
            index,
            edges: vec![],
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &N> + '_> {
        Box::new(self.arena.iter().map(|(_, node)| &node.data))
    }

    pub fn iter_nodes(&self) -> Box<dyn Iterator<Item = &Node<N, E>> + '_> {
        Box::new(self.arena.iter().map(|(_, node)| node))
    }

    pub fn iter_edges(&self) -> Box<dyn Iterator<Item = (&N, &N, &E)> + '_> {
        let mut edge_data = vec![];
        for node in self.iter_nodes() {
            for edge in node.edges.iter() {
                edge_data.push((
                    &node.data,
                    &self.arena.get(edge.index).unwrap().data,
                    &edge.data,
                ));
            }
        }
        Box::new(edge_data.into_iter())
    }

    pub fn get(&self, index: Index) -> Option<&Node<N, E>> {
        self.arena.get(index)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut Node<N, E>> {
        self.arena.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }

    fn extend(&mut self, mut graph: Graph<N, E>) {
        let mut old_index_to_new_index: HashMap<Index, Index> = HashMap::new();
        for (old_index, mut node) in graph.arena.drain() {
            let new_index = self.arena.insert_with(|index| {
                node.index = index;
                node
            });
            old_index_to_new_index.insert(old_index, new_index);
        }
        for new_index in old_index_to_new_index.values() {
            for edge in self.arena[*new_index].edges.iter_mut() {
                edge.index = old_index_to_new_index[&edge.index];
            }
        }
    }
}

impl<N, E> FromIterator<Graph<N, E>> for Graph<N, E> {
    fn from_iter<T: IntoIterator<Item = Graph<N, E>>>(iter: T) -> Self {
        let mut graph = Graph::empty();
        for disjoint_subgraph in iter {
            graph.extend(disjoint_subgraph);
        }
        graph
    }
}
