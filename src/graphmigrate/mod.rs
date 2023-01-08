use std::{
    collections::{
        HashMap,
    },
    hash::Hash,
};
use std::fmt::Debug;
use crate::graphmigrate::graph::{
    Graph,
    TopoWalker,
};

pub mod graph;

pub trait NodeId: Hash + PartialEq + Eq + Clone + Debug { }

impl<T: Hash + PartialEq + Eq + Clone + Debug> NodeId for T { }

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
    pub(crate) body: T,
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
>(output: &'a mut T::O, prev_version: &Option<&Version<T, I>>, version: &Version<T, I>) {
    #[derive(Clone, Hash, PartialEq, Eq, Debug)]
    struct VersionNodeId<I: NodeId>(i32, I);

    enum DiffNode<T: NodeData> {
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
        node_ids: HashMap<VersionNodeId<I>, usize>,
        g: Graph<Option<DiffNode<T>>>,
    }

    impl<T: NodeData, I: NodeId> Default for Stage<T, I> {
        fn default() -> Self {
            Stage {
                node_ids: HashMap::new(),
                g: Graph::new(),
            }
        }
    }

    impl<T: NodeData, I: NodeId> Stage<T, I> {
        fn add(&mut self, k: VersionNodeId<I>, v: T) -> usize {
            let id = self.g.add(Some(DiffNode::Create { new: v }));
            self.node_ids.insert(k, id);
            id
        }

        fn remove(&mut self, k: VersionNodeId<I>, v: T) -> usize {
            let id = self.g.add(Some(DiffNode::Delete { old: v }));
            self.node_ids.insert(k, id);
            id
        }

        fn edge(&mut self, a: usize, b: usize) {
            self.g.edge(a, b);
        }

        fn get(&self, v: i32, i: &I) -> Option<usize> {
            self.node_ids.get(&VersionNodeId(v, i.clone())).map(|i| *i)
        }
    }

    let mut stage = Stage::default();
    let prev_version_i = 0;
    let version_i = 1;

    // Initialize graph with previous state, as deletions
    if let Some(prev_version) = prev_version {
        for (k, n) in *prev_version {
            stage.remove(VersionNodeId(prev_version_i, k.clone()), n.body.clone());
        }
        for (k, n) in *prev_version {
            let gk = stage.get(prev_version_i, k).unwrap();
            for dep in &n.deps {
                stage.edge(stage.get(prev_version_i, dep).unwrap(), gk);
            }
        }
    }

    // Add new state to graph, turning delets into donothings/updates where compatible.
    for (k, n) in version {
        let vk = VersionNodeId(version_i, k.clone());
        match stage.get(prev_version_i, k) {
            Some(gk) => {
                let old_n = match stage.g.data.get(gk).unwrap() {
                    Some(DiffNode::Delete { old }) => old,
                    None | Some(DiffNode::Create { .. }) | Some(DiffNode::Update { .. }) => unreachable!(),
                };
                match n.body.compare(old_n) {
                    Comparison::DoNothing => {
                        println!("do nothing due to compare {:?}", vk);
                        stage.g.data[gk] = None;
                        gk
                    },
                    Comparison::Update => {
                        stage.g.data[gk] = Some(DiffNode::Update {
                            old: old_n.clone(),
                            new: n.body.clone(),
                        });
                        gk
                    },
                    Comparison::DeleteCreate => {
                        let new_gk = stage.add(vk, n.body.clone());
                        stage.edge(gk, new_gk);
                        new_gk
                    },
                }
            },
            None => {
                let n = stage.add(vk.clone(), n.body.clone());
                println!("new node {:?}, n {}", vk, n);
                n
            },
        };
    };
    for (k, n) in version {
        let gk = stage.get(version_i, k).unwrap();
        for dep in &n.deps {
            let dep_id = stage.get(version_i, dep).unwrap();
            println!("edge {} {}", dep_id, gk);
            stage.edge(dep_id, gk);
        }
    }

    // Perform changes in order
    let mut iter = TopoWalker::new_whole_graph(&stage.g);
    while let Some(n) = iter.get() {
        println!("walk {}", n);
        match stage.g.data.get_mut(n).unwrap().take() {
            None => {
                println!("-> no node");
            },
            Some(node) => match node {
                DiffNode::Delete { mut old } => {
                    println!("-> delete");
                    let mut coalesce_iter = TopoWalker::new_rooted_at(&stage.g, n);
                    coalesce_iter.enter(&stage.g);
                    while let Some(n) = coalesce_iter.get() {
                        if match stage.g.data.get(n).unwrap().as_ref().unwrap() {
                            DiffNode::Delete { old: v } => old.delete_coalesce(v),
                            _ => {
                                false
                            },
                        } {
                            stage.g.data[n] = None;
                            coalesce_iter.enter(&stage.g);
                            println!("----> coalescing for delete {}", n);
                        } else {
                            coalesce_iter.skip(&stage.g);
                        }
                    }
                    old.delete(output);
                },
                DiffNode::Create { mut new } => {
                    println!("-> create");
                    let mut coalesce_iter = TopoWalker::new_rooted_at(&stage.g, n);
                    println!("   create (zub)");
                    coalesce_iter.enter(&stage.g);
                    while let Some(n) = coalesce_iter.get() {
                        println!("   coalesce {}", n);
                        if match stage.g.data.get(n).unwrap().as_ref() {
                            Some(DiffNode::Create { new: v }) => new.create_coalesce(v),
                            _ => {
                                false
                            },
                        } {
                            stage.g.data[n] = None;
                            coalesce_iter.enter(&stage.g);
                            println!("----> coalescing for create {}", n);
                        } else {
                            coalesce_iter.skip(&stage.g);
                        }
                    }
                    new.create(output);
                },
                DiffNode::Update { old, new } => {
                    println!("-> update");
                    new.update(output, &old);
                },
            },
        }
        iter.enter(&stage.g);
    }
}
