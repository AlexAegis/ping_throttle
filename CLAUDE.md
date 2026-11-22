# ping_throttle

A Rust library crate: collects events and notifies subscribers at a cadence.
`readme.md` describes the cadence and the API.

- The crate reads no clock and starts no timer. The caller passes the time of
  each call. Keep it that way, so that each test sets its own times.
- The crate has no dependencies. A dependency on a runtime, a clock or a timer
  does not fit the rule above.
- Checks before each commit, the same that CI runs:
  - `cargo fmt --all -- --check`
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo build --locked --all-targets --all-features`
  - `cargo test --locked --all-features`
  - `cog check`
- Tests: one file for each subject in `tests/`, names that start with
  `should_`, no comments.
