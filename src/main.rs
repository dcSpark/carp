use oura_postgres_sink::blocks_table::prelude::{Block, BlockModel};
use sea_orm::{prelude::*, Database};

// DATABASE_URL=postgresql://root:root@localhost:5432/azul

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: use an environment variable before going to production
    let conn = Database::connect("postgresql://root:root@localhost:5432/cardano").await?;

    let blocks: Vec<BlockModel> = Block::find().all(&conn).await?;

    println!("{:?}", blocks);

    Ok(())
}
