use crate::types::{MicroReceipt, SlabReceipt, NodeContext};
use std::fs::File;
use std::io::BufReader;

pub fn load_micro_vector(path: &str) -> anyhow::Result<(NodeContext, MicroReceipt)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: serde_json::Value = serde_json::from_reader(reader)?;
    let ctx = serde_json::from_value(data["context"].clone())?;
    let receipt = serde_json::from_value(data["receipt"].clone())?;
    Ok((ctx, receipt))
}

pub fn load_slab_vector(path: &str) -> anyhow::Result<(NodeContext, SlabReceipt)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: serde_json::Value = serde_json::from_reader(reader)?;
    let ctx = serde_json::from_value(data["context"].clone())?;
    let slab = serde_json::from_value(data["slab"].clone())?;
    Ok((ctx, slab))
}
