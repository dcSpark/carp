---
sidebar_position: 3
---

# Pagination Philosophy

Implementing our safe pagination philosophy requires different kinds of pagination for different endpoints.

## Block or Transaction-based data

Having a clear start block and end block for these queries is important for two reasons:

- Data changes over time. For example, a user could make a transaction while you are paginating over their account information which could lead to inconsistencies
- In Cardano, small rollbacks of 1~2 block are very frequent and expected (read Ouroboros for why), so it's important that the block you're using still exists between paginated calls

However, paginating by blocks only is not enough to guarantee liveness as it's possible to hit the pagination limit partway through a block. To avoid this problem, the pagination needs to support a starting transaction inside the block for the next call to avoid duplication.

This is why all endpoints that rely on block or transaction data takes a `after` and `until` parameter based on the above requirements.

## Non-block-based data

Some data does not strictly belong to a block or transaction and instead can appear in multiple transactions or blocks. One main example of this is addresses. Just because transaction that contains an address gets rolled back, it doesn't mean the address itself stops existing (it may still exist in other places).

Although this isn't problematic in itself, it does cause an issue for pagination of queries that rely solely on these types. Some example queries of this type:

- All addresses for a set of credentials
- All asset names for a policy ID

There are three ways you could implement pagination on these types:

1. Defining the return to be in lexicographic order. This doesn't work well because in the middle of paginating, a new transaction could introduce an address that is lexicographically before where your iterator is
2. Paginate based off the auto-incrementing field. This means you have to expose some database internals to the user (the ID key), but it does mean you don't have to introduce any new data to your SQL schema for pagination. Usually paginating off of IDs would not be safe (since the data could be rolled back), but this is safe for these tables if they don't get deleted on rollbacks.
3. Paginate based off a "first seen at" row in the table that keeps track of when an address was first seen on-chain. These can then use this for pagination and also use this to automatically delete addresses on rollback. However, it means storing a few million new database entries.

Based off the above, we prefer option (3) for pagination of these types.
