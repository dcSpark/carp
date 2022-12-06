use anyhow::{anyhow, Context};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub fn dump_hashmap_to_file<Key: Eq + Hash + Serialize, Value: Serialize>(
    hashmap: &HashMap<Key, Value>,
    path: PathBuf,
) -> anyhow::Result<()> {
    let mut out_file = File::create(path)?;
    out_file.write_all(format!("{}\n", hashmap.len()).as_bytes())?;
    for (key, value) in hashmap.iter() {
        out_file.write_all(
            format!(
                "{}:{}\n",
                serde_json::to_string(key)?,
                serde_json::to_string(value)?
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}

pub fn dump_hashset_to_file<Value: Eq + Hash + Serialize>(
    hashmap: &HashSet<Value>,
    path: PathBuf,
) -> anyhow::Result<()> {
    let mut out_file = File::create(path)?;
    out_file.write_all(format!("{}\n", hashmap.len()).as_bytes())?;
    for value in hashmap.iter() {
        out_file.write_all(format!("{}\n", serde_json::to_string(value)?).as_bytes())?;
    }
    Ok(())
}

pub fn read_hashmap_from_file<Key: Eq + Hash + DeserializeOwned, Value: DeserializeOwned>(
    path: PathBuf,
) -> anyhow::Result<HashMap<Key, Value>> {
    let mut result = HashMap::<Key, Value>::new();

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
    for (num, line) in lines.enumerate() {
        let unwrapped = line?;
        let mut split = unwrapped.split(":");
        let key: Key = if let Some(data) = split.next() {
            serde_json::from_str(data).context(format!("Key at line: {}", num + 2))?
        } else {
            return Err(anyhow!("Can't parse {} line: key corrupted", num + 2));
        };
        let value: Value = if let Some(data) = split.next() {
            serde_json::from_str(data).context(format!("Index at line: {}", num + 2))?
        } else {
            return Err(anyhow!("Can't parse {} line: index corrupted", num + 2));
        };

        result.insert(key, value);

        read += 1;
    }
    if read != lines_count {
        return Err(anyhow!("Data corrupted: lines count mismatch"));
    }
    Ok(result)
}

pub fn read_hashset_from_file<Value: Eq + Hash + DeserializeOwned>(
    path: PathBuf,
) -> anyhow::Result<HashSet<Value>> {
    let mut result = HashSet::<Value>::new();

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
    for (num, line) in lines.enumerate() {
        let unwrapped = line?;
        let value: Value = serde_json::from_str(unwrapped.as_str())
            .context(format!("Index at line: {}", num + 2))?;
        result.insert(value);
        read += 1;
    }
    if read != lines_count {
        return Err(anyhow!("Data corrupted: lines count mismatch"));
    }
    Ok(result)
}
