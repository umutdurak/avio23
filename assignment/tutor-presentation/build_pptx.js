// Build script for the Avio23 tutor intro deck.
// Run with: node build_pptx.js

const pptxgen = require("pptxgenjs");
const path = require("path");

// ============================================================
// Palette — "Cockpit Instrumentation"
// ============================================================
const C = {
  ink:    "0F1B2C",  // dominant dark text / navy backgrounds
  navy:   "1A2F4A",
  deep:   "065A82",  // primary blue
  teal:   "1C7293",  // secondary
  sky:    "9AD1F4",  // light highlight
  amber:  "F2A65A",  // single sharp accent (warning lights, attention)
  paper:  "F4F7FA",  // light slide background
  muted:  "64748B",
  white:  "FFFFFF",
  rule:   "D5DDE8",
};

const F = { head: "Calibri", body: "Calibri" };

const pres = new pptxgen();
pres.layout = "LAYOUT_WIDE";   // 13.33 x 7.5
pres.author = "apl. Prof. Dr.-Ing. Umut Durak";
pres.title = "Avio23 — An IMA Teaching Platform";

const W = 13.33;
const H = 7.5;

// ============================================================
// Reusable helpers
// ============================================================

function darkChrome(slide) {
  slide.background = { color: C.ink };
}
function lightChrome(slide) {
  slide.background = { color: C.paper };
}

// Page header — colored chip + section label + page footer line.
function lightHeader(slide, kicker, title) {
  slide.addShape(pres.shapes.RECTANGLE, {
    x: 0.6, y: 0.6, w: 0.18, h: 0.32, fill: { color: C.deep }, line: { color: C.deep },
  });
  slide.addText(kicker.toUpperCase(), {
    x: 0.9, y: 0.55, w: 8, h: 0.4,
    fontFace: F.body, fontSize: 11, color: C.deep, bold: true, charSpacing: 4,
    margin: 0,
  });
  slide.addText(title, {
    x: 0.6, y: 1.0, w: W - 1.2, h: 0.9,
    fontFace: F.head, fontSize: 36, color: C.ink, bold: true,
    margin: 0,
  });
}

function footer(slide, page) {
  slide.addShape(pres.shapes.RECTANGLE, {
    x: 0.6, y: H - 0.55, w: W - 1.2, h: 0.01,
    fill: { color: C.rule }, line: { color: C.rule },
  });
  slide.addText("Avio23  ·  Aeronautical Informatics  ·  DLR / TU Clausthal", {
    x: 0.6, y: H - 0.45, w: 10, h: 0.3,
    fontFace: F.body, fontSize: 9, color: C.muted,
  });
  slide.addText(`${page}`, {
    x: W - 1.6, y: H - 0.45, w: 1.0, h: 0.3,
    fontFace: F.body, fontSize: 9, color: C.muted, align: "right",
  });
}

// ============================================================
// Slide 1 — Title
// ============================================================
{
  const s = pres.addSlide();
  darkChrome(s);

  // Left rail accent bar
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0, y: 0, w: 0.5, h: H, fill: { color: C.deep }, line: { color: C.deep },
  });
  // Inner deep blue band
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.5, y: 0, w: 0.05, h: H, fill: { color: C.teal }, line: { color: C.teal },
  });

  s.addText("AERONAUTICAL INFORMATICS", {
    x: 1.0, y: 1.2, w: 11, h: 0.5,
    fontFace: F.body, fontSize: 13, color: C.sky, charSpacing: 8, bold: true, margin: 0,
  });

  s.addText("Avio23", {
    x: 1.0, y: 1.8, w: 11, h: 1.6,
    fontFace: F.head, fontSize: 84, color: C.white, bold: true, margin: 0,
  });

  s.addText("An IMA Teaching Platform", {
    x: 1.0, y: 3.3, w: 11, h: 0.8,
    fontFace: F.head, fontSize: 30, color: C.sky, margin: 0,
  });

  s.addText("Where every partition has its turn.", {
    x: 1.0, y: 4.2, w: 11, h: 0.5,
    fontFace: F.body, fontSize: 16, color: C.white, italic: true, margin: 0,
  });

  // Author block
  s.addShape(pres.shapes.RECTANGLE, {
    x: 1.0, y: 5.6, w: 0.05, h: 1.2, fill: { color: C.amber }, line: { color: C.amber },
  });
  s.addText([
    { text: "apl. Prof. Dr.-Ing. Umut Durak", options: { breakLine: true, bold: true, fontSize: 14, color: C.white } },
    { text: "DLR Institute of Flight Systems · TU Clausthal", options: { fontSize: 12, color: C.sky } },
  ], { x: 1.25, y: 5.6, w: 10, h: 1.2, fontFace: F.body, margin: 0 });
}

