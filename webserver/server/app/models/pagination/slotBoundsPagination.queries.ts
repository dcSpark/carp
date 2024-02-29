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

const slotBoundsPaginationIR: any = {"usedParamSet":{"low":true,"high":true},"params":[{"name":"low","required":true,"transform":{"type":"scalar"},"locs":[{"a":155,"b":159}]},{"name":"high","required":true,"transform":{"type":"scalar"},"locs":[{"a":409,"b":414}]}],"statement":"WITH\n    low_block AS (\n        SELECT\n            \"Block\".id,\n            \"Block\".slot\n        FROM\n            \"Block\"\n        WHERE\n            slot <= :low! AND tx_count > 0\n        ORDER BY\n            \"Block\".id DESC\n        LIMIT\n            1\n    ),\n    high_block AS (\n        SELECT\n            \"Block\".id,\n            \"Block\".slot\n        FROM\n            \"Block\"\n        WHERE\n            slot <= :high! AND tx_count > 0\n        ORDER BY\n            \"Block\".id DESC\n        LIMIT\n            1\n    ),\n    min_hash AS (\n        SELECT\n            COALESCE(MAX(\"Transaction\".id), -1) AS min_tx_id,\n            slot AS min_slot\n        FROM\n            \"Transaction\"\n            JOIN low_block ON \"Transaction\".block_id = low_block.id\n        GROUP BY\n            low_block.slot\n        LIMIT\n            1\n    ),\n    max_hash AS (\n        SELECT\n            COALESCE(MAX(\"Transaction\".id), -2) AS max_tx_id,\n            slot AS max_slot\n        FROM\n            \"Transaction\"\n            JOIN high_block ON \"Transaction\".block_id = high_block.id\n        GROUP BY\n            high_block.slot\n    )\nSELECT\n    *\nFROM min_hash\nLEFT JOIN max_hash ON 1 = 1"};

/**
 * Query generated from SQL:
 * ```
 * WITH
 *     low_block AS (
 *         SELECT
 *             "Block".id,
 *             "Block".slot
 *         FROM
 *             "Block"
 *         WHERE
 *             slot <= :low! AND tx_count > 0
 *         ORDER BY
 *             "Block".id DESC
 *         LIMIT
 *             1
 *     ),
 *     high_block AS (
 *         SELECT
 *             "Block".id,
 *             "Block".slot
 *         FROM
 *             "Block"
 *         WHERE
 *             slot <= :high! AND tx_count > 0
 *         ORDER BY
 *             "Block".id DESC
 *         LIMIT
 *             1
 *     ),
 *     min_hash AS (
 *         SELECT
 *             COALESCE(MAX("Transaction".id), -1) AS min_tx_id,
 *             slot AS min_slot
 *         FROM
 *             "Transaction"
 *             JOIN low_block ON "Transaction".block_id = low_block.id
 *         GROUP BY
 *             low_block.slot
 *         LIMIT
 *             1
 *     ),
 *     max_hash AS (
 *         SELECT
 *             COALESCE(MAX("Transaction".id), -2) AS max_tx_id,
 *             slot AS max_slot
 *         FROM
 *             "Transaction"
 *             JOIN high_block ON "Transaction".block_id = high_block.id
 *         GROUP BY
 *             high_block.slot
 *     )
 * SELECT
 *     *
 * FROM min_hash
 * LEFT JOIN max_hash ON 1 = 1
 * ```
 */
export const slotBoundsPagination = new PreparedQuery<ISlotBoundsPaginationParams,ISlotBoundsPaginationResult>(slotBoundsPaginationIR);


