import { Address } from "./Address";
import { Pagination, RelationFilter } from "./common";

export type AddressUsedRequest = {
  addresses: Address[];
  /** Defaults to `RelationFilterType.NO_FILTER` */
  relationFilter?: RelationFilter;
} & Pagination;

export type AddressUsedResponse = {
  addresses: Address[];
};
