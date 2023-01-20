# MultieraSundaeSwapV1MeanPriceTask
Adds SundaeSwap V1 mean price updates to the database


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

   * [MultieraAddressTask](./MultieraAddressTask)


## Data accessed
#### Reads from

   * ` multiera_txs `
   * ` multiera_addresses `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_sundaeswap_v1_mean_price.rs)
