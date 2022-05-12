---
sidebar_position: 1
---

# Core pillars

- **Flexibility**: The webserver and client follow the same philosophy as the indexer: flexibility by providing raw cbor instead of JSON objects.
- **Type safety**: All the code in the webserver and in the client have type definitions available.
- **Safe pagination**: All endpoints are paginated by blocks. Having a clear start block and end block for all queries is important for two reasons:
  - Data changes over time. For example, a user could make a transaction while you are paginating over their account information which could lead to inconsistencies
  - In Cardano, small rollbacks of 1~2 block are very frequent and expected (read Ouroboros for why), so it's important that the block you're using still exists between paginated calls

