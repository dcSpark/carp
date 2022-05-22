# ByronBlockTask
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
` byron `

## Data accessed
#### Writes to

   * ` byron_block `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/byron/byron_block.rs)
