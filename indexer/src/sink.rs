use dcspark_blockchain_source::EventObject;

pub trait Sink {
    type Event: EventObject;

    fn process(&mut self, event: Self::Event) -> anyhow::Result<()>;
}