// ============================================================
// Slide 2 — Where avionics runs (comparison)
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "01 — Why IMA", "Where avionics software runs today");

  // Two-column comparison
  const cols = [
    { x: 0.8, fill: C.rule, title: "Pre-IMA (federated)", body: "One LRU per function. Dozens of boxes. Each owns its CPU, wiring, certification.", emoji: "OLD" },
    { x: 6.9, fill: C.deep, title: "IMA (today)",         body: "A handful of shared CPMs hosting many partitions. Shared CPU, shared bus, isolated logic.", emoji: "NEW" },
  ];
  cols.forEach((c) => {
    s.addShape(pres.shapes.RECTANGLE, {
      x: c.x, y: 2.2, w: 5.6, h: 4.4,
      fill: { color: C.white }, line: { color: C.rule, width: 0.75 },
      shadow: { type: "outer", color: "000000", blur: 12, offset: 2, angle: 90, opacity: 0.08 },
    });
    s.addShape(pres.shapes.RECTANGLE, {
      x: c.x, y: 2.2, w: 0.12, h: 4.4, fill: { color: c.fill }, line: { color: c.fill },
    });
    s.addText(c.title, {
      x: c.x + 0.35, y: 2.4, w: 5.0, h: 0.6,
      fontFace: F.head, fontSize: 22, color: C.ink, bold: true, margin: 0,
    });
    s.addText(c.body, {
      x: c.x + 0.35, y: 3.05, w: 5.1, h: 2.0,
      fontFace: F.body, fontSize: 15, color: C.ink, margin: 0,
    });
  });

  s.addText("Modern cockpit software (A350, B787, E2, Global) runs the right column.", {
    x: 0.8, y: 6.8, w: 12, h: 0.4,
    fontFace: F.body, fontSize: 12, color: C.muted, italic: true,
  });

  footer(s, 2);
}

// ============================================================
// Slide 3 — What IMA gives you (3 props with icons)
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "02 — What IMA Provides", "Three properties certification depends on");

  const props = [
    { num: "01", h: "Space partitioning",  d: "One partition cannot corrupt another partition's memory. Hardware-enforced isolation.", c: C.deep },
    { num: "02", h: "Time partitioning",   d: "One partition cannot steal another partition's CPU time. Static schedule, hard guarantees.", c: C.teal },
    { num: "03", h: "Deterministic schedule", d: "Every partition runs at known times every frame. Timing analysable, hard real-time.", c: C.amber },
  ];

  props.forEach((p, i) => {
    const y = 2.3 + i * 1.55;
    // Numeric chip
    s.addShape(pres.shapes.RECTANGLE, {
      x: 0.8, y: y, w: 1.2, h: 1.2, fill: { color: p.c }, line: { color: p.c },
    });
    s.addText(p.num, {
      x: 0.8, y: y, w: 1.2, h: 1.2,
      fontFace: F.head, fontSize: 36, color: C.white, bold: true, align: "center", valign: "middle", margin: 0,
    });
    // Text
    s.addText(p.h, {
      x: 2.4, y: y + 0.08, w: 10, h: 0.5,
      fontFace: F.head, fontSize: 22, color: C.ink, bold: true, margin: 0,
    });
    s.addText(p.d, {
      x: 2.4, y: y + 0.6, w: 10, h: 0.7,
      fontFace: F.body, fontSize: 14, color: C.muted, margin: 0,
    });
  });

  footer(s, 3);
}

// ============================================================
// Slide 4 — ARINC 653 vocabulary
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "03 — Standard", "ARINC 653 in one slide");

  const rows = [
    ["Partition",      "An isolated application with its own memory and its own scheduled time slot."],
    ["Major frame",    "The repeating period (40 ms in Avio23) that contains one slot for every partition."],
    ["Time window",    "A partition's (offset, duration) inside the major frame."],
    ["Sampling port",  "One-way mailbox; latest-value semantics; no queue."],
    ["Queueing port",  "One-way FIFO; messages preserved in order."],
    ["Health monitor", "Watchdog that escalates partition faults."],
  ];

  const tableData = rows.map((r, idx) => [
    {
      text: r[0],
      options: {
        bold: true, color: C.deep, fontSize: 14, valign: "middle", margin: 6,
        fill: { color: idx % 2 ? C.paper : C.white },
      },
    },
    {
      text: r[1],
      options: {
        color: C.ink, fontSize: 13, valign: "middle", margin: 6,
        fill: { color: idx % 2 ? C.paper : C.white },
      },
    },
  ]);

  s.addTable(tableData, {
    x: 0.8, y: 2.2, w: 11.7, colW: [3.0, 8.7],
    border: { type: "solid", pt: 0.5, color: C.rule },
    fontFace: F.body,
  });

  s.addText("We use sampling ports today — perfect for periodic sensor data.", {
    x: 0.8, y: 6.7, w: 12, h: 0.4,
    fontFace: F.body, fontSize: 12, color: C.muted, italic: true,
  });

  footer(s, 4);
}

