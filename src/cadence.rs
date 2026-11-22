use std::time::Duration;

/// When a throttle may ping for the events that wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cadence {
	/// After a ping, no ping comes for this long.
	pub quiet_time: Duration,
	/// This many waiting events are due for a ping, when the quiet time allows
	/// it. 0 turns this rule off.
	pub batch_size: usize,
	/// Fewer waiting events are due when the oldest of them has waited this
	/// long. [`Duration::MAX`] turns this rule off.
	pub longest_wait: Duration,
}
