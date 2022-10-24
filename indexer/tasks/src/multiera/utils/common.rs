use std::collections::BTreeSet;

use entity::{
    prelude::*,
    sea_orm::{entity::*, prelude::*, Condition, DatabaseTransaction},
};

pub async fn asset_from_pair(
    db_tx: &DatabaseTransaction,
    pairs: &[(Vec<u8> /* policy id */, Vec<u8> /* asset name */)],
) -> Result<Vec<NativeAssetModel>, DbErr> {
    // https://github.com/dcSpark/carp/issues/46
    let mut asset_conditions = Condition::any();
    for (policy_id, asset_name) in pairs.iter() {
        asset_conditions = asset_conditions.add(
            Condition::all()
                .add(NativeAssetColumn::PolicyId.eq(policy_id.clone()))
                .add(NativeAssetColumn::AssetName.eq(asset_name.clone())),
        );
    }

    let assets = NativeAsset::find()
        .filter(asset_conditions)
        .all(db_tx)
        .await?;
    Ok(assets)
}
