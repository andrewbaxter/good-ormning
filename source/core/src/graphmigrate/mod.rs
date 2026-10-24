pub mod graph;

use {
    crate::graphmigrate::graph::{
        Graph,
        TopoWalker,
    },
    std::{
        cmp::Reverse,
        collections::{
            BTreeMap,
            BTreeSet,
            BinaryHeap,
            HashMap,
            HashSet,
        },
        fmt::Debug,
        hash::Hash,
    },
};

// Clone is required for initial shuffling - once create/delete/coalesce starts no
// more cloning
pub trait NodeData: Clone {
    type I: Hash + Eq + Clone + Debug + PartialOrd + Ord;
    type N: Hash + Eq + Clone + Debug;
    type O;

    fn apply(ctx: &mut Self::O, ops: &[Op<'_, Self>]);
    fn compare(&self, old: &Self, created: &HashSet<Self::I>) -> Comparison;
    fn create_coalesce(&mut self, other: Self) -> Option<Self>;
    fn delete_coalesce(&mut self, other: Self) -> Option<Self>;
    fn names(&self, old: bool) -> Vec<Self::N>;

    fn ordering(_new: &[(&Self, Option<usize>)], _ops: &[Op<'_, Self>]) -> Vec<(usize, usize)> {
        return vec![];
    }
    fn renamed_from(&self) -> Option<Self::I>;
}

pub enum Comparison {
    DoNothing,
    Recreate,
    Update,
}

pub enum Op<'a, T> {
    Create(&'a T),
    Delete(&'a T),
    Update(OpUpdate<'a, T>),
}

pub struct OpUpdate<'a, T> {
    pub new: &'a T,
    pub old: &'a T,
}

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
            delete_graph_lookup.insert(k.clone(), id);
        }
        for (k, n) in prev_version {
            let gk = *delete_graph_lookup.get(k).unwrap();
            for dep in &n.deps {
                delete_graph.edge(*delete_graph_lookup.get(dep).unwrap(), gk);
            }
        }
    }

    // Create the create/update graph with the new version
    let mut create_graph: Graph<(T::I, Option<DiffNode<T>>)> = Graph::new();
    let mut create_graph_lookup = HashMap::new();
    for k in version.keys() {
        let id = create_graph.add((k.clone(), None));
        create_graph_lookup.insert(k.clone(), id);
    }
    for (k, n) in version {
        let gk = *create_graph_lookup.get(k).unwrap();
        for dep in &n.deps {
            create_graph.edge(*create_graph_lookup.get(dep).unwrap(), gk);
        }
    }

    // Coalesce deletes
    let mut recreated = vec![];
    let mut updated_old = HashMap::new();
    {
        let mut iter = TopoWalker::new_whole_graph(&create_graph);
        let mut created = HashSet::new();
        while let Some(graph_key) = iter.get() {
            iter.enter(&create_graph);
            let k = create_graph.data.get(graph_key).unwrap().0.clone();
            let node = version.get(&k).unwrap();
            let create_graph_key = *create_graph_lookup.get(&k).unwrap();
            let old_k = node.body.renamed_from().unwrap_or_else(|| k.clone());
            if let Some(old_node) = prev_version.as_ref().and_then(|v| v.get(&old_k)) {
                let delete_graph_key = *delete_graph_lookup.get(&old_k).unwrap();
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
                        updated_old.insert(create_graph_key, delete_graph_key);
                    },
                    Comparison::Recreate => {
                        created.insert(k.clone());
                        *create_graph.data.get_mut(create_graph_key).unwrap() =
                            (k, Some(DiffNode::Create { new: node.body.clone() }));
                        recreated.push((delete_graph_key, create_graph_key));
                    },
                }
            } else {
                created.insert(k.clone());
                *create_graph.data.get_mut(create_graph_key).unwrap() =
                    (k, Some(DiffNode::Create { new: node.body.clone() }));
            }
        }
    }
    let mut delete_coalesced = HashMap::new();
    {
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
                match delete_graph.data.get_mut(other_graph_key).unwrap().take() {
                    Some(n) => match root.delete_coalesce(n) {
                        Some(n) => {
                            *delete_graph.data.get_mut(other_graph_key).unwrap() = Some(n);
                            coalesce_iter.skip(&delete_graph);
                        },
                        None => {
                            delete_coalesced.insert(other_graph_key, graph_key);
                            coalesce_iter.enter(&delete_graph);
                        },
                    },
                    None => {
                        // was consumed
                        coalesce_iter.enter(&delete_graph);
                    },
                }
            }
            *delete_graph.data.get_mut(graph_key).unwrap() = Some(root);
        }
    }
    let mut create_coalesced = HashMap::new();
    {
        let mut iter = TopoWalker::new_whole_graph(&create_graph);
        while let Some(graph_key) = iter.get() {
            iter.enter(&create_graph);
            let mut new = match create_graph.data.get_mut(graph_key).unwrap().1.take() {
                Some(DiffNode::Create { new }) => new,
                other => {
                    create_graph.data.get_mut(graph_key).unwrap().1 = other;
                    continue;
                },
            };
            let mut coalesce_iter = TopoWalker::new_rooted_at(&create_graph, graph_key);
            coalesce_iter.enter(&create_graph);
            while let Some(other_graph_key) = coalesce_iter.get() {
                match create_graph.data.get_mut(other_graph_key).unwrap().1.take() {
                    Some(DiffNode::Create { new: other_new }) => match new.create_coalesce(other_new) {
                        Some(other_new) => {
                            create_graph.data.get_mut(other_graph_key).unwrap().1 =
                                Some(DiffNode::Create { new: other_new });
                            coalesce_iter.skip(&create_graph);
                        },
                        None => {
                            create_coalesced.insert(other_graph_key, graph_key);
                            coalesce_iter.enter(&create_graph);
                        },
                    },
                    Some(other) => {
                        create_graph.data.get_mut(other_graph_key).unwrap().1 = Some(other);
                        coalesce_iter.skip(&create_graph);
                    },
                    None => {
                        // was consumed
                        coalesce_iter.enter(&create_graph);
                    },
                }
            }
            create_graph.data.get_mut(graph_key).unwrap().1 = Some(DiffNode::Create { new: new });
        }
    }
    let mut op_nodes: Vec<(Option<usize>, Option<usize>)> = vec![];
    let mut old_op = HashMap::new();
    let mut new_op = HashMap::new();
    {
        let mut delete_walk_graph = Graph::new();
        delete_walk_graph.data = vec![
            ();
            delete_graph.data.len()
        ];
        if let Some(prev_version) = &prev_version {
            for (k, n) in prev_version {
                let gk = *delete_graph_lookup.get(k).unwrap();
                for dep in &n.deps {
                    delete_walk_graph.edge(gk, *delete_graph_lookup.get(dep).unwrap());
                }
            }
        }
        let mut iter = TopoWalker::new_whole_graph(&delete_walk_graph);
        while let Some(graph_key) = iter.get() {
            iter.enter(&delete_walk_graph);
            if delete_graph.data.get(graph_key).unwrap().is_some() {
                old_op.insert(graph_key, op_nodes.len());
                op_nodes.push((Some(graph_key), None));
            }
        }
    }
    {
        let mut iter = TopoWalker::new_whole_graph(&create_graph);
        while let Some(graph_key) = iter.get() {
            iter.enter(&create_graph);
            match &create_graph.data.get(graph_key).unwrap().1 {
                Some(DiffNode::Create { .. }) => {
                    new_op.insert(graph_key, op_nodes.len());
                    op_nodes.push((None, Some(graph_key)));
                },
                Some(DiffNode::Update { .. }) => {
                    let old_graph_key = *updated_old.get(&graph_key).unwrap();
                    old_op.insert(old_graph_key, op_nodes.len());
                    new_op.insert(graph_key, op_nodes.len());
                    op_nodes.push((Some(old_graph_key), Some(graph_key)));
                },
                None => { },
            }
        }
    }
    let delete_coalesced_list = delete_coalesced.into_iter().collect::<BTreeMap<_, _>>();
    let create_coalesced_list = create_coalesced.into_iter().collect::<BTreeMap<_, _>>();
    for (absorbed, root) in &delete_coalesced_list {
        let op = *old_op.get(root).unwrap();
        old_op.insert(*absorbed, op);
    }
    for (absorbed, root) in &create_coalesced_list {
        let op = *new_op.get(root).unwrap();
        new_op.insert(*absorbed, op);
    }
    let ops = op_nodes.iter().map(|(old, new)| match (old, new) {
        (Some(old), None) => Op::Delete(delete_graph.data.get(*old).unwrap().as_ref().unwrap()),
        (_, Some(new)) => match &create_graph.data.get(*new).unwrap().1 {
            Some(DiffNode::Create { new }) => Op::Create(new),
            Some(DiffNode::Update { old, new }) => Op::Update(OpUpdate {
                new: new,
                old: old,
            }),
            None => unreachable!(),
        },
        (None, None) => unreachable!(),
    }).collect::<Vec<_>>();
    let old_nodes = prev_version.iter().flat_map(|v| v.values()).collect::<Vec<_>>();
    let new_nodes = version.values().collect::<Vec<_>>();
    let mut edges = BTreeSet::new();
    for (graph_key, node) in old_nodes.iter().enumerate() {
        let Some(&op) = old_op.get(&graph_key) else {
            continue;
        };
        if op_nodes.get(op).unwrap().1.is_some() {
            continue;
        }
        for dep in &node.deps {
            if let Some(&dep_op) = old_op.get(delete_graph_lookup.get(dep).unwrap()) {
                edges.insert((op, dep_op));
            }
        }
    }
    for (graph_key, node) in new_nodes.iter().enumerate() {
        let Some(&op) = new_op.get(&graph_key) else {
            continue;
        };
        for dep in &node.deps {
            if let Some(&dep_op) = new_op.get(create_graph_lookup.get(dep).unwrap()) {
                edges.insert((dep_op, op));
            }
        }
    }
    for (old, new) in recreated {
        edges.insert((*old_op.get(&old).unwrap(), *new_op.get(&new).unwrap()));
    }
    {
        let mut released: HashMap<T::N, Vec<usize>> = HashMap::new();
        let mut acquired: HashMap<T::N, Vec<usize>> = HashMap::new();
        for (op, (old, new)) in op_nodes.iter().enumerate() {
            let old_names = old.map(|k| old_nodes.get(k).unwrap().body.names(true)).unwrap_or_default();
            let new_names = new.map(|k| new_nodes.get(k).unwrap().body.names(false)).unwrap_or_default();
            for name in &old_names {
                if !new_names.contains(name) {
                    released.entry(name.clone()).or_default().push(op);
                }
            }
            for name in &new_names {
                if !old_names.contains(name) {
                    acquired.entry(name.clone()).or_default().push(op);
                }
            }
        }
        for (graph_key, root) in &delete_coalesced_list {
            let op = *old_op.get(root).unwrap();
            for name in old_nodes.get(*graph_key).unwrap().body.names(true) {
                released.entry(name).or_default().push(op);
            }
        }
        for (graph_key, root) in &create_coalesced_list {
            let op = *new_op.get(root).unwrap();
            for name in new_nodes.get(*graph_key).unwrap().body.names(false) {
                acquired.entry(name).or_default().push(op);
            }
        }
        for (name, release_ops) in &released {
            for acquire_op in acquired.get(name).into_iter().flatten() {
                for release_op in release_ops {
                    edges.insert((*release_op, *acquire_op));
                }
            }
        }
    }
    edges.extend(
        T::ordering(
            &new_nodes
                .iter()
                .enumerate()
                .map(|(graph_key, node)| (&node.body, new_op.get(&graph_key).copied()))
                .collect::<Vec<_>>(),
            &ops,
        ),
    );
    edges.retain(|(a, b)| a != b);
    let mut adjacency = vec![
        vec![];
        ops.len()
    ];
    for (a, b) in &edges {
        adjacency.get_mut(*a).unwrap().push(*b);
    }
    let reachable = (0 .. ops.len()).map(|start| {
        let mut seen = HashSet::new();
        let mut stack = vec![start];
        while let Some(op) = stack.pop() {
            for next in adjacency.get(op).unwrap() {
                if seen.insert(*next) {
                    stack.push(*next);
                }
            }
        }
        return seen;
    }).collect::<Vec<_>>();
    let mut group_of = vec![
        usize::MAX;
        ops.len()
    ];
    let mut groups: Vec<Vec<usize>> = vec![];
    for op in 0 .. ops.len() {
        if *group_of.get(op).unwrap() != usize::MAX {
            continue;
        }
        let members =
            (op .. ops.len())
                .filter(
                    |other| *other == op ||
                        (reachable.get(op).unwrap().contains(other) &&
                            reachable.get(*other).unwrap().contains(&op)),
                )
                .collect::<Vec<_>>();
        for member in &members {
            *group_of.get_mut(*member).unwrap() = groups.len();
        }
        groups.push(members);
    }
    let mut group_edges = vec![
        BTreeSet::new();
        groups.len()
    ];
    let mut group_dep_counts = vec![
        0usize;
        groups.len()
    ];
    for (a, b) in &edges {
        let (a, b) = (*group_of.get(*a).unwrap(), *group_of.get(*b).unwrap());
        if a != b && group_edges.get_mut(a).unwrap().insert(b) {
            *group_dep_counts.get_mut(b).unwrap() += 1;
        }
    }
    let mut ready = BinaryHeap::new();
    for (group, count) in group_dep_counts.iter().enumerate() {
        if *count == 0 {
            ready.push(Reverse((*groups.get(group).unwrap().first().unwrap(), group)));
        }
    }
    while let Some(Reverse((_, group))) = ready.pop() {
        let group_ops = groups.get(group).unwrap().iter().map(|op| match ops.get(*op).unwrap() {
            Op::Create(n) => Op::Create(*n),
            Op::Delete(n) => Op::Delete(*n),
            Op::Update(u) => Op::Update(OpUpdate {
                new: u.new,
                old: u.old,
            }),
        }).collect::<Vec<_>>();
        T::apply(output, &group_ops);
        for next in group_edges.get(group).unwrap() {
            let count = group_dep_counts.get_mut(*next).unwrap();
            *count -= 1;
            if *count == 0 {
                ready.push(Reverse((*groups.get(*next).unwrap().first().unwrap(), *next)));
            }
        }
    }
}

pub struct Node<T: NodeData> {
    pub(crate) body: T,
    deps: Vec<T::I>,
}

impl<T: NodeData> Node<T> {
    pub fn new(deps: Vec<T::I>, body: T) -> Self {
        return Self {
            deps: deps,
            body: body,
        };
    }
}

type Version<T> = BTreeMap<<T as NodeData>::I, Node<T>>;
