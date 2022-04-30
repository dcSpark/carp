/* @name sqlBlockByHash */
SELECT "Block".id as until_block_id
FROM "Block"
WHERE "Block".hash = (:until_block);