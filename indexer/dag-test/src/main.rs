use shred::{DispatcherBuilder, World};
use tasks::test::{PrepareTask, PrintA, PrintB, ResA, ResB};

mod tasks;
extern crate shred;

fn main() {
    let mut world = World::empty();
    let mut dispatcher = DispatcherBuilder::new()
        .with(PrepareTask, "prepareTask", &[])
        .with(PrintA, "printA", &[])
        .with(PrintB, "printB", &["printA"])
        .build();
    world.insert(ResA(0));
    world.insert(ResB(0));

    dispatcher.dispatch(&mut world);
    println!("asdf");
}
