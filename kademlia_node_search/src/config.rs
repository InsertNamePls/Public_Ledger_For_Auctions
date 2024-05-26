//----------------------------------- KADEMLIA CONFIGURATION (BEGIN) --------------------------
//Upper limit for random routing table refresh
pub const REFRESH_TIMER_UPPER: u64 = 20;
//Lower limit for random routing table refresh
pub const REFRESH_TIMER_LOWER: u64 = 5;
//Upper limit for random ping refresh
pub const PING_TIMER_UPPER: u64 = 20;
//Lower limit for random ping refresh
pub const PING_TIMER_LOWER: u64 = 5;
//Number of nodes to ping, for random ping refresh
pub const N: usize = 5; 
//Timeout in seconds for each request
pub const TIMEOUT_TIMER: u64 = 100;
//Maximum number of attempts for each request for timeout
pub const TIMEOUT_MAX_ATTEMPTS: u64 = 3;
//----------------------------------- KADEMLIA CONFIGURATION (END) ----------------------------

//----------------------------------- ROUTING TABLE CONFIGURATION (BEGIN) ---------------------
// The maximum number of nodes in a bucket
pub const K: usize = 20; 
// The number of bits in the node ID
pub const N_BITS: usize = 160; 
//----------------------------------- ROUTING TABLE CONFIGURATION (End) -----------------------


//----------------------------------- NODE GENERATION (BEGIN) ---------------------------------
//Leading zero bits for node ID generation
pub const C1: u32 = 1;
//Number of attempts it takes to log elapsed time for node generation
pub const LOG_INTERVAL: u64 = 10_000;
//----------------------------------- NODE GENERATION (END) -----------------------------------


//----------------------------------- ATTACK PREVENTION (BEGIN) -------------------------------
// Replay attack prevention time window in seconds
pub const REPLAY_WINDOW: i64 = 120;
// The maximum number of nodes per IP address
pub const MAX_NODES_PER_IP: usize = 5;
// The reputation threshold for a node to be removed from the routing table
pub const REPUTATION_THRESHOLD: i32 = -5;
//----------------------------------- ATTACK PREVENTION (END) ---------------------------------