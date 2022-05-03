extern crate shred;

use entity::{
    prelude::*,
    sea_orm::{prelude::*, DatabaseTransaction, Set},
};
use pallas::ledger::primitives::byron;
use shred::{DispatcherBuilder, Read, ReadExpect, ResourceId, System, SystemData, World, Write};
use std::{thread, time};

#[derive(SystemData)]
pub struct Data<'a> {
    dbTx: ReadExpect<'a, DatabaseTransaction>,
    block: ReadExpect<'a, (BlockModel, byron::Block)>,
}

pub struct ByronInputTask;

impl<'a> System<'a> for ByronInputTask {
    type SystemData = Data<'a>;

    fn run(&mut self, bundle: Data<'a>) {}
}
