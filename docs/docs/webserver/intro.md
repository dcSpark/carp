---
sidebar_position: 1
---

# Core pillars

- **Flexibility**: The webserver and client follow the same philosophy as the indexer: flexibility by providing raw cbor instead of JSON objects.
- **Type safety**: All the code in the webserver and in the client have type definitions available.
- **Safe pagination**:
  1. Atomic: the database state changing mid-pagination should either not affect the result set or explicitly fail.
  2. Liveness: pagination should always make progress (no change of getting stuck paginating infinitely)
