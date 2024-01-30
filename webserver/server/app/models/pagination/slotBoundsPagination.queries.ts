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

const slotBoundsPaginationIR: any = {"usedParamSet":{"low":true,"high":true},"params":[{"name":"low","required":true,"transform":{"type":"scalar"},"locs":[{"a":281,"b":285}]},{"name":"high","required":true,"transform":{"type":"scalar"},"locs":[{"a":668,"b":673}]}],"statement":"WITH\nmin_hash AS\n(\n         SELECT   COALESCE(\"Transaction\".id, -1) AS min_tx_id,\n                  slot                           AS min_slot\n         FROM     \"Transaction\"\n         JOIN     \"Block\"\n         ON       \"Block\".id = \"Transaction\".block_id\n         WHERE    slot <= :low!\n         ORDER BY \"Block\".id DESC,\n                  \"Transaction\".id DESC\n         LIMIT 1\n),\nmax_hash AS\n(\n         SELECT   slot                                AS max_slot,\n                  COALESCE(Max(\"Transaction\".id), -2) AS max_tx_id\n         FROM     \"Transaction\"\n         JOIN     \"Block\"\n         ON       \"Transaction\".block_id = \"Block\".id\n         WHERE    slot <= :high!\n         GROUP BY \"Block\".id\n         ORDER BY \"Block\".id DESC\n         LIMIT 1\n)\nSELECT    *\nFROM      min_hash\nLEFT JOIN max_hash\nON        1 = 1"};

/**
 * Query generated from SQL:
 * ```
 * WITH
 * min_hash AS
 * (
 *          SELECT   COALESCE("Transaction".id, -1) AS min_tx_id,
 *                   slot                           AS min_slot
 *          FROM     "Transaction"
 *          JOIN     "Block"
 *          ON       "Block".id = "Transaction".block_id
 *          WHERE    slot <= :low!
 *          ORDER BY "Block".id DESC,
 *                   "Transaction".id DESC
 *          LIMIT 1
 * ),
 * max_hash AS
 * (
 *          SELECT   slot                                AS max_slot,
 *                   COALESCE(Max("Transaction".id), -2) AS max_tx_id
 *          FROM     "Transaction"
 *          JOIN     "Block"
 *          ON       "Transaction".block_id = "Block".id
 *          WHERE    slot <= :high!
 *          GROUP BY "Block".id
 *          ORDER BY "Block".id DESC
 *          LIMIT 1
 * )
 * SELECT    *
 * FROM      min_hash
 * LEFT JOIN max_hash
 * ON        1 = 1
 * ```
 */
export const slotBoundsPagination = new PreparedQuery<ISlotBoundsPaginationParams,ISlotBoundsPaginationResult>(slotBoundsPaginationIR);


