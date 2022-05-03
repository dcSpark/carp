use entity::{
    prelude::*,
    sea_orm::{prelude::*, JoinType, QueryOrder, QuerySelect},
};
use futures::TryStreamExt;

pub async fn start_reparse(conn: DatabaseConnection) -> anyhow::Result<()> {
    tracing::info!("{}", "Starting to process txs");

    // TODO: switch to join_all();
    reparse_addresses(&conn, 0).await?;
    reparse_tx_out(&conn, 0).await?;
    reparse_txs(&conn, 0).await?;
    Ok(())
}

static PAGE_SIZE: usize = 8192 * 4;

async fn reparse_txs(conn: &DatabaseConnection, start_index: u64) -> Result<(), DbErr> {
    let tx_count = Transaction::find().count(conn).await?;
    let mut tx_stream = Transaction::find()
        .order_by_asc(TransactionColumn::Id)
        .filter(TransactionColumn::Id.gt(start_index))
        .paginate(conn, PAGE_SIZE)
        .into_stream();

    while let Some(txs) = &tx_stream.try_next().await? {
        println!(
            "txs: {} / {} ({:.1}%)",
            txs.first().unwrap().id,
            tx_count,
            (100.0 * txs.first().unwrap().id as f64) / (tx_count as f64)
        );
        for tx in txs {
            // TODO: this will fail on all Byron txs
            // https://github.com/dcSpark/cardano-multiplatform-lib/issues/61
            if let Err(e) = &cardano_multiplatform_lib::Transaction::from_bytes(tx.payload.clone())
            {
                println!(
                    "\nFailed tx at tx hash {}. {:?} {}\n",
                    hex::encode(&tx.hash),
                    e,
                    hex::encode(&tx.payload)
                );
            };
        }
    }
    println!("Done parsing transactions");
    Ok(())
}

async fn reparse_addresses(conn: &DatabaseConnection, start_index: u64) -> Result<(), DbErr> {
    let address_count = Address::find().count(conn).await?;
    let mut address_stream = Address::find()
        .order_by_asc(AddressColumn::Id)
        .filter(AddressColumn::Id.gt(start_index))
        .paginate(conn, PAGE_SIZE)
        .into_stream();

    while let Some(addresses) = &address_stream.try_next().await? {
        println!(
            "addrs: {} / {} ({:.1}%)",
            addresses.first().unwrap().id,
            address_count,
            (100.0 * addresses.first().unwrap().id as f64) / (address_count as f64)
        );
        for addr in addresses {
            if let Err(e) =
                &cardano_multiplatform_lib::address::Address::from_bytes(addr.payload.clone())
            {
                let bad_tx = Transaction::find()
                    .join(
                        JoinType::InnerJoin,
                        TransactionRelation::TransactionOutput.def(),
                    )
                    .join(
                        JoinType::InnerJoin,
                        TransactionOutputRelation::Address.def(),
                    )
                    .filter(AddressColumn::Id.eq(addr.id))
                    .one(conn)
                    .await?;
                if addr.payload.len() == 500 {
                    println!(
                        "Expected failure on truncated address at tx hash {}",
                        hex::encode(bad_tx.unwrap().hash)
                    );
                } else {
                    println!(
                        "\nFailed address at tx hash {}. {:?} {}\n",
                        hex::encode(bad_tx.unwrap().hash),
                        e,
                        hex::encode(&addr.payload)
                    );
                }
            };
        }
    }
    println!("Done parsing addresses");
    Ok(())
}

async fn reparse_tx_out(conn: &DatabaseConnection, start_index: u64) -> Result<(), DbErr> {
    let tx_out_count = TransactionOutput::find().count(conn).await?;
    let mut tx_out_stream = TransactionOutput::find()
        .order_by_asc(TransactionOutputColumn::Id)
        .filter(TransactionOutputColumn::Id.gt(start_index))
        .paginate(conn, PAGE_SIZE)
        .into_stream();

    while let Some(tx_outs) = &tx_out_stream.try_next().await? {
        println!(
            "tx_outs: {} / {} ({:.1}%)",
            tx_outs.first().unwrap().id,
            tx_out_count,
            (100.0 * tx_outs.first().unwrap().id as f64) / (tx_out_count as f64)
        );
        for tx_out in tx_outs {
            // TODO: this will fail on all Byron txs
            // https://github.com/dcSpark/cardano-multiplatform-lib/issues/61
            if let Err(e) =
                &cardano_multiplatform_lib::TransactionOutput::from_bytes(tx_out.payload.clone())
            {
                let bad_tx = Transaction::find()
                    .join(
                        JoinType::InnerJoin,
                        TransactionRelation::TransactionOutput.def(),
                    )
                    .filter(TransactionOutputColumn::Id.eq(tx_out.id))
                    .one(conn)
                    .await?;
                println!(
                    "\nFailed tx_out at tx hash {}. {:?} {}\n",
                    hex::encode(bad_tx.unwrap().hash),
                    e,
                    hex::encode(&tx_out.payload)
                );
            };
        }
    }
    println!("Done parsing tx_outs");
    Ok(())
}
