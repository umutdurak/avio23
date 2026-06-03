# Fuel Controller Starter Kit

Your working directory for the Avio23 Fuel Management Assignment. Pure Rust — no Docker, no Linux VM, no special setup.

## Quick start

```bash
# 1. Make sure you have Rust 1.80+
rustc --version

# 2. (optional) build once to pull deps
cargo build --release

# 3. Validate your cpm_f.yaml after editing Part 1
cargo run --bin test_bench -- --validate-config

# 4. Run the default scenario against your controller
cargo run --bin test_bench

# 5. Run every scenario the grader uses
cargo run --bin test_bench -- --all

# 6. Run the full grader (same code the tutor uses)
cargo test --release -- --nocapture
```

## Files

```
starter/
├── Cargo.toml              # crate metadata (don't edit)
├── cpm_f.yaml              # << Part 1: you edit this
├── src/
│   ├── lib.rs              # types: Tank, FuelSensors, FuelActuators, ...
│   ├── controller.rs       # << Part 2: you edit this (select_source_tank)
│   ├── config.rs           # YAML validator (don't edit)
│   ├── scenario.rs         # scenario runner (don't edit)
│   └── bin/
│       └── test_bench.rs   # CLI entry point (don't edit)
└── tests/
    └── grader.rs           # graded integration test (don't edit)
```

## Outputs

- `out/trace.csv` — per-frame trace from the last test-bench run. Plot it with any spreadsheet or `gnuplot`:

  ```bash
  gnuplot -p -e "set datafile separator ','; \
                 plot 'out/trace.csv' using 1:2 with lines title 'left', \
                      '' using 1:3 with lines title 'right'"
  ```

## Submission

Leave these files in this folder at the end of the session:

1. `cpm_f.yaml`
2. `src/controller.rs`
3. `REFLECTION.md` (1 short paragraph — see `assignment.md`)

The tutor will run `cargo test --release` against this directory. That run produces your grade.
