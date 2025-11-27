/// Logs a confession event for auditing purposes.
/// The log includes the hash and the content.
pub fn log_confession(hash: &str, content: &str) {
    log::warn!(
        "Confession received: {} | {}",
        hash,
        content.replace('\n', " \\n ")
    );
}