// ============================================================
// Slide 5 — Avio23 architecture diagram (block diagram)
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "04 — Architecture", "Avio23 — 5 nodes, 14 partitions");

  // sim_gateway at top
  s.addShape(pres.shapes.RECTANGLE, {
    x: 5.6, y: 2.2, w: 2.5, h: 0.9,
    fill: { color: C.amber }, line: { color: C.amber },
  });
  s.addText([
    { text: "sim_gateway", options: { bold: true, fontSize: 14, color: C.ink, breakLine: true } },
    { text: "IOM · 172.20.0.2", options: { fontSize: 10, color: C.ink } },
  ], { x: 5.6, y: 2.2, w: 2.5, h: 0.9, align: "center", valign: "middle", fontFace: F.body, margin: 0 });

  // bus line down
  s.addShape(pres.shapes.LINE, {
    x: 6.85, y: 3.1, w: 0, h: 0.7,
    line: { color: C.ink, width: 2 },
  });
  // horizontal trunk
  s.addShape(pres.shapes.LINE, {
    x: 1.7, y: 3.8, w: 10.3, h: 0,
    line: { color: C.ink, width: 2 },
  });

  // 4 CPMs
  const cpms = [
    { x: 1.2,  c: C.deep, name: "CPM-L", dom: "Landing Gear", ip: "172.20.0.3", dal: "DAL B" },
    { x: 4.2,  c: C.teal, name: "CPM-F", dom: "Fuel",         ip: "172.20.0.4", dal: "DAL C" },
    { x: 7.2,  c: C.deep, name: "CPM-A", dom: "ECS",          ip: "172.20.0.5", dal: "DAL D" },
    { x: 10.2, c: C.deep, name: "CPM-E", dom: "Electrical",   ip: "172.20.0.6", dal: "DAL B" },
  ];
  cpms.forEach((c) => {
    // drop line
    s.addShape(pres.shapes.LINE, {
      x: c.x + 1.2, y: 3.8, w: 0, h: 0.5,
      line: { color: C.ink, width: 2 },
    });
    // body
    s.addShape(pres.shapes.RECTANGLE, {
      x: c.x, y: 4.3, w: 2.4, h: 1.7, fill: { color: C.white }, line: { color: c.c, width: 2 },
    });
    s.addShape(pres.shapes.RECTANGLE, {
      x: c.x, y: 4.3, w: 2.4, h: 0.35, fill: { color: c.c }, line: { color: c.c },
    });
    s.addText(c.name, { x: c.x, y: 4.3, w: 2.4, h: 0.35, fontFace: F.head, fontSize: 14, color: C.white, bold: true, align: "center", valign: "middle", margin: 0 });
    s.addText([
      { text: c.dom, options: { fontSize: 13, color: C.ink, bold: true, breakLine: true } },
      { text: c.ip, options: { fontSize: 10, color: C.muted, breakLine: true } },
      { text: c.dal, options: { fontSize: 10, color: c.c, bold: true } },
    ], { x: c.x + 0.1, y: 4.75, w: 2.2, h: 1.25, align: "center", valign: "middle", fontFace: F.body, margin: 0 });
  });

  s.addText("Each box is a Docker container running its own a653rs-linux hypervisor.", {
    x: 0.8, y: 6.3, w: 12, h: 0.3, fontFace: F.body, fontSize: 12, color: C.muted, italic: true,
  });
  s.addText("Boxes talk over a virtual AFDX network (UDP on a Docker bridge).", {
    x: 0.8, y: 6.6, w: 12, h: 0.3, fontFace: F.body, fontSize: 12, color: C.muted, italic: true,
  });

  footer(s, 5);
}

// ============================================================
// Slide 6 — Inside one CPM: major frame timeline
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "05 — Schedule", "Inside one CPM — the 40 ms major frame");

  // Frame bar background
  const fx = 0.8, fy = 2.6, fw = 11.7, fh = 1.1;
  s.addShape(pres.shapes.RECTANGLE, {
    x: fx, y: fy, w: fw, h: fh, fill: { color: C.white }, line: { color: C.rule, width: 1 },
  });

  // 4 slot blocks: 3ms gateway, 2ms app1, 2ms app2, 33ms idle
  const slots = [
    { label: "gateway",      ms: 3,  color: C.deep },
    { label: "fuel_qty",     ms: 2,  color: C.teal },
    { label: "transfer_pump",ms: 2,  color: C.teal },
    { label: "idle margin",  ms: 33, color: C.rule },
  ];
  let xs = fx;
  const px_per_ms = fw / 40;
  slots.forEach((sl) => {
    const w = sl.ms * px_per_ms;
    s.addShape(pres.shapes.RECTANGLE, {
      x: xs, y: fy, w: w, h: fh, fill: { color: sl.color }, line: { color: sl.color },
    });
    if (sl.ms >= 2) {
      s.addText(sl.label, {
        x: xs, y: fy, w: w, h: fh,
        fontFace: F.body, fontSize: sl.ms >= 6 ? 12 : 9,
        color: sl.color === C.rule ? C.muted : C.white,
        bold: true, align: "center", valign: "middle", margin: 0,
      });
      s.addText(`${sl.ms} ms`, {
        x: xs, y: fy + fh - 0.05, w: w, h: 0.25,
        fontFace: F.body, fontSize: 9,
        color: sl.color === C.rule ? C.muted : C.sky,
        align: "center", margin: 0,
      });
    }
    xs += w;
  });

  // Axis ticks 0..40 ms
  for (let t = 0; t <= 40; t += 10) {
    const tx = fx + t * px_per_ms;
    s.addShape(pres.shapes.LINE, {
      x: tx, y: fy + fh, w: 0, h: 0.15, line: { color: C.ink, width: 1 },
    });
    s.addText(`${t}ms`, {
      x: tx - 0.4, y: fy + fh + 0.18, w: 0.8, h: 0.25,
      fontFace: F.body, fontSize: 9, color: C.muted, align: "center", margin: 0,
    });
  }

  // Annotation
  s.addText("33 ms idle is intentional — headroom for the partition you will add.", {
    x: 0.8, y: 4.6, w: 12, h: 0.4, fontFace: F.body, fontSize: 14, color: C.ink, italic: true,
  });

  s.addText([
    { text: "Static, repeating, deterministic.", options: { bold: true, color: C.deep, breakLine: true } },
    { text: "Every partition gets exactly one window per frame. Misses are catastrophic.", options: { color: C.muted } },
  ], { x: 0.8, y: 5.4, w: 12, h: 1.2, fontFace: F.body, fontSize: 15, margin: 0 });

  footer(s, 6);
}

