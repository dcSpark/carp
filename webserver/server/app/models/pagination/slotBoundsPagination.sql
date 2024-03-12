/* @name slotBoundsPagination */
WITH
    low_block AS (
        SELECT
            "Block".id,
            "Block".slot
        FROM
            "Block"
        WHERE
        /*
            We use <= here even though slot filter parameter is exclusive on the
            lower bound. This is because the tx that we find here (after joining
            with min_hash) is used later in a condition of the form:

            "Transaction".id > :after_tx_id!

            For example.

            Lets say :low is 1, and there is a block with txs at this slot. This
            means we want to find _at least_ the first tx in slot 2.

            So what we want in this query is to find the last tx in slot 1.
            Then, when we use the > comparator we would get the right tx.
        */
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