# MultiEraProjectedNftTask
Parses projected NFT contract data


<details>
    <summary>Configuration</summary>

```rust
use super::PayloadConfig::PayloadConfig;
use super::ReadonlyConfig::ReadonlyConfig;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct PayloadAndReadonlyConfig {
    pub include_payload: bool,
    pub readonly: bool,
}

```
</details>


## Era
` multiera `

## Dependencies

   * [MultieraUsedInputTask](./MultieraUsedInputTask)
   * [MultieraOutputTask](./MultieraOutputTask)


## Data accessed
#### Reads from

   * ` multiera_txs `
   * ` multiera_outputs `
   * ` multiera_used_inputs_to_outputs_map `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_projected_nft.rs)
