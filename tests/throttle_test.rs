use std::time::{Duration, Instant};

use ping_throttle::{Cadence, Ping, Throttle};

const CADENCE: Cadence = Cadence {
	quiet_time: Duration::from_secs(30),
	batch_size: 4,
	longest_wait: Duration::from_secs(120),
};

fn seconds(count: u64) -> Duration {
	Duration::from_secs(count)
}

fn pinged_throttle(start: Instant) -> Throttle<&'static str> {
	let mut throttle = Throttle::new(CADENCE);
	throttle.arrive("first", start);
	assert!(throttle.take_ping(start).is_some());
	throttle
}

#[test]
fn should_be_due_at_once_for_the_first_event() {
	let start = Instant::now();
	let mut throttle = Throttle::new(CADENCE);
	assert_eq!(throttle.due(), None);
	throttle.arrive("a", start);
	assert_eq!(throttle.due(), Some(start));
	assert_eq!(
		throttle.take_ping(start),
		Some(Ping {
			keys: vec!["a"],
			at: start
		})
	);
	assert_eq!(throttle.due(), None);
	assert_eq!(throttle.take_ping(start + seconds(500)), None);
}

#[test]
fn should_wait_the_longest_wait_for_an_event_that_arrives_in_the_quiet_time() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	throttle.arrive("b", start + seconds(10));
	assert_eq!(throttle.due(), Some(start + seconds(130)));
	assert_eq!(throttle.take_ping(start + seconds(129)), None);
	assert_eq!(
		throttle
			.take_ping(start + seconds(130))
			.map(|ping| ping.keys),
		Some(vec!["b"])
	);
}

#[test]
fn should_be_due_at_once_for_an_event_that_arrives_after_the_quiet_time_while_nothing_waits() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	throttle.arrive("b", start + seconds(30));
	assert_eq!(throttle.due(), Some(start + seconds(30)));
	throttle.arrive("c", start + seconds(31));
	assert_eq!(throttle.due(), Some(start + seconds(30)));
}

#[test]
fn should_be_due_when_a_batch_waits_but_not_inside_the_quiet_time() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	for (key, second) in [("b", 1), ("c", 2), ("d", 3)] {
		throttle.arrive(key, start + seconds(second));
	}
	assert_eq!(throttle.due(), Some(start + seconds(121)));
	throttle.arrive("e", start + seconds(4));
	assert_eq!(throttle.due(), Some(start + seconds(30)));
	assert_eq!(throttle.take_ping(start + seconds(29)), None);
	assert_eq!(
		throttle
			.take_ping(start + seconds(30))
			.map(|ping| ping.keys),
		Some(vec!["b", "c", "d", "e"])
	);
}

#[test]
fn should_be_due_at_the_arrival_of_the_last_event_of_a_batch_after_the_quiet_time() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	for (key, second) in [("b", 10), ("c", 40), ("d", 50)] {
		throttle.arrive(key, start + seconds(second));
	}
	assert_eq!(throttle.due(), Some(start + seconds(130)));
	throttle.arrive("e", start + seconds(60));
	assert_eq!(throttle.due(), Some(start + seconds(60)));
}

#[test]
fn should_not_count_an_event_that_left() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	for (key, second) in [("b", 1), ("c", 2), ("d", 3), ("e", 4)] {
		throttle.arrive(key, start + seconds(second));
	}
	throttle.leave(&"b");
	assert_eq!(throttle.due(), Some(start + seconds(122)));
	for key in ["c", "d", "e"] {
		throttle.leave(&key);
	}
	assert_eq!(throttle.due(), None);
	assert_eq!(throttle.take_ping(start + seconds(500)), None);
}

#[test]
fn should_keep_the_arrival_time_of_an_event_that_arrives_again() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	throttle.arrive("b", start + seconds(5));
	throttle.arrive("b", start + seconds(50));
	assert_eq!(throttle.due(), Some(start + seconds(125)));
	assert_eq!(
		throttle
			.take_ping(start + seconds(125))
			.map(|ping| ping.keys),
		Some(vec!["b"])
	);
}

#[test]
fn should_start_a_new_quiet_time_at_each_ping() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	throttle.arrive("b", start + seconds(40));
	assert!(throttle.take_ping(start + seconds(45)).is_some());
	throttle.arrive("c", start + seconds(50));
	assert_eq!(throttle.due(), Some(start + seconds(170)));
	throttle.leave(&"c");
	throttle.arrive("d", start + seconds(75));
	assert_eq!(throttle.due(), Some(start + seconds(75)));
}

