use std::{fs::File, path::Path};

use markdown_gen::markdown::Markdown;

use tasks::dsl::markdown_task::TaskMarkdownRegistryEntry;

use tracing_subscriber::prelude::*;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Output directory
    #[clap(short, long)]
    output: String,
}

fn main() -> anyhow::Result<()> {
    // Start logging setup block
    let fmt_layer = tracing_subscriber::fmt::layer().with_test_writer();

    let sqlx_filter = tracing_subscriber::filter::Targets::new()
        .with_default(tracing_subscriber::fmt::Subscriber::DEFAULT_MAX_LEVEL);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(sqlx_filter)
        .init();
    // End logging setup block

    let args = Args::parse();

    let get_md = |name| {
        let base_path = Path::new(&args.output);
        let full_path = base_path.join(format!("{}.md", name));
        let file = File::create(full_path).unwrap();
        Markdown::new(file)
    };
    for registry_entry in inventory::iter::<TaskMarkdownRegistryEntry> {
        match &registry_entry {
            TaskMarkdownRegistryEntry::Byron(entry) => {
                let mut md = get_md(entry.builder.get_name());
                entry.builder.generate_docs(&mut md);
            }
            TaskMarkdownRegistryEntry::Multiera(entry) => {
                let mut md = get_md(entry.builder.get_name());
                entry.builder.generate_docs(&mut md);
            }
            TaskMarkdownRegistryEntry::Genesis(entry) => {
                let mut md = get_md(entry.builder.get_name());
                entry.builder.generate_docs(&mut md);
            }
        }
    }

    {
        use markdown_gen::markdown::AsMarkdown;
        let mut index_md = get_md("index");
        index_md.write("Tasks".heading(1)).unwrap();
        index_md
            .write("List of all tasks that can be used inside execution plans".paragraph())
            .unwrap();
    }

    Ok(())
}
