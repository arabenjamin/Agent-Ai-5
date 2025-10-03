use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginCallParams {
    pub name: String,
    pub action: String,
    pub args: HashMap<String, Value>,
}