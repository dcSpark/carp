use std::{borrow::Cow, collections::BTreeMap};

use dot::LabelText;
use tasks::{
    dsl::database_task::TaskRegistryEntry, execution_plan::ExecutionPlan,
    utils::find_task_registry_entry,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, strum_macros::Display)]
enum SubgraphNames {
    Genesis,
    Byron,
    Multiera,
}

pub fn generate(exec_plan: &ExecutionPlan, plan_name: &str) -> Graph {
    let mut nodes = Vec::<String>::default();
    let mut edges = Vec::<(usize, usize)>::default();
    let mut subgraphs = BTreeMap::<SubgraphNames, Vec<NodeIdentifier>>::default();

    let mut add_node = |subgraph: SubgraphNames,
                        task_name: &str,
                        entry_name: &str,
                        entry_dependencies: &[&str]| {
        if entry_name == task_name {
            let own_id = nodes.len();
            nodes.push(entry_name.to_string());
            subgraphs
                .entry(subgraph)
                .and_modify(|nodes| nodes.push(own_id))
                .or_insert_with(|| vec![own_id]);
            for dep in entry_dependencies {
                let dep_position = nodes.iter().position(|task| task == dep).unwrap();
                edges.push((dep_position, own_id));
            }
        }
    };
    for (task_name, val) in exec_plan.0.iter() {
        if let toml::value::Value::Table(_task_props) = val {
            let entry = find_task_registry_entry(task_name);
            match &entry {
                None => {
                    panic!("Could not find task named {task_name}");
                }
                Some(task) => match task {
                    TaskRegistryEntry::Genesis(entry) => {
                        add_node(
                            SubgraphNames::Genesis,
                            task_name,
                            entry.builder.get_name(),
                            entry.builder.get_dependencies(),
                        );
                    }
                    TaskRegistryEntry::Byron(entry) => {
                        add_node(
                            SubgraphNames::Byron,
                            task_name,
                            entry.builder.get_name(),
                            entry.builder.get_dependencies(),
                        );
                    }
                    TaskRegistryEntry::Multiera(entry) => {
                        add_node(
                            SubgraphNames::Multiera,
                            task_name,
                            entry.builder.get_name(),
                            entry.builder.get_dependencies(),
                        );
                    }
                },
            }
        }
    }

    Graph {
        name: plan_name.to_string(),
        nodes,
        edges,
        subgraphs: subgraphs
            .iter()
            .map(|(name, nodes)| (*name, nodes.clone()))
            .collect(),
    }
}

type NodeIdentifier = usize;
type EdgeType = (usize, usize);
type SubgraphIdentifier = usize;

pub struct Graph {
    name: String,
    nodes: Vec<String>,
    edges: Vec<EdgeType>,
    subgraphs: Vec<(SubgraphNames, Vec<NodeIdentifier>)>,
}

impl<'a> dot::Labeller<'a, NodeIdentifier, &'a EdgeType, SubgraphIdentifier> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new(self.name.clone()).unwrap()
    }
    fn node_id(&'a self, n: &NodeIdentifier) -> dot::Id<'a> {
        dot::Id::new(format!("N{n}")).unwrap()
    }
    fn node_label<'b>(&'b self, n: &NodeIdentifier) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(self.nodes[*n].clone().into())
    }
    fn edge_label<'b>(&'b self, _: &&EdgeType) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr("".into())
    }

    fn node_shape(&'a self, _n: &NodeIdentifier) -> Option<LabelText<'a>> {
        Some(LabelText::LabelStr(Cow::Owned("box".to_string())))
    }

    fn subgraph_id(&'a self, s: &SubgraphIdentifier) -> Option<dot::Id<'a>> {
        dot::Id::new(format!("cluster_{}", self.subgraphs[*s].0)).ok()
    }

    fn subgraph_label<'b>(&'b self, s: &SubgraphIdentifier) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(Cow::Owned(self.subgraphs[*s].0.to_string()))
    }

    fn subgraph_color<'b>(&'b self, _: &SubgraphIdentifier) -> Option<dot::LabelText<'b>> {
        Some(dot::LabelText::LabelStr(Cow::Owned("grey85".to_string())))
    }
}

impl<'a> dot::GraphWalk<'a, NodeIdentifier, &'a EdgeType, SubgraphIdentifier> for Graph {
    fn nodes(&self) -> dot::Nodes<'a, NodeIdentifier> {
        (0..self.nodes.len()).collect()
    }
    fn edges(&'a self) -> dot::Edges<'a, &'a EdgeType> {
        self.edges.iter().collect()
    }
    fn source(&self, e: &&EdgeType) -> NodeIdentifier {
        e.0
    }
    fn target(&self, e: &&EdgeType) -> NodeIdentifier {
        e.1
    }

    fn subgraphs(&'a self) -> dot::Subgraphs<'a, SubgraphIdentifier> {
        (0..self.subgraphs.len()).collect()
    }

    fn subgraph_nodes(&'a self, s: &SubgraphIdentifier) -> dot::Nodes<'a, NodeIdentifier> {
        Cow::Borrowed(&self.subgraphs[*s].1)
    }
}
