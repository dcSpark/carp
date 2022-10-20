use async_trait::async_trait;

pub type MultiEraBlock<'b> = pallas::ledger::traverse::MultiEraBlock<'b>;

#[async_trait]
pub trait StoppableService {
    async fn stop(self) -> anyhow::Result<()>;
}
