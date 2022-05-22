# MultieraMetadataTask
Adds the transaction metadata to the database as a series of <metadata\_label, cbor> pair


<details>
    <summary>Configuration</summary>

```rust
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct ReadonlyConfig {
    pub readonly: bool,
}

```
</details>


## Era
` multiera `

## Dependencies

   * [MultieraTransactionTask](./MultieraTransactionTask)


## Data accessed
#### Reads from

   * ` multiera_txs `


#### Writes to

   * ` multiera_metadata `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_metadata.rs)
