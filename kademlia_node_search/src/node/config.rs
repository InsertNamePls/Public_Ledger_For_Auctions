//Upper limit for random routing table refresh
pub const REFRESH_TIMER_UPPER: u64 = 20;
//Lower limit for random routing table refresh
pub const REFRESH_TIMER_LOWER: u64 = 5;
//Timeout in seconds for each request
pub const TIMEOUT_TIMER: u64 = 3;
//Maximum number of attempts for each request for timeout
pub const TIMEOUT_MAX_ATTEMPTS: u64 = 3;
//Leading zero bits for node ID generation
pub const C1: u32 = 14;
//Number of attempts it takes to log elapsed time for node generation
pub const LOG_INTERVAL: u64 = 10_000;
// Replay attack prevention time window in seconds
pub const REPLAY_WINDOW: i64 = 120;