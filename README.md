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
3747 bytes
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
Score of STRO-005ec0b21fbee26675b3efb3696e3de5e8427504 vs we4k-tcec: 223 - 169 - 108  [0.554] 500
...      STRO-005ec0b21fbee26675b3efb3696e3de5e8427504 playing White: 116 - 88 - 46  [0.556] 250
...      STRO-005ec0b21fbee26675b3efb3696e3de5e8427504 playing Black: 107 - 81 - 62  [0.552] 250
...      White vs Black: 197 - 195 - 108  [0.502] 500
Elo difference: 37.7 +/- 27.1, LOS: 99.7 %, DrawRatio: 21.6 %
Finished match
```

### How do you plan on fitting a neural network in 4096 bytes?
Hope.
