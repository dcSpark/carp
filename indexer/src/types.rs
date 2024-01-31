use async_trait::async_trait;

pub type MultiEraBlock = cml_multi_era::MultiEraBlock;

#[async_trait]
pub trait StoppableService {
    async fn stop(self) -> anyhow::Result<()>;
}
