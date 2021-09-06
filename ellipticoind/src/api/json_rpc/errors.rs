type Error = i32;
pub const PARSE_ERROR: Error = -32700;
pub const INVALID_SENDER: Error = -32000;
pub type Result<T> = std::result::Result<T, Error>;

pub fn error_message<'a>(error_id: Error) -> &'a str {
    match error_id {
        PARSE_ERROR => "Parse Error",
        INVALID_SENDER => "invalid sender",
        _ => panic!("error not found"),
    }
}
