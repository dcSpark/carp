/* 
@name sqlStakeDelegationByPool
@param pools -> (...)
*/
SELECT 
	encode("Transaction".hash, 'hex') as "tx_id!",
	encode("Block".hash, 'hex') as "block!",
	json_agg(json_build_object(
		'credential', encode(credential, 'hex'),
		'slot', "Block".slot,
		'pool',
			CASE WHEN "StakeDelegationCredentialRelation".pool_credential IN :pools!
			THEN encode("StakeDelegationCredentialRelation".pool_credential, 'hex')
			ELSE NULL
			END
		)
	) as "payload!"
FROM "StakeDelegationCredentialRelation"
JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
JOIN "Transaction" ON "Transaction".id = "StakeDelegationCredentialRelation".tx_id
JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE 
    (
		"StakeDelegationCredentialRelation".pool_credential IN :pools! OR
	 	"StakeDelegationCredentialRelation".previous_pool IN :pools!
	) AND
	"Transaction".id > :after_tx_id! AND
	"Transaction".id <= :until_tx_id!
GROUP BY ("Block".hash, "Transaction".id)
ORDER BY "Transaction".id ASC
LIMIT :limit!;