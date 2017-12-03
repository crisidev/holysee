* Issues are welcome
* PRs are even more welcome
* **Please run `cargo clippy` and do not care about these 2 warnings**
```
warning: variable does not need to be mutable
   --> src/main.rs:105:9
    |
105 | /         chan_select! {
106 | |             from_irc.recv() -> irc_answer => {
107 | |                 match irc_answer {
108 | |                     Some(msg) => {
...   |
129 | |             }
130 | |         }
    | |_________^ help: remove this `mut`
    |
    = note: #[warn(unused_mut)] on by default
    = note: this error originates in a macro outside of the current crate

warning: use of `or_insert` followed by a function call
  --> src/commands/last_seen.rs:76:10
   |
76 |           *(self.last_seen.entry(who.to_owned()).or_insert(
   |  __________^
77 | |             Local::now().timestamp(),
78 | |         )) = Local::now().timestamp();
   | |__________^ help: try this: `self.last_seen.entry(who.to_owned()).or_insert_with(|| Local::now().timestamp())`
   |
   = note: #[warn(or_fun_call)] on by default
   = help: for further information visit https://rust-lang-nursery.github.io/rust-clippy/v0.0.171/index.html#or_fun_call
```
* **Please run `cargo fmt` before sending a PR**
