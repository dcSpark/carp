# MultieraWingRidersV1SwapTask
Adds WingRiders V1 swaps to the database


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

   * [MultieraUsedInputTask](./MultieraUsedInputTask)


## Data accessed
#### Reads from

   * ` multiera_txs `
   * ` multiera_addresses `
   * ` multiera_used_inputs_to_outputs_map `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_wingriders_v1_swap.rs)
