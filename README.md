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
    * Null Move Pruning
    * Futility Pruning
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