// ============================================================
// Slide 7 — Sampling ports flow
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "06 — Inter-partition comms", "Sampling ports across CPMs");

  // 4-box flow: sensor -> gateway -> peer_gateway -> consumer
  const boxes = [
    { x: 0.8,  l: "sensor_partition", sub: "in CPM-X",        c: C.teal },
    { x: 4.0,  l: "cpm_x_gateway",    sub: "AFDX TX",          c: C.deep },
    { x: 7.2,  l: "cpm_y_gateway",    sub: "AFDX RX",          c: C.deep },
    { x: 10.4, l: "consumer_part",    sub: "in CPM-Y",         c: C.teal },
  ];
  boxes.forEach((b) => {
    s.addShape(pres.shapes.RECTANGLE, {
      x: b.x, y: 3.0, w: 2.5, h: 1.2, fill: { color: C.white }, line: { color: b.c, width: 2 },
    });
    s.addShape(pres.shapes.RECTANGLE, {
      x: b.x, y: 3.0, w: 2.5, h: 0.3, fill: { color: b.c }, line: { color: b.c },
    });
    s.addText("", { x: b.x, y: 3.0, w: 2.5, h: 0.3 }); // header bar
    s.addText([
      { text: b.l, options: { fontSize: 13, color: C.ink, bold: true, breakLine: true } },
      { text: b.sub, options: { fontSize: 10, color: C.muted } },
    ], { x: b.x + 0.1, y: 3.35, w: 2.3, h: 0.85, align: "center", valign: "middle", fontFace: F.body, margin: 0 });
  });

  // Arrows + labels between
  const arrows = [
    { x: 3.3,  label: "sampling port" },
    { x: 6.5,  label: "UDP over AFDX" },
    { x: 9.7,  label: "sampling port" },
  ];
  arrows.forEach((a) => {
    s.addShape(pres.shapes.LINE, {
      x: a.x, y: 3.6, w: 0.7, h: 0, line: { color: C.amber, width: 3, endArrowType: "triangle" },
    });
    s.addText(a.label, {
      x: a.x - 0.2, y: 3.85, w: 1.1, h: 0.3,
      fontFace: F.body, fontSize: 9, color: C.muted, align: "center", margin: 0,
    });
  });

  s.addText([
    { text: "Inside a CPM:", options: { bold: true, color: C.deep } },
    { text: " ports are shared memory.", options: {} },
    { text: "  ·  ", options: { color: C.rule } },
    { text: "Between CPMs:", options: { bold: true, color: C.deep } },
    { text: " gateways serialize port content to UDP.", options: {} },
  ], { x: 0.8, y: 5.2, w: 12, h: 0.6, fontFace: F.body, fontSize: 14, color: C.ink, margin: 0 });

  s.addText("Application code is identical in both cases — that is the whole point.", {
    x: 0.8, y: 6.0, w: 12, h: 0.5, fontFace: F.body, fontSize: 13, color: C.muted, italic: true,
  });

  footer(s, 7);
}

