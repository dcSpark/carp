/*
@name sqlMintBurnRange
*/
SELECT
    "AssetMint".amount as amount,
    encode("NativeAsset".policy_id, 'hex') as policy_id,
    encode("NativeAsset".asset_name, 'hex') as asset_name,
    encode("Transaction".hash, 'hex') as action_tx_id,
    encode("Block".hash, 'hex') as action_block_id,
    CASE
        WHEN "TransactionMetadata".payload = NULL THEN NULL
        ELSE encode("TransactionMetadata".payload, 'hex')
        END AS action_tx_metadata,
    "Block".slot as action_slot
FROM "AssetMint"
         LEFT JOIN "TransactionMetadata" ON "TransactionMetadata".id = "AssetMint".tx_id
         JOIN "NativeAsset" ON "NativeAsset".id = "AssetMint".asset_id
         JOIN "Transaction" ON "Transaction".id = "AssetMint".tx_id
         JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE
        "Block".slot > :min_slot!
    AND "Block".slot <= :max_slot!
ORDER BY ("Block".height, "Transaction".tx_index) ASC;

/*
@name sqlMintBurnRangeByPolicyIds
@param policy_ids -> (...)
*/
SELECT
    "AssetMint".amount as amount,
    encode("NativeAsset".policy_id, 'hex') as policy_id,
    encode("NativeAsset".asset_name, 'hex') as asset_name,
    encode("Transaction".hash, 'hex') as action_tx_id,
    encode("Block".hash, 'hex') as action_block_id,
    CASE
        WHEN "TransactionMetadata".payload = NULL THEN NULL
        ELSE encode("TransactionMetadata".payload, 'hex')
        END AS action_tx_metadata,
    "Block".slot as action_slot
FROM "AssetMint"
         LEFT JOIN "TransactionMetadata" ON "TransactionMetadata".id = "AssetMint".tx_id
         JOIN "NativeAsset" ON "NativeAsset".id = "AssetMint".asset_id
         JOIN "Transaction" ON "Transaction".id = "AssetMint".tx_id
         JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE
        "Block".slot > :min_slot!
    AND "Block".slot <= :max_slot!
    AND "NativeAsset".policy_id IN :policy_ids!
ORDER BY ("Block".height, "Transaction".tx_index) ASC;
