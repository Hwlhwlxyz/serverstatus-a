// Use Deserialize to convert e.g. from request JSON into Book struct.
use serde::Deserialize;

// Demo book structure with some example fields for id, title, author.
// A production app could prefer an id to be type u32, UUID, etc.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct UpdateStatus {
    uptime: i64,
    load_1: i64,
    load_5: i64,
    load_15: i64,
    memory_total: i64,
    memory_used: i64,
    swap_total: i64,
    swap_used: i64,
    hdd_total: i64,
    hdd_used: i64,
    cpu: f64,
    network_rx: i64,
    network_tx: i64,
    network_in: i64,
    network_out: i64,
    ip_status: bool,
    ping_10010: i64,
    ping_189: i64,
    ping_10086: i64,
    time_10010: i64,
    time_189: i64,
    time_10086: i64,
    tcp: i64,
    udp: i64,
    process: i64,
    thread: i64,
    io_read: i64,
    io_write: i64,
}

// Display the book using the format "{title} by {author}".
// This is a typical Rust trait and is not axum-specific.
impl std::fmt::Display for UpdateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} by {}", self.uptime, self.load_1 )
    }
}