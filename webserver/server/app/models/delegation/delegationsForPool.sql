/* 
@name sqlStakeDelegationByPool
@param pools -> (...)
*/
SELECT 
	encode(credential, 'hex') as credential,
	encode("Transaction".hash, 'hex') as tx_id,
	COALESCE("StakeDelegationCredentialRelation".pool_credential IN :pools!, false) as is_delegation
FROM "StakeDelegationCredentialRelation"
JOIN "StakeCredential" ON stake_credential = "StakeCredential".id
JOIN "Transaction" ON "Transaction".id = "StakeDelegationCredentialRelation".tx_id
JOIN "Block" ON "Transaction".block_id = "Block".id
WHERE 
    (
		"StakeDelegationCredentialRelation".pool_credential IN :pools! OR
	 	"StakeDelegationCredentialRelation".previous_pool IN :pools!
	) AND
	"Block".slot > :min_slot! AND
	"Block".slot <= :max_slot!
ORDER BY ("Block".height, "Transaction".tx_index) ASC;