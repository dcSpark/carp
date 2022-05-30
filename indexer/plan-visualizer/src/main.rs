use std::{path::Path, process::Command};

use tasks::execution_plan::ExecutionPlan;
use tracing_subscriber::prelude::*;

use clap::Parser;

use crate::generate_image::generate;
mod generate_image;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Path of the execution plan to use
    #[clap(short, long, default_value = "execution_plans/default.toml")]
    plan: String,

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

    tracing::info!("Execution plan {}", args.plan);
    let exec_plan = ExecutionPlan::load_execution_plan(&args.plan);
    let plan_name = Path::new(&args.plan).file_stem().unwrap().to_str().unwrap();

    let graph = generate(&exec_plan, plan_name);

    use std::fs::File;

    let base_path = Path::new(&args.output);
    let dot_file = base_path.join(format!("{}.dot", plan_name));
    let svg_file = base_path.join(format!("{}.svg", plan_name));

    let mut output = File::create(&dot_file).unwrap();
    dot::render(&graph, &mut output).unwrap();
    Command::new("dot")
        .args([
            "-Tsvg",
            dot_file.to_str().unwrap(),
            "-o",
            svg_file.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute process");

    Ok(())
}
