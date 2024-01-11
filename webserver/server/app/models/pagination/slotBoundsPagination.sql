/* @name slotBoundsPagination */
WITH MIN_HASH AS
	(SELECT COALESCE("Transaction".ID,

										-1) AS MIN_TX_ID,
			SLOT AS MIN_SLOT
		FROM "Transaction"
		JOIN "Block" ON "Block".ID = "Transaction".BLOCK_ID
		WHERE SLOT <= :low!
		ORDER BY "Block".ID DESC, "Transaction".ID DESC
		LIMIT 1),
	MAX_HASH AS
	(SELECT SLOT AS MAX_SLOT,
			COALESCE(MAX("Transaction".ID),

				-2) AS MAX_TX_ID
		FROM "Transaction"
		JOIN "Block" ON "Transaction".BLOCK_ID = "Block".ID
		WHERE SLOT <= :high!
		GROUP BY "Block".ID
		ORDER BY "Block".ID DESC
		LIMIT 1)
SELECT *
FROM MIN_HASH
LEFT JOIN MAX_HASH ON 1 = 1;