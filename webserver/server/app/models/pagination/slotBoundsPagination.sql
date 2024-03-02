/* @name slotBoundsPagination */
WITH
    low_block AS (
        SELECT
            "Block".id,
            "Block".slot
        FROM
            "Block"
        WHERE
            slot <= :low! AND tx_count > 0
        ORDER BY
            "Block".id DESC
        LIMIT
            1
    ),
    high_block AS (
        SELECT
            "Block".id,
            "Block".slot
        FROM
            "Block"
        WHERE
            slot <= :high! AND tx_count > 0
        ORDER BY
            "Block".id DESC
        LIMIT 1
    ),
    min_hash AS (
        (SELECT
            COALESCE(MAX("Transaction".id), -1) AS min_tx_id
        FROM
            "Transaction"
            JOIN low_block ON "Transaction".block_id = low_block.id
        GROUP BY
            low_block.slot
        LIMIT
            1
        )
        UNION (SELECT min_tx_id FROM (values(-1)) s(min_tx_id))
        ORDER BY min_tx_id DESC
        LIMIT
            1
    ),
    max_hash AS (
        SELECT
            COALESCE(MAX("Transaction".id), -2) AS max_tx_id
        FROM
            "Transaction"
            JOIN high_block ON "Transaction".block_id = high_block.id
        GROUP BY
            high_block.slot
    )
SELECT
    *
FROM min_hash
LEFT JOIN max_hash ON 1 = 1;