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

## Benchmark
Inside a VM with 3 threads and 4 GB of memory on my laptop, I get the following results on simple example data.

First, to generate example data I ran the following node script:

```js
let tx = 0;
console.log("type,client,tx,amount")
for (let i = 0; i < 5_000; i++) {
    for (let client = 0; client < 50; client++) {
        console.log(`deposit,${client},${tx++},${client}`)
    }
}
```

Then we can use Hyperfine to generate some benchmarks.

```sh
hyperfine --parameter-list threads 1,2,3,4,5,6 --parameter-list depth 1,10,100,1000,10000 --export-markdown bench.md -w 1 './target/release/payment_engine ./test.csv -d {depth} -w {threads}'
```

Running that got me the following summary:

```log
Summary
  './target/release/payment_engine ./test.csv -d 100 -w 1' ran
    1.01 ± 0.22 times faster than './target/release/payment_engine ./test.csv -d 10 -w 1'
    1.03 ± 0.34 times faster than './target/release/payment_engine ./test.csv -d 1000 -w 1'
    1.04 ± 0.29 times faster than './target/release/payment_engine ./test.csv -d 10000 -w 1'
    1.30 ± 0.28 times faster than './target/release/payment_engine ./test.csv -d 10000 -w 2'
    1.33 ± 0.30 times faster than './target/release/payment_engine ./test.csv -d 1000 -w 2'
    1.54 ± 0.49 times faster than './target/release/payment_engine ./test.csv -d 100 -w 2'
    1.67 ± 0.35 times faster than './target/release/payment_engine ./test.csv -d 1000 -w 3'
    1.68 ± 0.46 times faster than './target/release/payment_engine ./test.csv -d 10 -w 2'
    1.73 ± 0.45 times faster than './target/release/payment_engine ./test.csv -d 10000 -w 3'
    1.83 ± 0.41 times faster than './target/release/payment_engine ./test.csv -d 1000 -w 4'
    1.84 ± 0.41 times faster than './target/release/payment_engine ./test.csv -d 100 -w 3'
    1.89 ± 0.43 times faster than './target/release/payment_engine ./test.csv -d 10000 -w 4'
    1.94 ± 0.40 times faster than './target/release/payment_engine ./test.csv -d 1000 -w 5'
    1.96 ± 0.40 times faster than './target/release/payment_engine ./test.csv -d 10000 -w 6'
    1.98 ± 0.41 times faster than './target/release/payment_engine ./test.csv -d 1000 -w 6'
    1.98 ± 0.42 times faster than './target/release/payment_engine ./test.csv -d 10000 -w 5'
    2.00 ± 0.42 times faster than './target/release/payment_engine ./test.csv -d 100 -w 4'
    2.01 ± 0.43 times faster than './target/release/payment_engine ./test.csv -d 100 -w 5'
    2.03 ± 0.44 times faster than './target/release/payment_engine ./test.csv -d 1 -w 3'
    2.08 ± 0.45 times faster than './target/release/payment_engine ./test.csv -d 10 -w 5'
    2.08 ± 0.43 times faster than './target/release/payment_engine ./test.csv -d 100 -w 6'
    2.10 ± 0.50 times faster than './target/release/payment_engine ./test.csv -d 1 -w 2'
    2.15 ± 0.49 times faster than './target/release/payment_engine ./test.csv -d 1 -w 6'
    2.15 ± 0.43 times faster than './target/release/payment_engine ./test.csv -d 1 -w 4'
    2.19 ± 0.48 times faster than './target/release/payment_engine ./test.csv -d 1 -w 5'
    2.22 ± 0.50 times faster than './target/release/payment_engine ./test.csv -d 10 -w 6'
    2.48 ± 0.56 times faster than './target/release/payment_engine ./test.csv -d 1 -w 1'
    2.55 ± 0.65 times faster than './target/release/payment_engine ./test.csv -d 10 -w 3'
    2.75 ± 0.91 times faster than './target/release/payment_engine ./test.csv -d 10 -w 4'
```

