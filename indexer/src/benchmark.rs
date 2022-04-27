use entity::{
    prelude::*,
    sea_orm::{prelude::*, ColumnTrait, DatabaseTransaction, TransactionTrait},
};
use migration::DbErr;
use oura::pipelining::StageReceiver;

pub struct Config<'a> {
    pub conn: &'a DatabaseConnection,
}

impl<'a> Config<'a> {
    pub async fn bootstrap(&self, _input: StageReceiver) -> anyhow::Result<()> {
        tracing::info!("{}", "Starting to process blocks");

        self.conn
            .transaction::<_, (), DbErr>(|txn| Box::pin(benchmark(txn)))
            .await?;

        Ok(())
    }
}

async fn benchmark(txn: &DatabaseTransaction) -> Result<(), DbErr> {
    println!(
        "{:?}",
        "8200581cff57a0bbcaaada72b6e3d6d9044c420d1a8dd97794884d39021f1e".to_owned()
            + &format!("{:02}", 0)
    );

    let creds = (0..100).map(|i| {
        hex::decode(
            "8200581cff57a0bbcaaada72b6e3d6d9044c420d1a8dd97794884d39021f1e".to_owned()
                + &format!("{:02}", i),
        )
        .unwrap()
    });

    let time_counter = std::time::Instant::now();

    // for i in 0..100 {
    //     StakeCredential::find()
    //         .filter(
    //             StakeCredentialColumn::Credential.eq(hex::decode(
    //                 "8200581cff57a0bbcaaada72b6e3d6d9044c420d1a8dd97794884d39021f1ea3",
    //             )
    //             .unwrap()),
    //         )
    //         // note: we know this exists ("credential" is unique) and "all" is faster than "one" if we know the result exists
    //         .all(txn)
    //         .await?;
    // }

    let mut base = StakeCredential::find();
    for cred in creds {
        base = base.filter(StakeCredentialColumn::Credential.eq(cred))
    }
    base.all(txn).await?;

    println!("{:?}", time_counter.elapsed());
    // ~38
    Ok(())
}
