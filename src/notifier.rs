use std::{fmt, time::Instant};

use crate::{Cadence, Ping, Throttle};

type Subscriber<K> = Box<dyn FnMut(&Ping<K>) + Send>;

/// A [`Throttle`] that hands each ping to its subscribers.
pub struct Notifier<K> {
	throttle: Throttle<K>,
	subscribers: Vec<Subscriber<K>>,
}

impl<K: fmt::Debug> fmt::Debug for Notifier<K> {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter
			.debug_struct("Notifier")
			.field("throttle", &self.throttle)
			.field("subscriber_count", &self.subscribers.len())
			.finish()
	}
}

impl<K: PartialEq> Notifier<K> {
	#[must_use]
	pub fn new(cadence: Cadence) -> Self {
		Self {
			throttle: Throttle::new(cadence),
			subscribers: Vec::new(),
		}
	}

	pub fn subscribe(&mut self, subscriber: impl FnMut(&Ping<K>) + Send + 'static) {
		self.subscribers.push(Box::new(subscriber));
	}

	/// Replaces the cadence, as [`Throttle::set_cadence`] describes.
	pub fn set_cadence(&mut self, cadence: Cadence) {
		self.throttle.set_cadence(cadence);
	}

	pub fn arrive(&mut self, key: K, now: Instant) {
		self.throttle.arrive(key, now);
	}

	pub fn leave(&mut self, key: &K) {
		self.throttle.leave(key);
	}

	/// Hands a ping that is due at `now` to each subscriber. Returns the time
	/// of the next call that can ping, or `None` while the waiting events get no
	/// ping at any time, as [`Throttle::due`] describes.
	pub fn poll(&mut self, now: Instant) -> Option<Instant> {
		if let Some(ping) = self.throttle.take_ping(now) {
			for subscriber in &mut self.subscribers {
				subscriber(&ping);
			}
		}
		self.throttle.due()
	}
}