// ============================================================
// Slide 8 — The domains (table with DAL color coding)
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "07 — Domains", "Four domain CPMs and what they do");

  const dalColor = { B: C.deep, C: C.teal, D: C.muted };
  const rows = [
    ["CPM-L", "Landing Gear",           "B", "Gear lever, braking, steering"],
    ["CPM-F", "Fuel",                   "C", "Fuel quantity, transfer pumps, balance"],
    ["CPM-A", "ECS / Air Conditioning", "D", "Bleed air, cabin temperature"],
    ["CPM-E", "Electrical / Energy",    "B", "Generators, load shedding"],
  ];

  // header
  const headFill = C.deep;
  const headerRow = [
    { text: "CPM", options: { bold: true, color: C.white, fill: { color: headFill }, fontSize: 13, margin: 6, valign: "middle" } },
    { text: "Domain", options: { bold: true, color: C.white, fill: { color: headFill }, fontSize: 13, margin: 6, valign: "middle" } },
    { text: "DAL", options: { bold: true, color: C.white, fill: { color: headFill }, fontSize: 13, margin: 6, valign: "middle", align: "center" } },
    { text: "Function", options: { bold: true, color: C.white, fill: { color: headFill }, fontSize: 13, margin: 6, valign: "middle" } },
  ];
  const dataRows = rows.map((r, idx) => {
    const isF = r[0] === "CPM-F";
    const baseFill = isF ? "FFF4E1" : (idx % 2 ? C.paper : C.white);
    return [
      { text: r[0], options: { bold: true, color: isF ? C.amber : C.ink, fill: { color: baseFill }, fontSize: 14, margin: 6, valign: "middle" } },
      { text: r[1], options: { color: C.ink, fill: { color: baseFill }, fontSize: 13, margin: 6, valign: "middle" } },
      { text: r[2], options: { bold: true, color: dalColor[r[2]], fill: { color: baseFill }, fontSize: 14, margin: 6, valign: "middle", align: "center" } },
      { text: r[3], options: { color: C.ink, fill: { color: baseFill }, fontSize: 13, margin: 6, valign: "middle" } },
    ];
  });

  s.addTable([headerRow, ...dataRows], {
    x: 0.8, y: 2.3, w: 11.7,
    colW: [1.6, 3.0, 1.2, 5.9],
    border: { type: "solid", pt: 0.5, color: C.rule },
    fontFace: F.body,
  });

  // Highlight
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.8, y: 5.8, w: 11.7, h: 1.0, fill: { color: C.ink }, line: { color: C.ink },
  });
  s.addText([
    { text: "Today everyone works in CPM-F. ", options: { bold: true, color: C.amber, fontSize: 16, breakLine: true } },
    { text: "DAL C — fuel loss leads to engine starvation, not direct loss of aircraft.", options: { color: C.sky, fontSize: 12 } },
  ], { x: 1.0, y: 5.95, w: 11.3, h: 0.75, fontFace: F.body, margin: 0, valign: "middle" });

  footer(s, 8);
}

// ============================================================
// Slide 9 — Zooming in: CPM-F
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "08 — Zoom: CPM-F", "Four partitions on one Fuel CPM");

  // Gateway block at top
  s.addShape(pres.shapes.RECTANGLE, {
    x: 4.5, y: 2.2, w: 4.2, h: 0.85, fill: { color: C.deep }, line: { color: C.deep },
  });
  s.addText([
    { text: "cpm_f_gateway", options: { bold: true, fontSize: 14, color: C.white, breakLine: true } },
    { text: "to/from AFDX network", options: { fontSize: 10, color: C.sky } },
  ], { x: 4.5, y: 2.2, w: 4.2, h: 0.85, align: "center", valign: "middle", fontFace: F.body, margin: 0 });

  // bus line
  s.addShape(pres.shapes.LINE, { x: 6.6, y: 3.05, w: 0, h: 0.4, line: { color: C.ink, width: 2 } });
  s.addShape(pres.shapes.LINE, { x: 1.5,  y: 3.45, w: 10.3, h: 0,  line: { color: C.ink, width: 2 } });

  // 4 partitions
  const parts = [
    { x: 1.1,  name: "fuel_quantity",   role: "given (reference)", c: C.teal },
    { x: 4.4,  name: "transfer_pump",   role: "given (reference)", c: C.teal },
    { x: 7.7,  name: "fuel_controller", role: "← YOU BUILD THIS",  c: C.amber },
    { x: 11.0, name: "(spare slot)",    role: "—",                  c: C.muted },
  ];
  parts.forEach((p) => {
    s.addShape(pres.shapes.LINE, { x: p.x + 1.1, y: 3.45, w: 0, h: 0.45, line: { color: C.ink, width: 2 } });
    s.addShape(pres.shapes.RECTANGLE, {
      x: p.x, y: 3.9, w: 2.2, h: 1.8,
      fill: { color: C.white }, line: { color: p.c, width: p.c === C.amber ? 3 : 1.5 },
    });
    s.addShape(pres.shapes.RECTANGLE, {
      x: p.x, y: 3.9, w: 2.2, h: 0.35, fill: { color: p.c }, line: { color: p.c },
    });
    s.addText(p.name, {
      x: p.x, y: 3.9, w: 2.2, h: 0.35,
      fontFace: F.body, fontSize: 11, color: C.white, bold: true, align: "center", valign: "middle", margin: 0,
    });
    s.addText(p.role, {
      x: p.x + 0.1, y: 4.35, w: 2.0, h: 1.25,
      fontFace: F.body, fontSize: 11, color: p.c === C.amber ? C.amber : C.ink,
      bold: p.c === C.amber, align: "center", valign: "middle", margin: 0,
    });
  });

  s.addText("You will configure its slot in cpm_f.yaml, then implement select_source_tank in controller.rs.", {
    x: 0.8, y: 6.2, w: 12, h: 0.5, fontFace: F.body, fontSize: 14, color: C.ink, italic: true,
  });

  footer(s, 9);
}

