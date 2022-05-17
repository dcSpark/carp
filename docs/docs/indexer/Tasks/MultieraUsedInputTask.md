# MultieraUsedInputTask

Adds the used inputs to the database \(regular inputs in most cases, collateral inputs if tx fails

## Era

`multiera`

## Dependencies

- [MultieraOutputTask](./MultieraOutputTask)

## Data accessed

#### Reads from

- `multiera_txs`

#### Writes to

- `vkey_relation_map`
- `multiera_used_inputs`

## Full source

[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_used_inputs.rs)
