use crate::config::EmptyConfig::EmptyConfig;
use crate::dsl::task_macro::*;

carp_task! {
  // The task name. This is what will show up in the task graph
  // and this is how you specify dependencies
  name ExampleTask;
  configuration EmptyConfig;
  doc "An example task to help people learn how to write custom Carp tasks";
  // The era your task operates on. Note: different eras have different block representations
  era multiera;
  // List of dependencies for this task. This is an array of names of other tasks
  // Note: your task will run if all dependencies either ran successfully OR were skipped for this block
  dependencies [];
  // Specify which fields your task will have read-access to
  read [multiera_txs];
  // Specify which fields your task will have write-access to
  write [multiera_addresses];
  // Specify whether or not your task needs to run for a given block
  // Note that by design, this function:
  // 1) CANNOT access parent task state
  // 2) Is NOT async
  // 3) CANNOT save intermediate state
  // (1) is because this function is called BEFORE any task is actually run to generate the actual execution plan for a block
  // (2) is because this is meant to be a cheap optimization to skip tasks if they clearly aren't required
  //     Ex: if your task can be skipped if no txs exists in the block, if no metadata exists in the block, etc.
  // (3) is because the cost of storing and passing around intermediate state would be more expensive than recomputing
  should_add_task |_block, _properties| {
    true
  };
  // Specify the function what your task actually does
  // Your task has access to the full block data and any data you specified in either `read` or `write`
  execute |_previous_data, task| handle_dummy(
      task.db_tx,
      task.block,
  );
  // Specify how to merge the result of your task back into the global state
  merge_result |data, _result| {
  };
}

async fn handle_dummy(_db_tx: &DatabaseTransaction, _block: BlockInfo<'_, MultiEraBlock<'_>>) -> Result<(), DbErr> {
    Ok(())
}