// ============================================================
// Slide 10 — Live demo
// ============================================================
{
  const s = pres.addSlide();
  darkChrome(s);

  // Header in dark
  s.addShape(pres.shapes.RECTANGLE, { x: 0.6, y: 0.6, w: 0.18, h: 0.32, fill: { color: C.amber }, line: { color: C.amber } });
  s.addText("09 — LIVE DEMO", { x: 0.9, y: 0.55, w: 6, h: 0.4, fontFace: F.body, fontSize: 11, color: C.amber, bold: true, charSpacing: 4, margin: 0 });
  s.addText("Avio23 running, right now", { x: 0.6, y: 1.0, w: 12.3, h: 0.9, fontFace: F.head, fontSize: 36, color: C.white, bold: true, margin: 0 });

  // Terminal-style box
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.8, y: 2.4, w: 11.7, h: 3.6,
    fill: { color: "0A0F1A" }, line: { color: C.teal, width: 1 },
  });
  // Mac-style traffic lights
  ["E06C75", "E5C07B", "98C379"].forEach((col, i) => {
    s.addShape(pres.shapes.OVAL, { x: 1.0 + i * 0.25, y: 2.55, w: 0.18, h: 0.18, fill: { color: col }, line: { color: col } });
  });
  s.addText("avio23 — bash", {
    x: 0.8, y: 2.45, w: 11.7, h: 0.3, fontFace: F.body, fontSize: 10, color: C.muted, align: "center", margin: 0,
  });
  s.addText([
    { text: "$ docker compose up -d", options: { color: C.sky, breakLine: true } },
    { text: " Creating sim_gateway, cpm_l_node, cpm_f_node, cpm_a_node, cpm_e_node", options: { color: C.muted, breakLine: true } },
    { text: "$ docker compose logs -f sim_gateway", options: { color: C.sky, breakLine: true } },
    { text: " [sim_gateway] tx tank_l=247.3L tank_r=247.1L flow=0.10L/s", options: { color: C.white, breakLine: true } },
    { text: " [cpm_f]      rx fuel_qty total=494.4L imbalance=0.2L", options: { color: C.white, breakLine: true } },
    { text: " [cpm_f]      controller selected source=Left, age_since_switch=2.4s", options: { color: C.amber, breakLine: true } },
    { text: " [sim_gateway] tx tank_l=247.0L tank_r=247.1L flow=0.10L/s ...", options: { color: C.white } },
  ], {
    x: 1.0, y: 2.85, w: 11.3, h: 3.0,
    fontFace: "Consolas", fontSize: 12, margin: 0,
  });

  s.addText("This is what your passing solution looks like by the end of the session.", {
    x: 0.8, y: 6.4, w: 12, h: 0.4, fontFace: F.body, fontSize: 14, color: C.sky, italic: true,
  });

  s.addText("10", { x: W - 1.6, y: H - 0.45, w: 1.0, h: 0.3, fontFace: F.body, fontSize: 9, color: C.muted, align: "right" });
}

// ============================================================
// Slide 11 — Why bother with IMA in a course
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "10 — Why this course", "Why bother with IMA at all?");

  const items = [
    { num: "→", h: "It's how real cockpits are written",      d: "The pattern in Avio23 is the pattern in every modern commercial cockpit." },
    { num: "→", h: "Integration is the avionics skill",        d: "Allocating ports and slots is what distinguishes avionics from generic embedded." },
    { num: "→", h: "You learn the standards' vocabulary",      d: "DO-297, ARP4754A, ARINC 653 — all written in the language you use today." },
  ];
  items.forEach((it, i) => {
    const y = 2.4 + i * 1.4;
    s.addShape(pres.shapes.RECTANGLE, { x: 0.8, y: y, w: 0.08, h: 1.1, fill: { color: C.amber }, line: { color: C.amber } });
    s.addText(it.h, {
      x: 1.1, y: y + 0.05, w: 11.5, h: 0.5,
      fontFace: F.head, fontSize: 20, color: C.ink, bold: true, margin: 0,
    });
    s.addText(it.d, {
      x: 1.1, y: y + 0.55, w: 11.5, h: 0.6,
      fontFace: F.body, fontSize: 14, color: C.muted, margin: 0,
    });
  });

  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.8, y: 6.5, w: 11.7, h: 0.45, fill: { color: C.ink }, line: { color: C.ink },
  });
  s.addText("The hard part isn't the algorithm — it's the constraints around it.", {
    x: 0.8, y: 6.5, w: 11.7, h: 0.45,
    fontFace: F.body, fontSize: 13, color: C.sky, italic: true, align: "center", valign: "middle", margin: 0,
  });

  footer(s, 11);
}

