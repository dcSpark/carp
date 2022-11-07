# ByronBlockMinterTask
Adds the minter of a block to the database


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

## Dependencies

   * [ByronBlockTask](./ByronBlockTask)


## Data accessed
#### Reads from

   * ` byron_block `


## Full source
[source](https://github.com/dcSpark/carp/tree/main/indexer/tasks/src/byron/byron_block_minter.rs)
