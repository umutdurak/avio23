# Avio23 Intro — Tutor Presentation Outline

Target: ~20 minutes, ~15 slides. Slides 1–11 set up the platform; slides 12–15 hand off to the assignment.

The companion `.pptx` mirrors this file slide-for-slide. Edit either and re-export to keep them in sync.

---

## Slide 1 — Title

**Aeronautical Informatics**
**Avio23 — An IMA Teaching Platform**

apl. Prof. Dr.-Ing. Umut Durak · DLR Institute of Flight Systems · TU Clausthal

*Speaker notes:* Welcome. Today you write a small piece of an avionics system on the same architectural pattern that runs every modern airliner cockpit.

---

## Slide 2 — Where avionics software runs

A modern cockpit (A350, B787, Embraer E2) is **not** dozens of separate boxes anymore. It's a handful of shared computers running **partitioned applications**. This is called **Integrated Modular Avionics (IMA)**.

*Speaker notes:* Pre-IMA: one LRU per function. Post-IMA: one CPM hosting many partitions. Saves weight, power, certification. Standardized by ARINC 653 and DO-297.

---

## Slide 3 — What IMA gives you

Three properties:

1. **Space partitioning** — one partition cannot corrupt another's memory.
2. **Time partitioning** — one partition cannot steal another's CPU time.
3. **Static schedule** — every partition runs at deterministic times, every frame.

*Speaker notes:* The first two are why certification at different DALs is even possible. The third is why timing is predictable enough for hard-real-time control.

---

## Slide 4 — ARINC 653 in one slide

| Concept | One-line definition |
|---------|---------------------|
| Partition | An isolated application with its own memory & scheduled time slot. |
| Major frame | The repeating period that contains one slot for every partition. |
| Time window | A partition's `(offset, duration)` inside the major frame. |
| Sampling port | One-way mailbox; latest-value semantics; no queue. |
| Queueing port | One-way FIFO; messages preserved in order. |
| Health monitor | Watchdog that escalates partition faults. |

*Speaker notes:* Sampling ports are what we use today. Reader gets whatever was last written. No backpressure, no queue overflow. Perfect for periodic sensor data.

---

## Slide 5 — Avio23 architecture

5 physical nodes (Docker containers) — sim_gateway + 4 domain CPMs. 14 partitions total. Inter-CPM communication over a virtual AFDX network (UDP on a Docker bridge).

```
       sim_gateway (IOM)
              |
   +----+-----+-----+----+
   |    |     |     |    |
 CPM-L CPM-F CPM-A CPM-E
```

*Speaker notes:* Each box is a separate Linux container running its own a653rs-linux hypervisor. AFDX bridging is faked with UDP, but the application code sees the same sampling-port API it would on real hardware.

---

## Slide 6 — Inside one CPM

Each CPM runs a local hypervisor (`a653rs-linux`, from DLR-FT) that schedules its partitions in a fixed major frame.

```
Major frame = 40 ms
| gateway 3ms | app_1 2ms | app_2 2ms | idle 33ms |
```

*Speaker notes:* The 33 ms of idle margin is intentional. Real systems get budget conflicts late in the program; you want headroom. Avio23 leaves plenty so students have room to add their own partition.

---

## Slide 7 — How partitions talk

Sampling ports. Inside one CPM, ports are shared memory. Between CPMs, the local gateway partition serializes the port content to UDP and a peer gateway deserializes it.

```
[ sensor_partition ] --port--> [ gateway ] --UDP--> [ peer_gateway ] --port--> [ consumer ]
```

*Speaker notes:* This is one place Avio23 differs from real ARINC 664 hardware — but the application code is unchanged. The gateway abstracts the network.

---

## Slide 8 — The domains

| CPM | Domain | DAL | What it does |
|-----|--------|-----|--------------|
| CPM-L | Landing Gear | B | Gear lever, braking, steering |
| **CPM-F** | **Fuel** | **C** | **Fuel quantity, transfer pumps, balance** |
| CPM-A | ECS / Air Conditioning | D | Bleed air, cabin temperature |
| CPM-E | Electrical / Energy | B | Generators, load shedding |

