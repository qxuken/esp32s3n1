use std::time::Duration;

pub fn ns(nanos: u64) -> Duration {
    Duration::from_nanos(nanos)
}
