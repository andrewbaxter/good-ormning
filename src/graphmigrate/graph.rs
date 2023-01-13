use std::collections::{
    HashMap,
    HashSet,
};

pub struct Graph<T> {
    pub data: Vec<T>,
    edges: HashMap<usize, Vec<usize>>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Graph {
            data: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add(&mut self, v: T) -> usize {
        let id = self.data.len();
        self.data.push(v);
        id
    }

    pub fn edge(&mut self, a: usize, b: usize) {
        match self.edges.entry(a) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.get_mut().push(b);
            },
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(vec![b]);
            },
        };
    }

    pub fn reverse_edges(&mut self) {
        let mut replace = HashMap::new();
        replace.reserve(self.edges.len());
        for (k, v) in self.edges.drain() {
            replace.insert(k, v);
        }
        self.edges = replace;
    }
}

/// Visit each node only when all nodes pointing to it have been visited
pub struct TopoWalker {
    dep_counts: HashMap<usize, usize>,
    stack: Vec<usize>,
}

impl TopoWalker {
    pub fn new_whole_graph<T>(g: &Graph<T>) -> TopoWalker {
        let mut dep_counts = HashMap::new();
        for n in 0 .. g.data.len() {
            for dest in g.edges.get(&n).iter().map(|x| *x).flatten() {
                *dep_counts.entry(*dest).or_insert(0usize) += 1;
            }
        }
        let mut stack = vec![];
        for n in 0 .. g.data.len() {
            if dep_counts.get(&n).map(|x| *x).unwrap_or_default() == 0 {
                stack.push(n);
            }
        }
        TopoWalker {
            dep_counts: dep_counts,
            stack: stack,
        }
    }

    pub fn new_rooted_at<T>(g: &Graph<T>, n: usize) -> TopoWalker {
        let mut dfs_stack = vec![n];
        let mut dfs_seen = HashSet::new();
        let mut dep_counts = HashMap::new();
        while let Some(source) = dfs_stack.pop() {
            if dfs_seen.contains(&source) {
                continue;
            }
            dfs_seen.insert(source);
            for dest in g.edges.get(&source).iter().map(|x| *x).flatten() {
                dfs_stack.push(*dest);
                *dep_counts.entry(*dest).or_insert(0usize) += 1;
            }
        }
        TopoWalker {
            dep_counts: dep_counts,
            stack: vec![n],
        }
    }

    pub fn get(&self) -> Option<usize> {
        self.stack.last().map(|x| *x)
    }

    pub fn skip<T>(&mut self, g: &Graph<T>) {
        let source = self.stack.pop().unwrap();
        for dest in g.edges.get(&source).iter().map(|x| *x).flatten() {
            *self.dep_counts.get_mut(dest).unwrap() -= 1;
        }
    }

    pub fn enter<T>(&mut self, g: &Graph<T>) {
        let source = self.stack.pop().unwrap();
        for dest in g.edges.get(&source).iter().map(|x| *x).flatten() {
            let dep_count = self.dep_counts.get_mut(dest).unwrap();
            *dep_count -= 1;
            if *dep_count == 0 {
                self.stack.push(*dest);
            }
        }
    }
}
