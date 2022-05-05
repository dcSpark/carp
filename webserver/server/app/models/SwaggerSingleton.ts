import type { JsonObject } from 'swagger-ui-express';

// eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-assignment
let cachedJson: JsonObject = null as any;

export default async (): Promise<JsonObject> => {
  // if not cached yet, fetch and cache
  // however, to get hot reload working properly, we re-fetch every time for dev builds
  if (cachedJson == null || process.env.NODE_ENV === 'development') {
    cachedJson = await import('../../build/swagger.json');
  }
  return cachedJson;
};
