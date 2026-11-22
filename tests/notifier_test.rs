use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ping_throttle::{Cadence, Notifier};

const CADENCE: Cadence = Cadence {
	quiet_time: Duration::from_secs(30),
	batch_size: 4,
	longest_wait: Duration::from_secs(120),
};

fn seconds(count: u64) -> Duration {
	Duration::from_secs(count)
}

#[test]
fn should_hand_each_ping_to_every_subscriber_one_time() {
	let start = Instant::now();
	let pings = Arc::new(Mutex::new(Vec::new()));
	let mut notifier = Notifier::new(CADENCE);
	for name in ["one", "two"] {
		let pings = Arc::clone(&pings);
		notifier.subscribe(move |ping| pings.lock().unwrap().push((name, ping.keys.clone())));
	}
	assert_eq!(notifier.poll(start), None);
	notifier.arrive("a", start);
	assert_eq!(notifier.poll(start), None);
	assert_eq!(notifier.poll(start + seconds(1)), None);
	assert_eq!(
		*pings.lock().unwrap(),
		vec![("one", vec!["a"]), ("two", vec!["a"])]
	);
}

#[test]
fn should_return_the_time_of_the_next_ping_while_events_wait() {
	let start = Instant::now();
	let pings = Arc::new(Mutex::new(Vec::new()));
	let mut notifier = Notifier::new(CADENCE);
	let seen = Arc::clone(&pings);
	notifier.subscribe(move |ping| seen.lock().unwrap().push(ping.keys.clone()));
	notifier.arrive("a", start);
	notifier.poll(start);
	notifier.arrive("b", start + seconds(5));
	assert_eq!(
		notifier.poll(start + seconds(6)),
		Some(start + seconds(125))
	);
	notifier.leave(&"b");
	assert_eq!(notifier.poll(start + seconds(7)), None);
	notifier.arrive("c", start + seconds(8));
	assert_eq!(notifier.poll(start + seconds(128)), None);
	assert_eq!(*pings.lock().unwrap(), vec![vec!["a"], vec!["c"]]);
}

#[test]
fn should_move_to_another_thread() {
	let start = Instant::now();
	let pings = Arc::new(Mutex::new(Vec::new()));
	let mut notifier = Notifier::new(CADENCE);
	let seen = Arc::clone(&pings);
	notifier.subscribe(move |ping| seen.lock().unwrap().push(ping.keys.clone()));
	std::thread::spawn(move || {
		notifier.arrive("a", start);
		notifier.poll(start);
	})
	.join()
	.unwrap();
	assert_eq!(*pings.lock().unwrap(), vec![vec!["a"]]);
}

#[test]
fn should_show_the_count_of_its_subscribers_in_debug_text() {
	let mut notifier = Notifier::<&str>::new(CADENCE);
	notifier.subscribe(|_| {});
	assert!(format!("{notifier:?}").contains("subscriber_count: 1"));
}

#[test]
fn should_not_ping_for_a_new_cadence_alone() {
	let start = Instant::now();
	let pings = Arc::new(Mutex::new(Vec::new()));
	let mut notifier = Notifier::new(CADENCE);
	let seen = Arc::clone(&pings);
	notifier.subscribe(move |ping| seen.lock().unwrap().push(ping.keys.clone()));
	notifier.arrive("a", start);
	notifier.poll(start);
	notifier.arrive("b", start + seconds(5));
	notifier.set_cadence(Cadence {
		quiet_time: seconds(120),
		batch_size: 8,
		longest_wait: seconds(600),
	});
	assert_eq!(
		notifier.poll(start + seconds(6)),
		Some(start + seconds(605))
	);
	assert_eq!(*pings.lock().unwrap(), vec![vec!["a"]]);
}