// ============================================================
// Slide 12 — Today's assignment (two columns)
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "11 — Assignment", "Today: Fuel Management System");

  s.addText("Two deliverables. Both graded automatically.", {
    x: 0.8, y: 2.3, w: 12, h: 0.4, fontFace: F.body, fontSize: 16, color: C.muted, italic: true, margin: 0,
  });

  const cards = [
    {
      x: 0.8, c: C.deep,
      label: "PART 1",
      title: "Configuration",
      body: "Add a fuel_controller partition to cpm_f.yaml. Pick a time window. Wire the sampling ports for sensors in and actuators out.",
      file: "cpm_f.yaml",
      time: "~15 minutes",
    },
    {
      x: 6.9, c: C.teal,
      label: "PART 2",
      title: "Controller logic",
      body: "Implement select_source_tank(left, right, current, secs_since_switch) → Tank. Keep |L−R| ≤ 10 L. Respect the 0.8 s valve cooldown.",
      file: "src/controller.rs",
      time: "~40 minutes",
    },
  ];

  cards.forEach((c) => {
    s.addShape(pres.shapes.RECTANGLE, {
      x: c.x, y: 3.0, w: 5.6, h: 3.7,
      fill: { color: C.white }, line: { color: C.rule, width: 0.75 },
      shadow: { type: "outer", color: "000000", blur: 12, offset: 2, angle: 90, opacity: 0.1 },
    });
    s.addShape(pres.shapes.RECTANGLE, { x: c.x, y: 3.0, w: 5.6, h: 0.35, fill: { color: c.c }, line: { color: c.c } });
    s.addText(c.label, { x: c.x + 0.3, y: 3.0, w: 5.0, h: 0.35,
      fontFace: F.body, fontSize: 11, color: C.white, bold: true, charSpacing: 4, valign: "middle", margin: 0 });
    s.addText(c.title, { x: c.x + 0.3, y: 3.5, w: 5.0, h: 0.5,
      fontFace: F.head, fontSize: 22, color: C.ink, bold: true, margin: 0 });
    s.addText(c.body, { x: c.x + 0.3, y: 4.05, w: 5.0, h: 1.6,
      fontFace: F.body, fontSize: 13, color: C.ink, margin: 0 });
    s.addText([
      { text: "File: ", options: { color: C.muted } },
      { text: c.file, options: { color: c.c, bold: true } },
    ], { x: c.x + 0.3, y: 5.7, w: 5.0, h: 0.35, fontFace: "Consolas", fontSize: 12, margin: 0 });
    s.addText(c.time, {
      x: c.x + 0.3, y: 6.1, w: 5.0, h: 0.35,
      fontFace: F.body, fontSize: 12, color: C.muted, margin: 0,
    });
  });

  footer(s, 12);
}

// ============================================================
// Slide 13 — The constraints (stat callouts)
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "12 — Constraints", "What makes this interesting");

  // 3 big stat cards
  const stats = [
    { x: 0.8,  v: "10 L", k: "Balance",   d: "Max tank imbalance under any flight condition",     c: C.deep },
    { x: 5.05, v: "0.8 s",k: "Cooldown",  d: "Min interval between pump-valve switches",          c: C.teal },
    { x: 9.3,  v: "40 ms",k: "Cycle",     d: "Major frame — controller is invoked every tick",    c: C.amber },
  ];
  stats.forEach((s2) => {
    s.addShape(pres.shapes.RECTANGLE, {
      x: s2.x, y: 2.4, w: 3.95, h: 3.0,
      fill: { color: C.white }, line: { color: s2.c, width: 1.5 },
      shadow: { type: "outer", color: "000000", blur: 10, offset: 2, angle: 90, opacity: 0.08 },
    });
    s.addText(s2.v, {
      x: s2.x, y: 2.55, w: 3.95, h: 1.4,
      fontFace: F.head, fontSize: 64, color: s2.c, bold: true, align: "center", valign: "middle", margin: 0,
    });
    s.addText(s2.k.toUpperCase(), {
      x: s2.x, y: 3.85, w: 3.95, h: 0.4,
      fontFace: F.body, fontSize: 12, color: C.ink, bold: true, charSpacing: 4, align: "center", margin: 0,
    });
    s.addText(s2.d, {
      x: s2.x + 0.3, y: 4.3, w: 3.35, h: 1.0,
      fontFace: F.body, fontSize: 12, color: C.muted, align: "center", margin: 0,
    });
  });

  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.8, y: 5.9, w: 11.7, h: 1.0, fill: { color: C.ink }, line: { color: C.ink },
  });
  s.addText([
    { text: "The interesting bit:", options: { bold: true, color: C.amber, fontSize: 14, breakLine: true } },
    { text: "the cooldown and the balance requirement are coupled. A naive ", options: { color: C.white, fontSize: 12 } },
    { text: "\"switch whenever imbalanced\"", options: { color: C.sky, italic: true, fontSize: 12 } },
    { text: " controller violates the cooldown. You have to think about state.", options: { color: C.white, fontSize: 12 } },
  ], { x: 1.0, y: 6.0, w: 11.3, h: 0.8, fontFace: F.body, margin: 0, valign: "middle" });

  footer(s, 13);
}

