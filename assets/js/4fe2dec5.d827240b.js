"use strict";(self.webpackChunkmy_website=self.webpackChunkmy_website||[]).push([[2329],{2226:(e,t,a)=>{a.r(t),a.d(t,{assets:()=>l,contentTitle:()=>r,default:()=>p,frontMatter:()=>i,metadata:()=>d,toc:()=>c});var n=a(7462),s=(a(7294),a(3905)),o=a(814);const i={sidebar_position:5},r="Adding your own task",d={unversionedId:"indexer/add_task",id:"indexer/add_task",title:"Adding your own task",description:"If you want to develop a new task, first think about whether or not you want to modify an existing task, or create a new task",source:"@site/docs/indexer/add_task.mdx",sourceDirName:"indexer",slug:"/indexer/add_task",permalink:"/carp/docs/indexer/add_task",draft:!1,editUrl:"https://github.com/dcSpark/carp/docs/indexer/add_task.mdx",tags:[],version:"current",sidebarPosition:5,frontMatter:{sidebar_position:5},sidebar:"tutorialSidebar",previous:{title:"Migrations & Rollbacks",permalink:"/carp/docs/indexer/migrations"},next:{title:"SQL Format",permalink:"/carp/docs/indexer/sql"}},l={},c=[],u={toc:c};function p(e){let{components:t,...a}=e;return(0,s.kt)("wrapper",(0,n.Z)({},u,a,{components:t,mdxType:"MDXLayout"}),(0,s.kt)("h1",{id:"adding-your-own-task"},"Adding your own task"),(0,s.kt)("p",null,"If you want to develop a new task, first think about whether or not you want to modify an existing task, or create a new task"),(0,s.kt)("p",null,"Usually, it's preferable to modify an existing task by providing a new parameter to the task that users can specify in their execution plan.\nThis is because if you provide two variants of the same task (ex: InsertBlock1, InsertBlock2), it makes it harder for child tasks to define their dependencies"),(0,s.kt)("p",null,"If you still want to add a new task, here are the steps to follow:"),(0,s.kt)("ol",null,(0,s.kt)("li",{parentName:"ol"},"(if needed) add a new entity for your schema definition in ",(0,s.kt)("inlineCode",{parentName:"li"},"indexer/entity")),(0,s.kt)("li",{parentName:"ol"},"(if needed) add a new migration to add your new table to ",(0,s.kt)("inlineCode",{parentName:"li"},"indexer/migration")),(0,s.kt)("li",{parentName:"ol"},"Add a new task to ",(0,s.kt)("inlineCode",{parentName:"li"},"indexer/tasks")),(0,s.kt)("li",{parentName:"ol"},"Add the new task to your execution plan in ",(0,s.kt)("inlineCode",{parentName:"li"},"indexer/execution_plan"))),(0,s.kt)("p",null,"Tasks are easiest to write using the ",(0,s.kt)("inlineCode",{parentName:"p"},"carp_task")," ",(0,s.kt)("a",{parentName:"p",href:"https://en.wikipedia.org/wiki/Domain-specific_language"},"DSL"),"."),(0,s.kt)("p",null,"Here is an example task that you can use as reference"),(0,s.kt)(o.Z,{language:"rust",title:"example_task.rs",showLineNumbers:!0,mdxType:"CodeBlock"},"use crate::config::EmptyConfig::EmptyConfig;\nuse crate::dsl::database_task::BlockGlobalInfo;\nuse crate::dsl::task_macro::*;\n\ncarp_task! {\n  // The task name. This is what will show up in the task graph\n  // and this is how you specify dependencies\n  name ExampleTask;\n  configuration EmptyConfig;\n  doc \"An example task to help people learn how to write custom Carp tasks\";\n  // The era your task operates on. Note: different eras have different block representations\n  era multiera;\n  // List of dependencies for this task. This is an array of names of other tasks\n  // Note: your task will run if all dependencies either ran successfully OR were skipped for this block\n  dependencies [];\n  // Specify which fields your task will have read-access to\n  read [multiera_txs];\n  // Specify which fields your task will have write-access to\n  write [multiera_addresses];\n  // Specify whether or not your task needs to run for a given block\n  // Note that by design, this function:\n  // 1) CANNOT access parent task state\n  // 2) Is NOT async\n  // 3) CANNOT save intermediate state\n  // (1) is because this function is called BEFORE any task is actually run to generate the actual execution plan for a block\n  // (2) is because this is meant to be a cheap optimization to skip tasks if they clearly aren't required\n  //     Ex: if your task can be skipped if no txs exists in the block, if no metadata exists in the block, etc.\n  // (3) is because the cost of storing and passing around intermediate state would be more expensive than recomputing\n  should_add_task |_block, _properties| {\n    true\n  };\n  // Specify the function what your task actually does\n  // Your task has access to the full block data and any data you specified in either `read` or `write`\n  execute |_previous_data, task| handle_dummy(\n      task.db_tx,\n      task.block,\n  );\n  // Specify how to merge the result of your task back into the global state\n  merge_result |data, _result| {\n  };\n}\n\nasync fn handle_dummy(\n    _db_tx: &DatabaseTransaction,\n    _block: BlockInfo<'_, cml_multi_era::MultiEraBlock, BlockGlobalInfo>,\n) -> Result<(), DbErr> {\n    Ok(())\n}\n"),(0,s.kt)("p",null,"As you can see, tasks all share access to an execution context that holds which variables you can read and write from. This context usually contains things like database IDs of the recently added data so that it can be properly references from other tables."),(0,s.kt)(o.Z,{language:"rust",title:"execution_context.rs",showLineNumbers:!0,mdxType:"CodeBlock"},"pub use crate::era_common::OutputWithTxData;\npub use entity::{\n    prelude::*,\n    sea_orm::{prelude::*, DatabaseTransaction},\n};\npub use std::collections::BTreeMap;\n\n#[macro_export]\nmacro_rules! data_to_type {\n  // genesis\n  (genesis_block) => { Option<BlockModel> };\n  (genesis_txs) => { Vec<TransactionModel> };\n  (genesis_addresses) => { Vec<AddressModel> };\n  (genesis_outputs) => { Vec<TransactionOutputModel> };\n\n  // byron\n  (byron_block) => { Option<BlockModel> };\n  (byron_txs) => { Vec<TransactionModel> };\n  (byron_addresses) => { BTreeMap<Vec<u8>, AddressInBlock> };\n  (byron_inputs) => { Vec<TransactionInputModel> };\n  (byron_outputs) => { Vec<TransactionOutputModel> };\n\n  // multiera\n  (multiera_block) => { Option<BlockModel> };\n  (multiera_txs) => { Vec<TransactionModel> };\n  (vkey_relation_map) => { RelationMap };\n  (multiera_queued_addresses_relations) => { BTreeSet<QueuedAddressCredentialRelation> };\n  (multiera_stake_credential) => { BTreeMap<Vec<u8>, StakeCredentialModel> };\n  (multiera_addresses) => { BTreeMap<Vec<u8>, AddressInBlock> };\n  (multiera_metadata) => { Vec<TransactionMetadataModel> };\n  (multiera_outputs) => { Vec<TransactionOutputModel> };\n  (multiera_used_inputs) => { Vec<TransactionInputModel> };\n  (multiera_used_inputs_to_outputs_map) => { BTreeMap<Vec<u8>, BTreeMap<i64, OutputWithTxData>> };\n  (multiera_assets) => { Vec<NativeAssetModel> };\n}\n\npub(crate) use data_to_type;\n"))}p.isMDXComponent=!0}}]);