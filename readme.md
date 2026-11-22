# [ping_throttle](https://github.com/AlexAegis/ping_throttle)

[![ci](https://github.com/AlexAegis/ping_throttle/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexAegis/ping_throttle/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/AlexAegis/ping_throttle?tab=MIT-1-ov-file)

Collects events and notifies subscribers at a cadence: one ping for many
events, not one ping for each.

The crate reads no clock and starts no timer. The caller passes the time of
each call and sleeps until the time that the crate returns. The times of later
calls must not go back. The crate has no dependencies.

## Cadence

- While the oldest waiting event is one that arrived outside a quiet time:
  ping at once. That is the case for an event that arrives while no other
  event waits and no quiet time runs.
- After each ping: no ping for the `quiet_time`.
- After the quiet time: a ping when `batch_size` events wait, or, with fewer,
  when the oldest waiting event has waited the `longest_wait`.
- An event that leaves before its ping, such as an answered request, waits no
  longer and counts for no ping.
- One ping contains every event that waits at that time.

A `batch_size` of 0 turns the batch rule off. A `longest_wait` of
`Duration::MAX` turns the wait rule off. With both rules off, an event that
arrives in a quiet time gets no ping while it is the oldest.

## Use

The crate is not on crates.io. Add it as a Git dependency:

```toml
[dependencies]
ping_throttle = { git = "https://github.com/AlexAegis/ping_throttle" }
```

```rust
use std::time::{Duration, Instant};

use ping_throttle::{Cadence, Notifier};

let mut notifier = Notifier::new(Cadence {
	quiet_time: Duration::from_secs(30),
	batch_size: 4,
	longest_wait: Duration::from_secs(120),
});
notifier.subscribe(|ping| println!("{} requests wait", ping.keys.len()));

let start = Instant::now();
notifier.arrive("request 1", start);
assert_eq!(notifier.poll(start), None);

notifier.arrive("request 2", start + Duration::from_secs(5));
let next_poll = notifier.poll(start + Duration::from_secs(5));
assert_eq!(next_poll, Some(start + Duration::from_secs(125)));
```

- `Notifier`: `arrive` and `leave` change the waiting events. `poll` hands a
  due ping to each subscriber and returns the time of the next call that can
  ping. Call `poll` after each change and at the time that the last call
  returned. A subscriber must be `Send`, so that the notifier can move to
  another thread.
- `Throttle`: the same cadence without subscribers. `due` returns the time
  from which a ping is due. `take_ping` returns the ping when it is due.

`Notifier::set_cadence` and `Throttle::set_cadence` replace the cadence. The
waiting events keep their arrival times and the quiet time keeps its start. The
new cadence counts from the last ping. Call `poll` after `set_cadence` too: the
time of the next ping can move.

An event is a key of the caller's type, such as an ID. A key that arrives again
while it waits keeps its first arrival time.
