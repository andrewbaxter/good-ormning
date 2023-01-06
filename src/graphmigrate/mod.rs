use std::{
    collections::{
        HashMap,
    },
    hash::Hash,
};
use petgraph::{
    prelude::GraphMap,
    visit::{
        Topo,
        Dfs,
    },
    Directed,
};

pub trait NodeId: Hash + PartialEq + Eq + Clone { }

impl<T: Hash + PartialEq + Eq + Clone> NodeId for T { }

// Clone is required for initial shuffling - once create/delete/coalesce starts no
// more cloning
pub trait NodeData: Clone {
    type O;

    fn compare(&self, old: &Self) -> Comparison;
    fn create_coalesce(&mut self, other: &Self) -> bool;
    fn create(&self, ctx: &mut Self::O);
    fn delete_coalesce(&mut self, other: &Self) -> bool;
    fn delete(&self, ctx: &mut Self::O);
    fn update(&self, ctx: &mut Self::O, old: &Self);
}

pub struct Node<T: NodeData, I: NodeId> {
    deps: Vec<I>,
    body: T,
}

impl<T: NodeData, I: NodeId> Node<T, I> {
    pub fn new(deps: Vec<I>, body: T) -> Self {
        Self {
            deps: deps,
            body: body,
        }
    }
}

pub enum Comparison {
    DoNothing,
    Update,
    DeleteCreate,
}

type Version<T, I> = HashMap<I, Node<T, I>>;

pub fn migrate<
    'a,
    T: NodeData,
    I: NodeId,
>(output: &'a mut T::O, prev_version: &Option<Version<T, I>>, version: &Version<T, I>) {
    #[derive(Clone, Hash, PartialEq, Eq)]
    struct VersionNodeId<I: NodeId>(i32, I);

    enum DiffNode<T: NodeData> {
        DoNothing,
        Create {
            new: T,
        },
        Delete {
            old: T,
        },
        Update {
            old: T,
            new: T,
        },
    }

    struct Stage<T: NodeData, I: NodeId> {
        nodes: Vec<DiffNode<T>>,
        node_ids: HashMap<VersionNodeId<I>, usize>,
        g: GraphMap<usize, usize, Directed>,
        ge: usize,
    }

    impl<T: NodeData, I: NodeId> Default for Stage<T, I> {
        fn default() -> Self {
            Stage {
                nodes: vec![],
                node_ids: HashMap::new(),
                g: GraphMap::new(),
                ge: 0,
            }
        }
    }

    impl<T: NodeData, I: NodeId> Stage<T, I> {
        fn add(&mut self, k: VersionNodeId<I>, v: T) -> usize {
            let id = self.nodes.len();
            self.nodes.push(DiffNode::Create { new: v });
            self.node_ids.insert(k, id);
            self.g.add_node(id);
            id
        }

        fn remove(&mut self, k: VersionNodeId<I>, v: T) -> usize {
            let id = self.nodes.len();
            self.nodes.push(DiffNode::Delete { old: v });
            self.node_ids.insert(k, id);
            self.g.add_node(id);
            id
        }

        fn edge(&mut self, a: usize, b: usize) {
            let id = self.ge;
            self.ge += 1;
            self.g.add_edge(a, b, id);
        }

        fn get(&self, v: i32, i: &I) -> Option<usize> {
            self.node_ids.get(&VersionNodeId(v, i.clone())).map(|i| *i)
        }
    }

    let mut current = Stage::default();
    let prev_version_i = 0;
    let version_i = 1;

    // Initialize graph with previous state, as deletions
    if let Some(prev_version) = prev_version {
        for (k, n) in prev_version {
            current.remove(VersionNodeId(prev_version_i, k.clone()), n.body.clone());
        }
        for (k, n) in prev_version {
            let gk = current.get(prev_version_i, k).unwrap();
            for dep in &n.deps {
                current.edge(current.get(prev_version_i, dep).unwrap(), gk);
            }
        }
    }

    // Add new state to graph, turning delets into donothings/updates where compatible.
    for (k, n) in version {
        let vk = VersionNodeId(version_i, k.clone());
        match current.get(prev_version_i, k) {
            Some(gk) => {
                let old_n = match current.nodes.get(gk).unwrap() {
                    DiffNode::Delete { old } => old,
                    DiffNode::DoNothing | DiffNode::Create { .. } | DiffNode::Update { .. } => unreachable!(),
                };
                match n.body.compare(old_n) {
                    Comparison::DoNothing => {
                        current.nodes[gk] = DiffNode::DoNothing;
                        gk
                    },
                    Comparison::Update => {
                        current.nodes[gk] = DiffNode::Update {
                            old: old_n.clone(),
                            new: n.body.clone(),
                        };
                        gk
                    },
                    Comparison::DeleteCreate => {
                        let new_gk = current.add(vk, n.body.clone());
                        current.edge(gk, new_gk);
                        new_gk
                    },
                }
            },
            None => {
                current.add(vk, n.body.clone())
            },
        };
    };
    for (k, n) in version {
        let gk = current.get(version_i, k).unwrap();
        for dep in &n.deps {
            current.edge(gk, current.get(version_i, dep).unwrap());
        }
    }

    // Perform changes in order
    let mut iter = Topo::new(&current.g);
    while let Some(n) = iter.next(&current.g) {
        match current.nodes.remove(n) {
            DiffNode::DoNothing => { },
            DiffNode::Delete { mut old } => {
                let mut dfs = Dfs::new(&current.g, n);
                dfs.next(&current.g);
                while let Some(n) = dfs.next(&current.g) {
                    if !match current.nodes.get(n).unwrap() {
                        DiffNode::Delete { old: v } => old.delete_coalesce(v),
                        _ => {
                            false
                        },
                    } {
                        dfs.stack.pop();
                        current.nodes[n] = DiffNode::DoNothing;
                    }
                }
                old.delete(output);
            },
            DiffNode::Create { mut new } => {
                let mut dfs = Dfs::new(&current.g, n);
                dfs.next(&current.g);
                while let Some(n) = dfs.next(&current.g) {
                    if !match current.nodes.get(n).unwrap() {
                        DiffNode::Create { new: v } => new.create_coalesce(v),
                        _ => {
                            false
                        },
                    } {
                        dfs.stack.pop();
                        current.nodes[n] = DiffNode::DoNothing;
                    }
                }
                new.create(output);
            },
            DiffNode::Update { old, new } => new.update(output, &old),
        }
    }
}
