extern crate ini;
use ini::Ini;

pub struct ExecutionPlan(pub Ini);
impl ExecutionPlan {
    pub fn load_from_file(path: &str) -> ExecutionPlan {
        ExecutionPlan(Ini::load_from_file(path).unwrap())
    }
}
