use super::errors::Result;
use super::RequestJSON;
use serde_json::{json, Value};

pub fn version(_request_json: &RequestJSON) -> Result<Value> {
    Ok(json!("24"))
}
