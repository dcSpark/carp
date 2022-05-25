import { Address } from "./Address";
import { Pagination, RelationFilter } from "./common";

export type AddressUsedRequest = {
  addresses: Address[];
} & Pagination;

export type AddressUsedResponse = {
  addresses: Address[];
};
