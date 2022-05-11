---
sidebar_position: 5
---

# Adding your own task

If you want to develop a new task, here are the steps to follow:

1. (if needed) add a new entity for your schema definition in `indexer/entity`
2. (if needed) add a new migration to add your new table to `indexer/migration`
3. Add a new task to `indexer/tasks`
4. Add the new task to your execution plan in `indexer/execution_plan`
