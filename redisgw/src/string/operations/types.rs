use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SetFlags {
    pub method: Option<SetMethod>,
    pub ttl: Option<SetTTL>,
    pub get: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetTTL {
    /// Set expiry in seconds.
    Ex(u64),
    /// Set expiry in milliseconds.
    Px(u64),
    /// Set expiry at a specific unix time in seconds.
    ExAt(u64),
    /// Set expiry at a specific unix time in milliseconds.
    PxAt(u64),
    /// Keep the existing TTL.
    KeepTTL,
}

impl SetTTL {
    pub fn unix_epoch_in_ms(&self) -> Result<u128, String> {
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(val) => val,
            Err(e) => return Err(format!("SystemTime error: {:?}", e)),
        };

        match self {
            SetTTL::Ex(secs) => Ok((now + Duration::from_secs(*secs)).as_millis()),
            SetTTL::Px(ms) => Ok((now + Duration::from_millis(*ms)).as_millis()),
            SetTTL::ExAt(timestamp) => Ok(Duration::from_secs(*timestamp).as_millis()),
            SetTTL::PxAt(timestamp) => Ok(Duration::from_millis(*timestamp).as_millis()),
            SetTTL::KeepTTL => Ok(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetMethod {
    /// Only set the key if it does not already exist.
    NX,
    /// Only set the key if it already exists.
    XX,
}
