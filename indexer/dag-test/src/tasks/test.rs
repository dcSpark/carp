extern crate shred;

use shred::{DispatcherBuilder, Read, ResourceId, System, SystemData, World, Write};
use std::{thread, time};

#[derive(Debug, Default)]
pub struct ResA(pub(crate) u64);

#[derive(Debug, Default)]
pub struct ResB(pub(crate) u64);

#[derive(SystemData)] // Provided with `shred-derive` feature
pub struct Prepare<'a> {
    a: Write<'a, ResA>,
    b: Write<'a, ResB>,
}

#[derive(SystemData)] // Provided with `shred-derive` feature
pub struct Data<'a> {
    a: Read<'a, ResA>,
    b: Read<'a, ResB>,
}

pub struct PrepareTask;

impl<'a> System<'a> for PrepareTask {
    type SystemData = Prepare<'a>;

    fn run(&mut self, mut bundle: Prepare<'a>) {
        println!("{}", "prepare start");
        bundle.a.0 = 5;
        bundle.b.0 = 6;
        thread::sleep(time::Duration::from_secs(2));
        println!("{:?} {:?}", &*bundle.a, &*bundle.b);
    }
}

pub struct PrintA;

impl<'a> System<'a> for PrintA {
    type SystemData = Data<'a>;

    fn run(&mut self, bundle: Data<'a>) {
        println!("{}", "Start a");

        thread::sleep(time::Duration::from_secs(2));
        // panic!();
        println!("{:?}", &*bundle.a);
    }
}

pub struct PrintB;

impl<'a> System<'a> for PrintB {
    type SystemData = Data<'a>;

    fn run(&mut self, bundle: Data<'a>) {
        println!("{}", "Start b");
        thread::sleep(time::Duration::from_secs(3));
        println!("{:?}", &*bundle.b);
    }
}
