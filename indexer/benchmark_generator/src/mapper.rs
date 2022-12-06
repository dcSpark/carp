use anyhow::{anyhow, Context};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Default, Debug)]
pub struct DataMapper<T: Hash + Eq + Serialize + DeserializeOwned> {
    mapping: HashMap<T, u64>,
    current_mapping_index: u64,
}

impl<T: Hash + Eq + Serialize + DeserializeOwned> DataMapper<T> {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::default(),
            current_mapping_index: 0,
        }
    }

    pub fn add_if_not_presented(&mut self, key: T) -> u64 {
        match self.mapping.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let value = self.current_mapping_index;
                entry.insert(value.clone());
                self.current_mapping_index += 1;
                value
            }
        }
    }

    pub fn get(&self, key: &T) -> Option<u64> {
        self.mapping.get(key).cloned()
    }

    pub fn dump_to_file(&self, path: PathBuf) -> anyhow::Result<()> {
        let mut output = File::create(path)?;
        output.write_all(format!("{}\n", self.mapping.len()).as_bytes())?;
        for (k, v) in self.mapping.iter() {
            output.write_all(format!("{}:{}\n", serde_json::to_string(k)?, v).as_bytes())?;
        }
        Ok(())
    }

    pub fn load_from_file(path: PathBuf) -> anyhow::Result<Self> {
        let mut result = Self::new();

        let reader = BufReader::new(File::open(path)?);
        let mut lines = reader.lines();
        let lines_count = if let Some(count) = lines.next() {
            let count = count?;
            let lines_count: usize = serde_json::from_str(count.as_str())?;
            lines_count
        } else {
            return Err(anyhow!(
                "Can't parse first line: expected to see count of values"
            ));
        };

        let mut read: usize = 0;
        let mut max_index: u64 = 0;
        for (num, line) in lines.enumerate() {
            let unwrapped = line?;
            let mut split = unwrapped.split(":");
            let data: T = if let Some(data) = split.next() {
                serde_json::from_str(data).context(format!("Key at line: {}", num + 2))?
            } else {
                return Err(anyhow!("Can't parse {} line: key corrupted", num + 2));
            };
            let index: u64 = if let Some(data) = split.next() {
                u64::from_str(data).context(format!("Index at line: {}", num + 2))?
            } else {
                return Err(anyhow!("Can't parse {} line: index corrupted", num + 2));
            };

            result.mapping.insert(data, index);

            max_index = max(max_index, index);
            read += 1;
        }
        result.current_mapping_index = max_index + 1;
        if read != lines_count {
            return Err(anyhow!("Data corrupted: lines count mismatch"));
        }
        Ok(result)
    }
}
