use std::fs::File;

use markdown_gen::markdown::Markdown;

#[derive(Copy, Clone)]
pub enum TaskMarkdownRegistryEntry {
    Genesis(GenesisTaskMarkdownRegistryEntry),
    Byron(ByronTaskMarkdownRegistryEntry),
    Multiera(MultieraTaskMarkdownRegistryEntry),
}

pub trait MarkdownTaskMeta {
    const TASK_NAME: &'static str;
    const DOC: &'static str;
    const ERA: &'static str;
    const READ_FROM: &'static [&'static str];
    const WRITE_TO: &'static [&'static str];
    const DEPENDENCIES: &'static [&'static str];
}

pub trait TaskMarkdownBuilder {
    fn get_name(&self) -> &'static str;
    fn get_doc(&self) -> &'static str;
    fn get_era(&self) -> &'static str;
    fn get_reads(&self) -> &'static [&'static str];
    fn get_writes(&self) -> &'static [&'static str];
    fn get_dependencies(&self) -> &'static [&'static str];

    fn generate_docs(&self, file: &mut Markdown<File>);
}

#[derive(Copy, Clone)]
pub struct GenesisTaskMarkdownRegistryEntry {
    pub builder: &'static (dyn TaskMarkdownBuilder + Sync),
}

#[derive(Copy, Clone)]
pub struct ByronTaskMarkdownRegistryEntry {
    pub builder: &'static (dyn TaskMarkdownBuilder + Sync),
}

#[derive(Copy, Clone)]
pub struct MultieraTaskMarkdownRegistryEntry {
    pub builder: &'static (dyn TaskMarkdownBuilder + Sync),
}

inventory::collect!(TaskMarkdownRegistryEntry);
