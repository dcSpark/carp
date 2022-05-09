use tasks::{
    database_task::TaskRegistryEntry, execution_plan::ExecutionPlan,
    utils::find_task_registry_entry,
};

pub fn generate(exec_plan: &ExecutionPlan, plan_name: &str) -> Graph {
    let mut nodes = Vec::<String>::default();
    let mut edges = Vec::<(usize, usize)>::default();

    let mut add_node = |task_name: &str, entry_name: &str, entry_dependencies: &[&str]| {
        if entry_name == task_name {
            let own_id = nodes.len();
            nodes.push(entry_name.to_string());
            for dep in entry_dependencies {
                let dep_position = nodes.iter().position(|task| task == dep).unwrap();
                edges.push((dep_position, own_id));
            }
        }
    };
    for task_name in exec_plan.0.sections().flatten() {
        let entry = find_task_registry_entry(task_name);
        match &entry {
            None => {
                panic!("Could not find task named {}", task_name);
            }
            Some(task) => match task {
                TaskRegistryEntry::Byron(entry) => {
                    add_node(
                        task_name,
                        entry.builder.get_name(),
                        entry.builder.get_dependencies(),
                    );
                }
                TaskRegistryEntry::Multiera(entry) => {
                    add_node(
                        task_name,
                        entry.builder.get_name(),
                        entry.builder.get_dependencies(),
                    );
                }
            },
        }
    }

    Graph {
        name: plan_name.to_string(),
        nodes,
        edges,
    }
}

type Nd = usize;
type Ed<'a> = &'a (usize, usize);
pub struct Graph {
    name: String,
    nodes: Vec<String>,
    edges: Vec<(usize, usize)>,
}

impl<'a> dot::Labeller<'a, Nd, Ed<'a>> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new(self.name.clone()).unwrap()
    }
    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }
    fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(self.nodes[*n].clone().into())
    }
    fn edge_label<'b>(&'b self, _: &Ed) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr("".into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed<'a>> for Graph {
    fn nodes(&self) -> dot::Nodes<'a, Nd> {
        (0..self.nodes.len()).collect()
    }
    fn edges(&'a self) -> dot::Edges<'a, Ed<'a>> {
        self.edges.iter().collect()
    }
    fn source(&self, e: &Ed) -> Nd {
        e.0
    }
    fn target(&self, e: &Ed) -> Nd {
        e.1
    }
}
