# Toy Transaction Engine

A simple toy transaction engine that takes a series of transactions as input, and outputs a csv of client summaries.

## Requirements
Currently, this only builds using Rust nightly, and has been tested with `rustc 1.64.0-nightly (6dba4ed21 2022-07-09)`.

### Why nightly?
Because the `Read` trait was only implemented for `VeqDeque` [on 9 June 2022](https://github.com/rust-lang/rust/pull/95632) and the latest Rust release (Rust 1.62.0 as of writing) does [not have this](https://doc.rust-lang.org/1.62.0/std/collections/struct.VecDeque.html#trait-implementations).

## How to use
Considering how unlikely it is that you vend/install this, the easiest way to use this is directly running from Cargo:

```
cargo run --release -- input.csv
```

You can also output to a another CSV file instead of stdout:

```
cargo run --release -- input.csv output.csv
```

In the ~~unlikely~~ event other features were ever added, you would be able to see them with using the help flag:

```
cargo run --release -- --help
```
