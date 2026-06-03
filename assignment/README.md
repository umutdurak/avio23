# Avio23 Assignment — Fuel Management System

Everything needed for the **Aeronautical Informatics** 2-hour hands-on session lives in this directory.

## Contents

| File / dir | Audience | Purpose |
|------------|----------|---------|
| [`student-manual.md`](student-manual.md) | Students | Reference manual: what Avio23 is, how the starter kit works, how to build & run, how grading works. Print or PDF for handouts. |
| [`assignment.md`](assignment.md) | Students | The actual assignment task — spec, deliverables, rubric, bonuses. |
| [`tutor-presentation/`](tutor-presentation/) | Tutor | 20-minute Avio23 intro deck (`.pptx`) plus a Markdown outline that mirrors it slide-for-slide. |
| [`starter/`](starter/) | Students | Self-contained Rust crate they edit. Pure Rust, no Docker. Two files to touch: `cpm_f.yaml` and `src/controller.rs`. |

## The 2-hour session at a glance

| Time | Activity | Material |
|------|----------|----------|
| 0:00 – 0:20 | Tutor introduces Avio23 | `tutor-presentation/avio23-intro.pptx` |
| 0:20 – 0:30 | Tutor introduces the assignment | `assignment.md` (project on screen) |
| 0:30 – 0:45 | Students: Part 1 — configuration | `starter/cpm_f.yaml` |
| 0:45 – 1:25 | Students: Part 2 — controller logic | `starter/src/controller.rs` |
| 1:25 – 1:50 | Grading + live demo on the full Docker stack | `cargo test --release` per student |
| 1:50 – 2:00 | Wrap-up, grades, leave | — |

## Reference solution for the tutor

A passing reference implementation of `select_source_tank` is intentionally **not** committed here, so the starter folder can be handed out as-is. The tutor's private reference lives outside this directory (or on a separate branch).

The full Avio23 docker stack at the repo root (`docker compose up`) is the live demo target. Drop a student's `controller.rs` into a pre-prepared `implementation/cpm_f/src/bin/fuel_controller.rs` wrapper (provided separately) to demo their code on real partitions.
