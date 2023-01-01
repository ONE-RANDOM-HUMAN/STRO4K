# STRO4K
A chess engine designed to fit into 4096 bytes. A successor to we4k.

## Current Plan
STRO4K targets 2700 elo (CCRL) on a single thread, while actually scaling with multiple threads (unlike we4k). This target is very unlikely to be met. If the playing strength is too low, then the we4k name will be used instead.

The plan is to first develop STRO, a non-4k version of STRO4K with more interface features. STRO4K will be created as a port of STRO. It is not decided whether STRO4K will be written in source or binary form, so STRO will be written as if it will be ported to binary.

A binary can be about 5500-6000 bytes and compress to. Planned byte allocations are
* Uci and board representation - 2000 bytes
* Search - 2000 bytes
* Eval - 1000 bytes
* Something else - 500 bytes

## Features
Maybe there will be some once development has actually started. An NNUE may be included.

## Questions
### Why is it called STRO4K?
It might be stronger than we4k after a while.
