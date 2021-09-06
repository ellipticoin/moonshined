use async_std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LATEST_BLOCK: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}
