/* @name sqlBlockMinter */
SELECT key FROM "Block"
INNER JOIN "BlockMinter" ON "BlockMinter".id = "Block".id
WHERE "Block".hash = ANY (:addresses);