#[test]
fn should_wait_the_longest_wait_with_a_batch_size_of_zero() {
	let start = Instant::now();
	let mut throttle = Throttle::new(Cadence {
		batch_size: 0,
		..CADENCE
	});
	throttle.arrive("a", start);
	assert!(throttle.take_ping(start).is_some());
	for (key, second) in [("b", 1), ("c", 2), ("d", 3), ("e", 4), ("f", 5)] {
		throttle.arrive(key, start + seconds(second));
	}
	assert_eq!(throttle.due(), Some(start + seconds(121)));
}

#[test]
fn should_stay_due_from_the_longest_wait_when_the_batch_fills_later() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	for (key, second) in [("b", 10), ("c", 140), ("d", 150), ("e", 160)] {
		throttle.arrive(key, start + seconds(second));
	}
	assert_eq!(throttle.due(), Some(start + seconds(130)));
}

#[test]
fn should_stay_due_at_once_when_the_oldest_of_the_events_after_the_quiet_time_leaves() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	throttle.arrive("b", start + seconds(100));
	throttle.arrive("c", start + seconds(101));
	throttle.leave(&"b");
	assert_eq!(throttle.due(), Some(start + seconds(101)));
}

#[test]
fn should_start_the_quiet_time_at_a_late_ping() {
	let start = Instant::now();
	let mut throttle = Throttle::new(CADENCE);
	throttle.arrive("a", start);
	assert_eq!(
		throttle.take_ping(start + seconds(10)),
		Some(Ping {
			keys: vec!["a"],
			at: start + seconds(10)
		})
	);
	throttle.arrive("b", start + seconds(35));
	assert_eq!(throttle.due(), Some(start + seconds(155)));
}

#[test]
fn should_be_due_at_the_end_of_the_quiet_time_with_a_batch_size_of_one() {
	let start = Instant::now();
	let mut throttle = Throttle::new(Cadence {
		batch_size: 1,
		..CADENCE
	});
	throttle.arrive("a", start);
	assert!(throttle.take_ping(start).is_some());
	throttle.arrive("b", start + seconds(5));
	assert_eq!(throttle.due(), Some(start + seconds(30)));
}

#[test]
fn should_wait_for_a_batch_with_the_longest_possible_wait() {
	let start = Instant::now();
	let mut throttle = Throttle::new(Cadence {
		longest_wait: Duration::MAX,
		..CADENCE
	});
	throttle.arrive("a", start);
	assert!(throttle.take_ping(start).is_some());
	for (key, second) in [("b", 1), ("c", 2), ("d", 3)] {
		throttle.arrive(key, start + seconds(second));
	}
	assert_eq!(throttle.due(), None);
	assert_eq!(throttle.take_ping(start + seconds(500)), None);
	throttle.arrive("e", start + seconds(40));
	assert_eq!(throttle.due(), Some(start + seconds(40)));
}

#[test]
fn should_ping_one_time_with_the_longest_possible_quiet_time() {
	let start = Instant::now();
	let mut throttle = Throttle::new(Cadence {
		quiet_time: Duration::MAX,
		..CADENCE
	});
	throttle.arrive("a", start);
	assert!(throttle.take_ping(start).is_some());
	throttle.arrive("b", start + seconds(1));
	assert_eq!(throttle.due(), None);
	assert_eq!(throttle.take_ping(start + seconds(500)), None);
}

#[test]
fn should_be_due_at_once_when_the_event_of_the_quiet_time_leaves_before_a_later_one() {
	let start = Instant::now();
	let mut throttle = pinged_throttle(start);
	throttle.arrive("b", start + seconds(10));
	throttle.arrive("c", start + seconds(100));
	assert_eq!(throttle.due(), Some(start + seconds(130)));
	throttle.leave(&"b");
	assert_eq!(throttle.due(), Some(start + seconds(100)));
}

#[test]
fn should_never_be_due_for_an_event_of_the_quiet_time_with_both_rules_off() {
	let start = Instant::now();
	let mut throttle = Throttle::new(Cadence {
		batch_size: 0,
		longest_wait: Duration::MAX,
		..CADENCE
	});
	throttle.arrive("a", start);
	assert!(throttle.take_ping(start).is_some());
	throttle.arrive("b", start + seconds(10));
	throttle.arrive("c", start + seconds(1000));
	assert_eq!(throttle.due(), None);
	throttle.leave(&"b");
	assert_eq!(throttle.due(), Some(start + seconds(1000)));
}
