/* @name sqlBlockLatest */
SELECT * FROM "Block" ORDER BY "Block".id DESC LIMIT 1 OFFSET (:offset);
