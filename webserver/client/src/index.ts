import type { EndpointTypes } from '../../shared/routes';
import type { ErrorShape } from '../../shared/errors';
import type { AxiosError, AxiosResponse } from "axios";
import axios from "axios";
import { Routes } from '../../shared/routes';

export type { RelationFilter, RelationFilterType, Pagination, UtxoPointer
} from '../../shared/models/common';
export * from '../../shared/routes';
export * from '../../shared/errors';
export type { EndpointTypes } from '../../shared/routes';
export type { ErrorShape } from '../../shared/errors';


export async function query<T extends Routes>(
  urlBase: string,
  data: EndpointTypes[T]["input"]
): Promise<EndpointTypes[T]["response"]> {
  const result = await axios.post<
  EndpointTypes[T]["response"],
    AxiosResponse<EndpointTypes[T]["response"]>,
    EndpointTypes[T]["input"]
  >(`${urlBase}/${Routes.transactionHistory}`, data);
  return result.data;
}

export function getErrorResponse(
  err: AxiosError<ErrorShape, unknown>
): AxiosResponse<ErrorShape, unknown> {
  if (err.response == null) throw new Error(`Unexpected null response`);
  return err.response;
}
