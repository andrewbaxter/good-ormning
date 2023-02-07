use std::{
    collections::{
        HashMap,
        HashSet,
        BTreeMap,
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
    type I: Hash + Eq + Clone + Debug + PartialOrd + Ord;

    fn compare(&self, old: &Self, created: &HashSet<Self::I>) -> Comparison;
    fn create_coalesce(&mut self, other: Self) -> Option<Self>;
    fn create(&self, ctx: &mut Self::O);
    fn delete_coalesce(&mut self, other: Self) -> Option<Self>;
    fn delete(&self, ctx: &mut Self::O);
    fn update(&self, ctx: &mut Self::O, old: &Self);
}

pub struct Node<T: NodeData> {
    deps: Vec<T::I>,
    pub(crate) body: T,
}

impl<T: NodeData> Node<T> {
    pub fn new(deps: Vec<T::I>, body: T) -> Self {
        Self {
            deps: deps,
            body: body,
        }
    }
}

pub enum Comparison {
    DoNothing,
    Update,
    Recreate,
}

type Version<T> = BTreeMap<<T as NodeData>::I, Node<T>>;

pub fn migrate<T: NodeData>(output: &mut T::O, prev_version: Option<Version<T>>, version: &Version<T>) {
    enum DiffNode<T: NodeData> {
        Create {
            new: T,
        },
        Update {
            old: T,
            new: T,
        },
    }

    // Create a graph of deletions based on previous version
    let mut delete_graph = Graph::new();
    let mut delete_graph_lookup = HashMap::new();
    if let Some(prev_version) = &prev_version {
        for (k, n) in prev_version {
            let id = delete_graph.add(Some(n.body.clone()));
            delete_graph_lookup.insert(k, id);
        }
        for (k, n) in prev_version {
            let gk = *delete_graph_lookup.get(k).unwrap();
            for dep in &n.deps {
                delete_graph.edge(*delete_graph_lookup.get(dep).unwrap(), gk);
            }
        }
    }

    // Create the create/update graph with the new version
    let mut create_graph = Graph::new();
    let mut create_graph_lookup = HashMap::new();
    for (k, _) in version {
        let id = create_graph.add((k.clone(), None));
        create_graph_lookup.insert(k, id);
    }
    for (k, n) in version {
        let gk = *create_graph_lookup.get(k).unwrap();
        for dep in &n.deps {
            create_graph.edge(*create_graph_lookup.get(dep).unwrap(), gk);
        }
    }

    // Walk the create graph, negating creates/deletes based on detected changes
    {
        let mut iter = TopoWalker::new_whole_graph(&create_graph);
        let mut created = HashSet::new();
        while let Some(graph_key) = iter.get() {
            iter.enter(&create_graph);
            let k = create_graph.data.get(graph_key).unwrap().0.clone();
            let node = version.get(&k).unwrap();
            let create_graph_key = *create_graph_lookup.get(&k).unwrap();
            if let Some(old_node) = prev_version.as_ref().and_then(|v| v.get(&k)) {
                let delete_graph_key = *delete_graph_lookup.get(&k).unwrap();
                match node.body.compare(&old_node.body, &created) {
                    Comparison::DoNothing => {
                        *delete_graph.data.get_mut(delete_graph_key).unwrap() = None;
                    },
                    Comparison::Update => {
                        let old_data = delete_graph.data.get_mut(delete_graph_key).unwrap().take().unwrap();
                        *create_graph.data.get_mut(create_graph_key).unwrap() = (k, Some(DiffNode::Update {
                            new: node.body.clone(),
                            old: old_data,
                        }));
                    },
                    Comparison::Recreate => {
                        created.insert(k.clone());
                        *create_graph.data.get_mut(create_graph_key).unwrap() =
                            (k, Some(DiffNode::Create { new: node.body.clone() }));
                    },
                }
            } else {
                created.insert(k.clone());
                *create_graph.data.get_mut(create_graph_key).unwrap() =
                    (k, Some(DiffNode::Create { new: node.body.clone() }));
            }
        }
    }

    // Handle deletes
    {
        // Coalesce deletes
        let mut iter = TopoWalker::new_whole_graph(&delete_graph);
        while let Some(graph_key) = iter.get() {
            iter.enter(&delete_graph);
            let mut root = match delete_graph.data.get_mut(graph_key).unwrap().take() {
                None => continue,
                Some(r) => r,
            };
            let mut coalesce_iter = TopoWalker::new_rooted_at(&delete_graph, graph_key);
            coalesce_iter.enter(&delete_graph);
            while let Some(other_graph_key) = coalesce_iter.get() {
                let unconsumed = match delete_graph.data.get_mut(other_graph_key).unwrap().take() {
                    Some(n) => {
                        root.delete_coalesce(n).map(|n| Some(n))
                    },
                    None => None,
                };
                match unconsumed {
                    Some(n) => {
                        // wasn't consumed; replace and skip tree
                        *delete_graph.data.get_mut(other_graph_key).unwrap() = n;
                        coalesce_iter.skip(&delete_graph);
                    },
                    None => {
                        // was consumed
                        coalesce_iter.enter(&delete_graph);
                    },
                }
            }
            *delete_graph.data.get_mut(graph_key).unwrap() = Some(root);
        }

        // Reverse edges so leaves come first in walk
        delete_graph.reverse_edges();

        // Do delete generation
        let mut iter = TopoWalker::new_whole_graph(&delete_graph);
        while let Some(graph_key) = iter.get() {
            iter.enter(&delete_graph);
            let n = match delete_graph.data.get(graph_key).unwrap() {
                Some(n) => n,
                None => continue,
            };
            n.delete(output);
        }
    }

    // Handle creates
    {
        // Coalesce creates and generate
        let mut iter = TopoWalker::new_whole_graph(&create_graph);
        while let Some(graph_key) = iter.get() {
            iter.enter(&create_graph);
            match create_graph.data.get_mut(graph_key).unwrap().1.take() {
                None => continue,
                Some(r) => match r {
                    DiffNode::Create { mut new } => {
                        let mut coalesce_iter = TopoWalker::new_rooted_at(&create_graph, graph_key);
                        coalesce_iter.enter(&create_graph);
                        while let Some(other_graph_key) = coalesce_iter.get() {
                            let (other_k, other_node) = {
                                let (other_k, other_node) = create_graph.data.get_mut(other_graph_key).unwrap();
                                (other_k.clone(), other_node.take())
                            };
                            let unconsumed = match other_node {
                                Some(other_node) => match other_node {
                                    DiffNode::Create { new: other_new } => {
                                        new.create_coalesce(other_new).map(|n| DiffNode::Create { new: n })
                                    },
                                    other_node => Some(other_node),
                                },
                                None => None,
                            };
                            match unconsumed {
                                Some(other_node) => {
                                    // wasn't consumed; replace and skip tree
                                    *create_graph.data.get_mut(other_graph_key).unwrap() = (other_k, Some(other_node));
                                    coalesce_iter.skip(&create_graph);
                                },
                                None => {
                                    // was consumed
                                    coalesce_iter.enter(&create_graph);
                                },
                            }
                        }
                        new.create(output);
                    },
                    DiffNode::Update { old, new } => {
                        new.update(output, &old);
                    },
                },
            };
        }
    }
}
