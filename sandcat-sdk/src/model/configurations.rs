#[cfg(debug_assertions)]
pub const STUN_SERVER: &str = "stun:localhost:3478";

#[cfg(not(debug_assertions))]
pub const STUN_SERVER: &str = "stun:localhost:3478";
