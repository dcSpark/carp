# MultieraBlockTask
Adds the block to the database


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

## Data accessed
#### Writes to

   * ` multiera_block `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/multiera/multiera_block.rs)
