#[uniffi::export]
pub fn chat_core_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
