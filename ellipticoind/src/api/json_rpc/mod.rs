mod encoders;
mod errors;
mod eth;
mod helpers;
mod net;
mod parsers;
mod transaction;

use errors::Error;
use serde::Deserialize;
use serde_json::{json, Value};
use tide::Request;

#[derive(Deserialize, Clone)]
pub struct RequestJSON {
    id: Value,
    method: String,
    params: Value,
}

pub async fn handle_json_rpc(mut request: Request<()>) -> tide::Result {
    let request_json: RequestJSON = request.body_json().await?;
    let result = match request_json.method.as_ref() {
        "eth_blockNumber" => eth::block_number(&request_json.params).await,
        "eth_call" => eth::call(&request_json.params).await,
        "eth_chainId" => eth::chain_id(&request_json.params),
        "eth_estimateGas" => eth::estimate_gas(&request_json.params),
        "eth_gasPrice" => eth::gas_price(&request_json.params).await,
        "eth_getBalance" => eth::get_balance(&request_json.params).await,
        "eth_getBlockByHash" => eth::get_block_by_hash(&request_json.params).await,
        "eth_getBlockByNumber" => eth::get_block_by_number(&request_json.params).await,
        "eth_getCode" => eth::get_code(&request_json.params).await,
        "eth_getTransactionCount" => eth::get_transaction_count(&request_json.params).await,
        "eth_getTransactionReceipt" => eth::get_transaction_receipt(&request_json.params).await,
        "eth_sendRawTransaction" => eth::send_raw_transaction(&request_json.params).await,
        "net_version" => net::version(&request_json),
        _ => return unsupported_method(&request_json),
    };

    match result {
        Err(err) => error_response(&request_json.id, err),
        Ok(result) => ok_response(&request_json.id, &result),
    }
}

pub fn unsupported_method(request_json: &RequestJSON) -> tide::Result {
    Ok(
json!({"jsonrpc": "2.0", "id": request_json.id, "error": {"code": -32601, "message": format!("Unsupported method [\"{}\"]", request_json.method)}}).into())
}

pub fn error_response(id: &Value, error: Error) -> tide::Result {
    Ok(json!(
    {"jsonrpc":"2.0","id": id,"error":{"code":error.0,"message":error.1}}
        )
    .into())
}

pub fn ok_response(id: &Value, result: &Value) -> tide::Result {
    Ok(json!(
    {"jsonrpc": "2.0", "id": id, "result": result}
    )
    .into())
}
