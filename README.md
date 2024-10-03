# project


## Aiming
- [ ] Version 1
  - [x] Scan disk and calculate folder size, reusing existing fsnodes.
  - [x] Serialize and Deserialize
  - [ ] Console interaction
- [ ] Version 2
  - [ ] Fuzz search, prefix and suffix trie
  - [ ] Cross platform
  - [ ] Python interface
- [ ] Version 3
  - [ ] synchronize with remote server

## Difficulties
- [ ] How to gracefully perform console interaction?
  - Answer: `crossterm`, a cross-platform library for terminal control.
- [ ] How to serialize and deserialize?
  - Answer: `serde`, deprecate JSON, use CSV instead.

## Debug
```sh
cargo run --release --bin project -- --debug
cargo test -- --test-threads=1
```
