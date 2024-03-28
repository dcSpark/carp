/* @name sqlDrepStakeDelegationForAddress */
SELECT encode(drep_credential, 'hex') as "drep!", encode("Transaction".hash, 'hex') as "tx_id!"
FROM "StakeDelegationDrepCredentialRelation"
JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
JOIN "Transaction" ON "Transaction".id = "StakeDelegationDrepCredentialRelation".tx_id
JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE
	"StakeCredential".credential = :credential! AND
	"Block".slot <= :slot!
ORDER BY "Transaction".id DESC
LIMIT 1;
