# STRO4K
A chess engine designed to fit into 4096 bytes. A successor to we4k.

## Current Plan
STRO4K targets 2700 elo (CCRL) on a single thread, while actually scaling with multiple threads (unlike we4k). This target is very unlikely to be met.

STRO is a non-4k version of STRO4K with more interface features, which is much easier to develop.

## Building
Building STRO4K requires `nasm`, `xz` and [`sstrip`](https://github.com/aunali1/super-strip). A script is provided which attempts to download `sstrip` and build STRO3K.

```
./build4k
```

STRO can be build using a Rust nightly compiler.
```
cargo build --release
```

## Current size
```
4043 bytes
```
## Features
* Search
    * Principal Variation Search
    * Transposition Table
    * MVV-LVA Move ordering
    * Killer Heuristic
    * History Heuristic
    * Late Move Reductions
    * Null Move Pruning
    * Futility Pruning
    * Lazy SMP
* Evaluation
    * Material
    * Mobility
    * Bishop Pair
    * Doubled Pawns
    * Passed Pawns
    * Open Files

A neural network is planned.

## Questions
### Why is it called STRO4K?
```
Score of STRO4K-f261aa061f4626d48f923c19961ef040f343980d vs we4k-tcec: 120 - 46 - 34  [0.685] 200
...      STRO4K-f261aa061f4626d48f923c19961ef040f343980d playing White: 64 - 20 - 16  [0.720] 100
...      STRO4K-f261aa061f4626d48f923c19961ef040f343980d playing Black: 56 - 26 - 18  [0.650] 100
...      White vs Black: 90 - 76 - 34  [0.535] 200
Elo difference: 135.0 +/- 46.8, LOS: 100.0 %, DrawRatio: 17.0 %
Finished match
```

### How do you plan on fitting a neural network in 4096 bytes?
Hope.
