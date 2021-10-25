pub type Error = (i32, String);

lazy_static! {
    pub static ref PARSE_ERROR: Error = (-32700, "Parse Error".to_string());
    pub static ref INVALID_SENDER: Error = (-32000, "Invalid Sender".to_string());
    pub static ref SMART_CONTACT_ERROR: i32 = -32001;
}
pub type Result<T> = std::result::Result<T, Error>;
