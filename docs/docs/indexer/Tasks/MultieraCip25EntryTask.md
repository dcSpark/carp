# MultieraCip25EntryTask
Maps CIP25 entries to the corresponding DB entry for the asset


<details>
    <summary>Configuration</summary>

```rust
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct EmptyConfig {}

```
</details>


## Era
` multiera `

## Dependencies

   * [MultieraMetadataTask](./MultieraMetadataTask)
   * [MultieraAssetMintTask](./MultieraAssetMintTask)


## Data accessed
#### Reads from

   * ` multiera_assets `
   * ` multiera_metadata `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_cip25entry.rs)
