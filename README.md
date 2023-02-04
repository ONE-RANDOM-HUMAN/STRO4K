# STRO4K
A chess engine designed to fit into 4096 bytes. A successor to we4k.

## Current Plan
STRO4K targets 2700 elo (CCRL) on a single thread, while actually scaling with multiple threads (unlike we4k). This target is very unlikely to be met.

The plan is to first develop STRO, a non-4k version of STRO4K with more interface features. STRO4K will be created as a port of STRO. It has not been decided whether STRO4K will be created as compressed source or binary (asm). STRO will be written as if it will be ported to asm, as porting to asm is much harder.

### Plan for Binary
A binary can be about 5500-6000 bytes and compress to 4k. Planned byte allocations are
* Uci and board representation - 2000 bytes
* Search - 2000 bytes
* Eval - 1000 bytes
* Something else - 500 bytes

## Features
* Search
    * Principal Variation Search
    * Transposition Table
    * MVV-LVA Move ordering
    * Killer Heuristic
    * History Heuristic
    * Late Move Reductions
* Eval
    * Material
    * Mobility
    * Bishop Pair
    * Doubled Pawns
    * Passed Pawns

A neural network is planned.

## Questions
### Why is it called STRO4K?
It might be stronger than we4k after a while.
