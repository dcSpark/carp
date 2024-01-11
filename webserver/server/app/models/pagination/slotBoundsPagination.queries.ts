/** Types generated for queries found in "app/models/pagination/slotBoundsPagination.sql" */
import { PreparedQuery } from '@pgtyped/runtime';

/** 'SlotBoundsPagination' parameters type */
export interface ISlotBoundsPaginationParams {
  high: number;
  low: number;
}

/** 'SlotBoundsPagination' return type */
export interface ISlotBoundsPaginationResult {
  max_slot: number;
  max_tx_id: string | null;
  min_slot: number;
  min_tx_id: string | null;
}

/** 'SlotBoundsPagination' query type */
export interface ISlotBoundsPaginationQuery {
  params: ISlotBoundsPaginationParams;
  result: ISlotBoundsPaginationResult;
}

const slotBoundsPaginationIR: any = {"usedParamSet":{"low":true,"high":true},"params":[{"name":"low","required":true,"transform":{"type":"scalar"},"locs":[{"a":193,"b":197}]},{"name":"high","required":true,"transform":{"type":"scalar"},"locs":[{"a":449,"b":454}]}],"statement":"WITH MIN_HASH AS\n\t(SELECT COALESCE(\"Transaction\".ID,\n\n\t\t\t\t\t\t\t\t\t\t-1) AS MIN_TX_ID,\n\t\t\tSLOT AS MIN_SLOT\n\t\tFROM \"Transaction\"\n\t\tJOIN \"Block\" ON \"Block\".ID = \"Transaction\".BLOCK_ID\n\t\tWHERE SLOT <= :low!\n\t\tORDER BY \"Block\".ID DESC, \"Transaction\".ID DESC\n\t\tLIMIT 1),\n\tMAX_HASH AS\n\t(SELECT SLOT AS MAX_SLOT,\n\t\t\tCOALESCE(MAX(\"Transaction\".ID),\n\n\t\t\t\t-2) AS MAX_TX_ID\n\t\tFROM \"Transaction\"\n\t\tJOIN \"Block\" ON \"Transaction\".BLOCK_ID = \"Block\".ID\n\t\tWHERE SLOT <= :high!\n\t\tGROUP BY \"Block\".ID\n\t\tORDER BY \"Block\".ID DESC\n\t\tLIMIT 1)\nSELECT *\nFROM MIN_HASH\nLEFT JOIN MAX_HASH ON 1 = 1"};

/**
 * Query generated from SQL:
 * ```
 * WITH MIN_HASH AS
 * 	(SELECT COALESCE("Transaction".ID,
 * 
 * 										-1) AS MIN_TX_ID,
 * 			SLOT AS MIN_SLOT
 * 		FROM "Transaction"
 * 		JOIN "Block" ON "Block".ID = "Transaction".BLOCK_ID
 * 		WHERE SLOT <= :low!
 * 		ORDER BY "Block".ID DESC, "Transaction".ID DESC
 * 		LIMIT 1),
 * 	MAX_HASH AS
 * 	(SELECT SLOT AS MAX_SLOT,
 * 			COALESCE(MAX("Transaction".ID),
 * 
 * 				-2) AS MAX_TX_ID
 * 		FROM "Transaction"
 * 		JOIN "Block" ON "Transaction".BLOCK_ID = "Block".ID
 * 		WHERE SLOT <= :high!
 * 		GROUP BY "Block".ID
 * 		ORDER BY "Block".ID DESC
 * 		LIMIT 1)
 * SELECT *
 * FROM MIN_HASH
 * LEFT JOIN MAX_HASH ON 1 = 1
 * ```
 */
export const slotBoundsPagination = new PreparedQuery<ISlotBoundsPaginationParams,ISlotBoundsPaginationResult>(slotBoundsPaginationIR);