Also presented here as a markdown table:

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/payment_engine ./test.csv -d 1 -w 1` | 837.2 ± 94.2 | 680.1 | 997.2 | 2.48 ± 0.56 |
| `./target/release/payment_engine ./test.csv -d 1 -w 2` | 706.8 ± 97.9 | 568.5 | 911.2 | 2.10 ± 0.50 |
| `./target/release/payment_engine ./test.csv -d 1 -w 3` | 684.9 ± 69.5 | 622.9 | 817.6 | 2.03 ± 0.44 |
| `./target/release/payment_engine ./test.csv -d 1 -w 4` | 725.8 ± 41.7 | 647.8 | 769.8 | 2.15 ± 0.43 |
| `./target/release/payment_engine ./test.csv -d 1 -w 5` | 739.7 ± 72.9 | 639.9 | 867.9 | 2.19 ± 0.48 |
| `./target/release/payment_engine ./test.csv -d 1 -w 6` | 724.3 ± 90.6 | 643.9 | 955.6 | 2.15 ± 0.49 |
| `./target/release/payment_engine ./test.csv -d 10 -w 1` | 339.8 ± 37.2 | 299.5 | 434.9 | 1.01 ± 0.22 |
| `./target/release/payment_engine ./test.csv -d 10 -w 2` | 564.8 ± 112.3 | 422.4 | 748.5 | 1.68 ± 0.46 |
| `./target/release/payment_engine ./test.csv -d 10 -w 3` | 858.6 ± 143.2 | 674.3 | 1145.9 | 2.55 ± 0.65 |
| `./target/release/payment_engine ./test.csv -d 10 -w 4` | 927.5 ± 249.7 | 670.5 | 1334.3 | 2.75 ± 0.91 |
| `./target/release/payment_engine ./test.csv -d 10 -w 5` | 700.1 ± 66.6 | 608.8 | 825.0 | 2.08 ± 0.45 |
| `./target/release/payment_engine ./test.csv -d 10 -w 6` | 746.9 ± 87.0 | 631.1 | 877.4 | 2.22 ± 0.50 |
| `./target/release/payment_engine ./test.csv -d 100 -w 1` | 337.1 ± 65.2 | 289.6 | 517.2 | 1.00 |
| `./target/release/payment_engine ./test.csv -d 100 -w 2` | 520.5 ± 132.3 | 391.9 | 749.8 | 1.54 ± 0.49 |
| `./target/release/payment_engine ./test.csv -d 100 -w 3` | 621.7 ± 68.9 | 530.7 | 721.2 | 1.84 ± 0.41 |
| `./target/release/payment_engine ./test.csv -d 100 -w 4` | 674.9 ± 59.2 | 595.8 | 795.1 | 2.00 ± 0.42 |
| `./target/release/payment_engine ./test.csv -d 100 -w 5` | 676.7 ± 63.1 | 610.7 | 790.3 | 2.01 ± 0.43 |
| `./target/release/payment_engine ./test.csv -d 100 -w 6` | 700.1 ± 52.7 | 645.9 | 824.1 | 2.08 ± 0.43 |
| `./target/release/payment_engine ./test.csv -d 1000 -w 1` | 346.7 ± 92.8 | 289.4 | 606.4 | 1.03 ± 0.34 |
| `./target/release/payment_engine ./test.csv -d 1000 -w 2` | 448.1 ± 50.2 | 381.5 | 535.1 | 1.33 ± 0.30 |
| `./target/release/payment_engine ./test.csv -d 1000 -w 3` | 564.2 ± 45.9 | 508.9 | 660.4 | 1.67 ± 0.35 |
| `./target/release/payment_engine ./test.csv -d 1000 -w 4` | 616.2 ± 68.6 | 551.5 | 794.2 | 1.83 ± 0.41 |
| `./target/release/payment_engine ./test.csv -d 1000 -w 5` | 653.8 ± 47.9 | 591.2 | 758.1 | 1.94 ± 0.40 |
| `./target/release/payment_engine ./test.csv -d 1000 -w 6` | 666.7 ± 52.2 | 599.4 | 788.3 | 1.98 ± 0.41 |
| `./target/release/payment_engine ./test.csv -d 10000 -w 1` | 351.9 ± 68.3 | 264.6 | 513.6 | 1.04 ± 0.29 |
| `./target/release/payment_engine ./test.csv -d 10000 -w 2` | 437.9 ± 45.4 | 384.3 | 538.5 | 1.30 ± 0.28 |
| `./target/release/payment_engine ./test.csv -d 10000 -w 3` | 584.8 ± 101.9 | 508.1 | 847.8 | 1.73 ± 0.45 |
| `./target/release/payment_engine ./test.csv -d 10000 -w 4` | 637.3 ± 75.7 | 547.4 | 750.0 | 1.89 ± 0.43 |
| `./target/release/payment_engine ./test.csv -d 10000 -w 5` | 668.5 ± 55.0 | 606.9 | 752.6 | 1.98 ± 0.42 |
| `./target/release/payment_engine ./test.csv -d 10000 -w 6` | 659.9 ± 42.9 | 590.8 | 712.3 | 1.96 ± 0.40 |

### Analysis
There are a few things going on here, but I think there's one particularly glaring one: [as predicted](https://github.com/mkeedlinger/toy-transaction-engine/blob/29f549eead3c8b0d88b23931cfeb9718f299db09/src/main.rs#L72-L78), async was likely unnecessary for this task, and likely even introduces overhead. I think this is also true of the threading impl, since the applications overall logic is relatively simple.

I think it's possible that these fancy implementation details could become more useful if (a) there were more complicated logic that required network calls to other services and/or (b) this were actually presented as a network service instead of a CLI tool.
