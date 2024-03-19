import type { EndpointTypes } from "../../shared/routes";
import type { ErrorShape } from "../../shared/errors";
import type { AxiosError, AxiosResponse } from "axios";
import axios from "axios";
import type { Routes } from "../../shared/routes";

export async function query<T extends Routes>(
  urlBase: string,
  route: T,
  data: EndpointTypes[T]["input"]
): Promise<EndpointTypes[T]["response"]> {
  const result = await axios.post<
    EndpointTypes[T]["response"],
    AxiosResponse<EndpointTypes[T]["response"]>,
    EndpointTypes[T]["input"]
  >(`${urlBase}/${route}`, data);
  return result.data;
}

export function getErrorResponse(
  err: AxiosError<ErrorShape, unknown>
): AxiosResponse<ErrorShape, unknown> {
  if (err.response == null) throw new Error(`Unexpected null response`);
  return err.response;
}
