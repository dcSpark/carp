/* @name votesForAddress */
SELECT 
    json_agg(
        json_build_object(
            'govActionId', encode(gov_action_id, 'hex'),
            'vote', encode(vote, 'hex')
        )
    ) as "votes!", 
    encode(tx.hash, 'hex') as "txId!",
    MIN(encode("Block".hash, 'hex')) as "block!"
FROM  "GovernanceVote"
JOIN "Transaction" tx ON tx.id = "GovernanceVote".tx_id
JOIN "Block" ON "Block".id = tx.block_id
WHERE
	tx.id < :before_tx_id AND
	tx.id <= :until_tx_id! AND
    voter = :voter!
GROUP BY tx.id
ORDER BY tx.id DESC
LIMIT :limit!;


/* 
@name didVote
@param gov_action_ids -> (...)
*/
SELECT gov_action_id as "govActionId!", vote as "vote!", "Transaction".id as "txId!"
FROM  "GovernanceVote"
JOIN "Transaction" ON "GovernanceVote".tx_id = "Transaction".id
WHERE
	"Transaction".id <= :until_tx_id! AND
    voter = :voter! AND
    gov_action_id IN :gov_action_ids!
ORDER BY "Transaction".id;