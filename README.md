# STRO4K
A chess engine designed to fit into 4096 bytes. A successor to we4k.

## Current Plan
STRO4K targets 2700 elo (CCRL) on a single thread, while actually scaling with multiple threads (unlike we4k). This target is very unlikely to be met.

STRO is a non-4k version of STRO4K with more interface features, which is much easier to develop. STRO4K is a work in progress assembly port of STRO designed to fit into 4096 bytes. Assembly was chosen due to unpromising attempts at minification.


### Plan for STRO4K
The current goal is to create an initial version of STRO4K as strong as we4k that will fit in 4096 bytes uncompressed. This goal is only slightly more likely than the 2700 elo goal. The approximate byte allocations for this are:
* Movegen - ~850 bytes
* Board Representation - ~650 bytes
* Uci - ~500 bytes
* Search - ~1400 bytes
* Evaluation - ~600 bytes

The ~1500 bytes of additional space given by compression will be used to increase the strength of STRO4K. Planned final byte allocations are approximately:
* Uci and board representation - ~2000 bytes
* Search - ~2000 bytes
* Eval - ~1000 bytes
* Something else - ~500 bytes

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
