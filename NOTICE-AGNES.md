# NOTICE: Agnes build of IronCalc

This package is built from <https://github.com/keboola/IronCalc>, a fork of
IronCalc (<https://github.com/ironcalc/IronCalc>) maintained by Keboola for
use in Agnes. It is not an official IronCalc release.

IronCalc is dual-licensed under the MIT License or the Apache License,
Version 2.0, at your option (`LICENSE-MIT`, `LICENSE-Apache-2.0`). The
changes in this fork are offered under the same terms.

## Base

IronCalc 0.9.1 (tag `v0.9.1`, commit `eb9ca75bd99160c17902359d2d87225c4571c468`).

## Changes

One patch, based on the one posted in
<https://github.com/ironcalc/IronCalc/issues/1414>:

- Whole-column and whole-row ranges (`A:A`, `1:1`) are bounded to the
  stored data of the sheet in `MATCH`, `LOOKUP`, `VLOOKUP` and `HLOOKUP`,
  instead of walking all 1,048,576 rows or 16,384 columns. `XLOOKUP` and
  `XMATCH` are unchanged.
- The same bound applies to aggregates for which empty cells are neutral:
  `MIN`, `MAX`, `COUNT`, `COUNTA`, `AVERAGE` and the functions that share
  their value walk (such as `GEOMEAN`, `MEDIAN`, `HARMEAN`, `SKEW`, `KURT`,
  `LARGE`, `SMALL`), `MINA`, `MAXA`, `AVERAGEA`, `AND`, `OR` and `XOR`.
- `Worksheet::dimension` is computed in time proportional to the number of
  rows instead of the number of cells.
- Tests: `base/src/test/test_full_range_bounds.rs`.

Package versions: Python `ironcalc` `0.9.1+agnes.1`, npm `@ironcalc/wasm`
`0.9.1-agnes.1`.
