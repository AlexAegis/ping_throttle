use std::time::Instant;

use crate::{Cadence, Ping};

#[derive(Debug)]
struct WaitingEvent<K> {
	key: K,
	since: Instant,
}

/// Collects events and says when one ping for them is due. It reads no clock:
/// the caller passes the time of each call.
///
/// While the oldest waiting event is one that arrived outside a quiet time, a
/// ping is due at once. Other events are due as the [`Cadence`] describes.
#[derive(Debug)]
pub struct Throttle<K> {
	cadence: Cadence,
	last_ping: Option<Instant>,
	waiting: Vec<WaitingEvent<K>>,
}

impl<K: PartialEq> Throttle<K> {
	#[must_use]
	pub fn new(cadence: Cadence) -> Self {
		Self {
			cadence,
			last_ping: None,
			waiting: Vec::new(),
		}
	}

	/// Adds an event that waits for a ping. An event that waits already keeps
	/// its arrival time.
	pub fn arrive(&mut self, key: K, now: Instant) {
		if self.waiting.iter().any(|event| event.key == key) {
			return;
		}
		self.waiting.push(WaitingEvent { key, since: now });
	}

	/// Removes an event that needs no ping any more, such as an answered one.
	pub fn leave(&mut self, key: &K) {
		self.waiting.retain(|event| event.key != *key);
	}

	/// The time from which a ping is due. `None` while no ping can come for
	/// the waiting events: none wait, each rule that would give a time is off,
	/// or a time is too far away for an [`Instant`].
	#[must_use]
	pub fn due(&self) -> Option<Instant> {
		let oldest = self.waiting.first()?;
		let quiet_until = match self.last_ping {
			None => None,
			Some(last) => Some(last.checked_add(self.cadence.quiet_time)?),
		};
		let by_cadence = if quiet_until.is_none_or(|quiet_until| oldest.since >= quiet_until) {
			oldest.since
		} else {
			let by_wait = oldest.since.checked_add(self.cadence.longest_wait);
			let by_batch = self
				.cadence
				.batch_size
				.checked_sub(1)
				.and_then(|last_of_batch| self.waiting.get(last_of_batch))
				.map(|event| event.since);
			match (by_batch, by_wait) {
				(Some(by_batch), Some(by_wait)) => by_batch.min(by_wait),
				(by_batch, by_wait) => by_batch.or(by_wait)?,
			}
		};
		Some(quiet_until.map_or(by_cadence, |quiet_until| by_cadence.max(quiet_until)))
	}

	/// The ping for all waiting events when one is due at `now`. The events of
	/// a ping wait no longer, and the quiet time starts at `now`.
	pub fn take_ping(&mut self, now: Instant) -> Option<Ping<K>> {
		if self.due()? > now {
			return None;
		}
		self.last_ping = Some(now);
		Some(Ping {
			keys: self.waiting.drain(..).map(|event| event.key).collect(),
			at: now,
		})
	}
}
