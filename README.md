# STRO4K
![logo](logo.png)
A chess engine designed to fit into 4096 bytes. A successor to we4k.

## Current Plan
STRO4K currently targets 2900 elo (CCRL) on a single thread. This target is very unlikely to be met.

STRO is a non-4k version of STRO4K with more interface features, which is much easier to develop.

## Version 2.1
STRO4K 2.1 will output the evaluation at the end of each search. This evaluation is provided in octal to minimised the number of bytes required for this feature. The depth is always output as 1.

STRO4K 2.1 does not reset the state of the engine after receiving `ucinewgame`. The engine should be restarted instead.

## Building
STRO4K has only been tested to build on Linux systems. Building STRO4K requires `nasm`, `xz` and [`sstrip`](https://github.com/aunali1/super-strip). A script is provided which attempts to download `sstrip` and build STRO4K.

```
./build4k <file_name> <thread_count> <hash_size_mb>
```

For a default build with 4 threads, 16MB hash, and output file `STRO4K`:
```
./build4kdefault
```

STRO can be built using a Rust nightly compiler. By default, this includes an `asm` option that will allow STRO4K search and eval code to be used. This will only work on Linux systems
```
cargo build --release
```

STRO can also be built without the `asm` feature. This should also work on Windows.
```
cargo build --release --no-default-features
```

## Current size
```
4091 bytes
```
## Features
* PV output in STRO
* Search
    * Principal Variation Search
    * Transposition Table
    * Aspiration Windows
    * MVV-LVA Move ordering
    * Killer Heuristic
    * History Heuristic
    * Late Move Reductions
    * History Leaf Pruning
    * Null Move Pruning
    * Static Null Move Pruning
    * Futility Pruning
    * Lazy SMP
    * Internal Iterative Reductions
* Evaluation
    * Material
    * Mobility
    * Bishop Pair
    * Doubled Pawns
    * Passed Pawns
        * Blocked Passed Pawns
    * Isolated Pawns
    * Open Files
    * Rank and File Piece Tables
    * Pawn Shield
    * Tempo
    * Pieces attacked and defended by pawns

A neural network is planned.

## Questions
### Why is it called STRO4K?
```
Score of STRO4K-1.0 vs we4k-tcec: 146 - 28 - 26  [0.795] 200
...      STRO4K-1.0 playing White: 78 - 13 - 9  [0.825] 100
...      STRO4K-1.0 playing Black: 68 - 15 - 17  [0.765] 100
...      White vs Black: 93 - 81 - 26  [0.530] 200
Elo difference: 235.4 +/- 54.2, LOS: 100.0 %, DrawRatio: 13.0 %
Finished match
```

### How do you plan on fitting a neural network in 4096 bytes?
Hope.
