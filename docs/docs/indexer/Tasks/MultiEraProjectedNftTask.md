# MultiEraProjectedNftTask
Parses projected NFT contract data


<details>
    <summary>Configuration</summary>

```rust
use pallas::ledger::addresses::Address;
use pallas::ledger::primitives::alonzo::PlutusScript;
use pallas::ledger::primitives::babbage::PlutusV2Script;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AddressConfig {
    pub address: String,
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
