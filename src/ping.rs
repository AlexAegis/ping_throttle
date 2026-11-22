use std::time::Instant;

/// One notification for all events that waited when it came.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ping<K> {
	/// The events of the ping, the oldest first.
	pub keys: Vec<K>,
	pub at: Instant,
}
