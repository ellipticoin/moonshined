use super::RequestJSON;
use serde_json::{json, Value};

pub fn version(_request_json: &RequestJSON) -> Result<Value, i32> {
    Ok(json!("24"))
}