// ============================================================
// Slide 14 — Toolchain
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "13 — Toolchain", "Your laptop, nothing else");

  // Three command blocks
  const cmds = [
    { y: 2.4,  h: "1.  Validate your config",        cmd: "$ cargo run --bin test_bench -- --validate-config",         hint: "Catches missing partition entries, port mis-wiring, frame budget overruns." },
    { y: 3.7,  h: "2.  Run a scenario",              cmd: "$ cargo run --bin test_bench",                                hint: "Replays a 600-s flight scenario; writes out/trace.csv for plotting." },
    { y: 5.0,  h: "3.  Run the grader (= your grade)", cmd: "$ cargo test --release -- --nocapture",                       hint: "Identical to what I'll run on your submission at the end of the session." },
  ];

  cmds.forEach((c) => {
    s.addText(c.h, { x: 0.8, y: c.y, w: 12, h: 0.35,
      fontFace: F.head, fontSize: 16, color: C.ink, bold: true, margin: 0 });
    s.addShape(pres.shapes.RECTANGLE, {
      x: 0.8, y: c.y + 0.4, w: 11.7, h: 0.5,
      fill: { color: C.ink }, line: { color: C.ink },
    });
    s.addText(c.cmd, {
      x: 1.0, y: c.y + 0.4, w: 11.5, h: 0.5,
      fontFace: "Consolas", fontSize: 13, color: C.sky, valign: "middle", margin: 0,
    });
    s.addText(c.hint, {
      x: 0.8, y: c.y + 0.95, w: 12, h: 0.35,
      fontFace: F.body, fontSize: 11, color: C.muted, italic: true, margin: 0,
    });
  });

  s.addText([
    { text: "No Docker, no Linux VM, no AFDX setup on your laptop.", options: { bold: true, color: C.deep, breakLine: true } },
    { text: "Pure Rust crate. cargo run, cargo test. That's it.", options: { color: C.muted } },
  ], { x: 0.8, y: 6.4, w: 12, h: 0.6, fontFace: F.body, fontSize: 13, margin: 0 });

  footer(s, 14);
}

// ============================================================
// Slide 15 — The plan for 2 hours
// ============================================================
{
  const s = pres.addSlide();
  lightChrome(s);
  lightHeader(s, "14 — Schedule", "Your next two hours");

  const rows = [
    ["0:00 – 0:20", "This intro",                            "tutor"],
    ["0:20 – 0:30", "Assignment briefing",                   "tutor"],
    ["0:30 – 0:45", "Part 1 — Configuration",                "you"],
    ["0:45 – 1:25", "Part 2 — Controller logic",             "you"],
    ["1:25 – 1:50", "Grading + live demo on the real stack", "both"],
    ["1:50 – 2:00", "Wrap-up, grades, leave",                "—"],
  ];

  const tag = (s2) => ({
    tutor: { c: C.deep, t: "TUTOR" },
    you:   { c: C.amber, t: "YOU" },
    both:  { c: C.teal, t: "BOTH" },
    "—":   { c: C.muted, t: "—" },
  }[s2]);

  rows.forEach((r, i) => {
    const y = 2.4 + i * 0.65;
    if (i % 2 === 0) {
      s.addShape(pres.shapes.RECTANGLE, {
        x: 0.8, y: y, w: 11.7, h: 0.6, fill: { color: C.white }, line: { color: C.rule, width: 0.5 },
      });
    } else {
      s.addShape(pres.shapes.RECTANGLE, {
        x: 0.8, y: y, w: 11.7, h: 0.6, fill: { color: C.paper }, line: { color: C.rule, width: 0.5 },
      });
    }
    s.addText(r[0], {
      x: 1.0, y: y, w: 2.5, h: 0.6,
      fontFace: "Consolas", fontSize: 14, color: C.deep, bold: true, valign: "middle", margin: 0,
    });
    s.addText(r[1], {
      x: 3.7, y: y, w: 7.3, h: 0.6,
      fontFace: F.body, fontSize: 14, color: C.ink, valign: "middle", margin: 0,
    });
    const T = tag(r[2]);
    s.addShape(pres.shapes.RECTANGLE, {
      x: 11.1, y: y + 0.15, w: 1.2, h: 0.3, fill: { color: T.c }, line: { color: T.c },
    });
    s.addText(T.t, {
      x: 11.1, y: y + 0.15, w: 1.2, h: 0.3,
      fontFace: F.body, fontSize: 10, color: C.white, bold: true, charSpacing: 3, align: "center", valign: "middle", margin: 0,
    });
  });

  s.addText("Questions?", {
    x: 0.8, y: 6.6, w: 12, h: 0.5,
    fontFace: F.head, fontSize: 22, color: C.deep, bold: true, margin: 0,
  });
  s.addText("Then let's go.", {
    x: 4.2, y: 6.65, w: 12, h: 0.4,
    fontFace: F.body, fontSize: 16, color: C.muted, italic: true, margin: 0,
  });

  footer(s, 15);
}

// ============================================================
const outPath = path.resolve(__dirname, "avio23-intro.pptx");
pres.writeFile({ fileName: outPath }).then((p) => {
  console.log("WROTE:", p);
});
