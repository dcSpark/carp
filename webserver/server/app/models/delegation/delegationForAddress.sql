/* @name sqlStakeDelegationForAddress */
SELECT encode(pool_credential, 'hex') as pool, encode("Transaction".hash, 'hex') as tx_id
FROM "StakeDelegationCredentialRelation"
JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
JOIN "Transaction" ON "Transaction".id = "StakeDelegationCredentialRelation".tx_id
JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE 
	"StakeCredential".credential = :credential! AND
	"Block".slot <= :slot!
ORDER BY ("Block".height, "Transaction".tx_index) DESC
LIMIT 1;