*Speaker notes:* DAL (Design Assurance Level) is from DO-178C: A is highest, E lowest. Fuel sits at C in this teaching model — failure leads to engine starvation but not directly to loss of aircraft.

---

## Slide 9 — Zooming in: CPM-F

Today everyone works in here.

```
              [ cpm_f_gateway ]  (talks to AFDX)
                  |
                  | iom_to_fuel
                  v
   +-------------+--+----------------+----------------+
   | fuel_quantity | transfer_pump   | fuel_controller |   <-- you add this one
   +---------------+-----------------+-----------------+
                  |
                  | controller_to_iom
                  v
              [ cpm_f_gateway ] -> AFDX -> sim_gateway -> sim
```

*Speaker notes:* The first three partitions are pre-built reference code. You only touch `fuel_controller`. You will both configure it (schedule slot + ports) and write its logic.

---

## Slide 10 — Live demo

`docker compose up` →
`docker compose logs -f sim_gateway`

Show the simulated flight running. Point out the fuel telemetry stream and the current (reference) transfer-pump activity.

*Speaker notes:* Keep this short. The goal is "this is what a passing solution looks like at the end of the session." Don't over-explain the logs.

---

## Slide 11 — Why bother with IMA in a course?

- Real cockpit software is structured exactly this way.
- The integration skill (filling in a schedule, allocating ports) is what distinguishes an avionics engineer from a generic embedded engineer.
- You learn the **vocabulary** that every avionics standards document (DO-297, ARP4754A, ARINC 653) is written in.

*Speaker notes:* Software engineering courses teach you to write functions. This course teaches you to write functions that meet a schedule, share a bus, and stay inside a partition. The hard part isn't the algorithm — it's the constraints around it.

---

## Slide 12 — Today's assignment

**Fuel Management System** — write the controller that decides which tank the engine draws from, keeping the two wing tanks balanced.

Two deliverables:

1. **Config** — `cpm_f.yaml`: add your partition window + sampling ports
2. **Code** — `controller.rs`: implement `select_source_tank()`

*Speaker notes:* This is the moment you switch from "Avio23 intro" to "what you do in the next 75 minutes." Hand them the printed assignment.

---

## Slide 13 — The constraints that make this interesting

- `|left − right| ≤ 10 L` at all times (aircraft stability)
- Pump valve must not switch more than once every 0.8 s (physical wear)
- Controller is invoked every 40 ms (one major frame)
- The fuel keeps burning the whole time

*Speaker notes:* The interesting part: the cooldown and balance requirements are **coupled**. A naive "switch whenever imbalanced" controller violates the cooldown. You have to think about state.

---

## Slide 14 — Your toolchain today

- Rust + `cargo` (no Docker, no Linux VM needed on your laptop)
- Test bench: `cargo run --bin test_bench`
- Config validator: `cargo run --bin test_bench -- --validate-config`
- Grader (same one I use): `cargo test --release -- --nocapture`

Submit `cpm_f.yaml`, `controller.rs`, and one paragraph of reflection. Grading is instant.

*Speaker notes:* Everyone has Rust on their laptop already from earlier sessions. If not, `rustup install stable` takes 90 seconds.

---

## Slide 15 — The plan for this 2-hour block

| Time | What you're doing |
|------|------------------|
| 0:00 – 0:20 | This intro |
| 0:20 – 0:30 | Assignment briefing (next presentation) |
| 0:30 – 0:45 | Part 1 — Configuration |
| 0:45 – 1:25 | Part 2 — Controller logic |
| 1:25 – 1:50 | Grading + live demo on the real stack |
| 1:50 – 2:00 | Wrap-up, grades, leave |

Questions? Then let's go.

*Speaker notes:* Keep the questions tight. Anything not answerable in 30 seconds, defer to during the work portion — you'll be circulating.
