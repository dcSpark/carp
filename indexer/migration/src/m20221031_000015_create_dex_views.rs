use sea_orm::{ConnectionTrait, Statement};
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20221031_000015_create_dex_views"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
            CREATE VIEW "DexSwap" AS SELECT 
                id, tx_id, address_id, dex, asset1_id, asset2_id, amount1, amount2, operation 
            FROM "Dex" WHERE "operation" IN (0, 1);
        "#;
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await?;

        let sql = r#"
            CREATE VIEW "DexMeanPrice" AS SELECT 
                id, tx_id, address_id, dex, asset1_id, asset2_id, amount1, amount2
            FROM "Dex" WHERE "operation" = 2;
        "#;
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
            DROP VIEW IF EXISTS "DexSwap";
             "#;
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await?;

        let sql = r#"
            DROP VIEW IF EXISTS "DexMeanPrice";
             "#;
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())
    }
}
