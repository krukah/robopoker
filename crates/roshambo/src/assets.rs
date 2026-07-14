//! Embedded HTML templates for the simplex visualizations.
//!
//! Previously standalone files under `assets/`; kept here as Rust string
//! constants so the repository stays Rust-only. Each template carries
//! `__P1_DATA__` / `__P2_DATA__` placeholders that [`crate::simplex`]
//! substitutes with per-player snapshot JSON at generation time.

/// Standalone SVG 2D policy-simplex viewer.
pub const SIMPLEX_2D: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>RPS Simplex Trajectory</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: #0a0a0a; color: #e0e0e0; font-family: 'SF Mono', 'Fira Code', monospace; display: flex; height: 100vh; }
#canvas { flex: 1; display: flex; align-items: center; justify-content: center; }
#panel { width: 380px; padding: 24px; border-left: 1px solid #222; overflow-y: auto; font-size: 13px; }
svg { filter: drop-shadow(0 0 20px rgba(100,100,255,0.08)); }
.controls { display: flex; gap: 8px; align-items: center; margin-bottom: 16px; flex-wrap: wrap; }
.controls button { background: #1a1a2e; border: 1px solid #333; color: #ccc; padding: 6px 14px; cursor: pointer; font-family: inherit; font-size: 12px; }
.controls button:hover { background: #2a2a4e; }
.controls button.active { background: #3a3a6e; border-color: #667; }
input[type=range] { flex: 1; accent-color: #667; min-width: 120px; }
.label { color: #888; font-size: 11px; text-transform: uppercase; letter-spacing: 1px; margin: 14px 0 6px; }
table { width: 100%; border-collapse: collapse; margin: 4px 0; table-layout: fixed; }
td, th { padding: 2px 0; text-align: right; font-size: 12px; font-variant-numeric: tabular-nums; white-space: pre; }
th { color: #555; font-weight: normal; }
th:first-child { text-align: left; color: #666; width: 36px; }
.p1 { color: #6699ff; }
.p2 { color: #ff6666; }
tr.p1-row th { color: #6699ff; }
tr.p1-row td { color: #8ab4ff; }
tr.p2-row th { color: #ff6666; }
tr.p2-row td { color: #ff9999; }
.nash { color: #44dd88; }
#epoch { font-size: 22px; font-weight: bold; margin-bottom: 12px; letter-spacing: -0.5px; }
.sep { border-top: 1px solid #1a1a1a; margin: 6px 0; }
</style>
</head>
<body>
<div id="canvas">
<svg id="simplex" width="600" height="560" viewBox="-20 -20 640 600"></svg>
</div>
<div id="panel">
<div id="epoch">t = 0</div>
<div class="controls">
  <button id="play" class="active">Play</button>
  <button id="pause">Pause</button>
  <input type="range" id="slider" min="0" max="0" value="0">
</div>
<div class="controls">
  <button id="speed1">1x</button>
  <button id="speed2" class="active">4x</button>
  <button id="speed4">16x</button>
  <button id="player-toggle">Show: Both</button>
</div>
<div class="label">Iterated Policy</div>
<table id="iterated-table">
<thead><tr><th></th><th>R</th><th>P</th><th>S</th></tr></thead>
<tbody></tbody>
</table>
<div class="label">Averaged Strategy</div>
<table id="averaged-table">
<thead><tr><th></th><th>R</th><th>P</th><th>S</th></tr></thead>
<tbody></tbody>
</table>
<div class="sep"></div>
<div class="label p1">P1 Encounters</div>
<table id="p1-table"><thead><tr><th></th><th>R</th><th>P</th><th>S</th></tr></thead><tbody></tbody></table>
<div class="label p2">P2 Encounters</div>
<table id="p2-table"><thead><tr><th></th><th>R</th><th>P</th><th>S</th></tr></thead><tbody></tbody></table>
</div>
<script>
const P1 = __P1_DATA__;
const P2 = __P2_DATA__;
const SZ = 560;
const PAD = 20;
const TRI = [
  [PAD, SZ - PAD],
  [SZ - PAD, SZ - PAD],
  [SZ / 2, PAD + (SZ - 2 * PAD) * (1 - Math.sqrt(3) / 2)]
];
function bary2xy(bary) {
  const [r, p, s] = bary;
  return [
    TRI[0][0] * r + TRI[1][0] * p + TRI[2][0] * s,
    TRI[0][1] * r + TRI[1][1] * p + TRI[2][1] * s
  ];
}
const NASH = bary2xy([0.4, 0.4, 0.2]);
const svg = document.getElementById('simplex');
function el(tag, attrs) {
  const e = document.createElementNS('http://www.w3.org/2000/svg', tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  return e;
}
// triangle
svg.appendChild(el('polygon', {
  points: TRI.map(p => p.join(',')).join(' '),
  fill: 'none', stroke: '#333', 'stroke-width': '1.5'
}));
// vertex labels
for (const [txt, pos, off] of [['R', TRI[0], [-14, 18]], ['P', TRI[1], [6, 18]], ['S', TRI[2], [-4, -10]]]) {
  const t = el('text', { x: pos[0] + off[0], y: pos[1] + off[1], fill: '#888', 'font-size': '16', 'font-family': 'inherit' });
  t.textContent = txt;
  svg.appendChild(t);
}
// nash marker
const nashG = el('g', {});
nashG.appendChild(el('circle', { cx: NASH[0], cy: NASH[1], r: '6', fill: 'none', stroke: '#44dd88', 'stroke-width': '1.5', 'stroke-dasharray': '3,3' }));
nashG.appendChild(el('circle', { cx: NASH[0], cy: NASH[1], r: '2', fill: '#44dd88' }));
svg.appendChild(nashG);
// trail groups
const p1trail = el('g', { opacity: '0.5' });
const p2trail = el('g', { opacity: '0.5' });
const p1avg   = el('g', { opacity: '0.9' });
const p2avg   = el('g', { opacity: '0.9' });
svg.appendChild(p1trail); svg.appendChild(p2trail);
svg.appendChild(p1avg);   svg.appendChild(p2avg);
// current dots
const p1dot    = el('circle', { r: '5', fill: '#6699ff', stroke: '#fff', 'stroke-width': '1' });
const p2dot    = el('circle', { r: '5', fill: '#ff6666', stroke: '#fff', 'stroke-width': '1' });
const p1avgdot = el('circle', { r: '6', fill: 'none', stroke: '#6699ff', 'stroke-width': '2' });
const p2avgdot = el('circle', { r: '6', fill: 'none', stroke: '#ff6666', 'stroke-width': '2' });
svg.appendChild(p1dot); svg.appendChild(p2dot);
svg.appendChild(p1avgdot); svg.appendChild(p2avgdot);
// state
let frame = 0;
let playing = true;
let speed = 4;
let showPlayer = 'both';
const slider = document.getElementById('slider');
slider.max = P1.length - 1;
// formatters
function fProb(v) { return (v >= 0 ? ' ' : '') + v.toFixed(3); }
function fReg(v)  { return (v >= 0 ? '+' : '') + v.toFixed(2); }
function fSci(v)  { return v.toExponential(2).replace('e+', 'e'); }
function fEv(v)   { return (v >= 0 ? '+' : '') + v.toFixed(4); }
function fVis(v)  { return String(v).padStart(7); }
function policyRow(cls, label, arr) {
  return `<tr class="${cls}-row"><th>${label}</th><td>${fProb(arr[0])}</td><td>${fProb(arr[1])}</td><td>${fProb(arr[2])}</td></tr>`;
}
function updatePolicyTable(id, s1, s2, field) {
  document.querySelector(`#${id} tbody`).innerHTML =
    policyRow('p1', 'P1', s1[field]) +
    policyRow('p2', 'P2', s2[field]);
}
function updateTable(id, snap) {
  document.querySelector(`#${id} tbody`).innerHTML =
    `<tr><th>reg</th><td>${fReg(snap.regrets[0])}</td><td>${fReg(snap.regrets[1])}</td><td>${fReg(snap.regrets[2])}</td></tr>` +
    `<tr><th>wgt</th><td>${fSci(snap.weights[0])}</td><td>${fSci(snap.weights[1])}</td><td>${fSci(snap.weights[2])}</td></tr>` +
    `<tr><th>ev</th><td>${fEv(snap.payoffs[0])}</td><td>${fEv(snap.payoffs[1])}</td><td>${fEv(snap.payoffs[2])}</td></tr>` +
    `<tr><th>vis</th><td>${fVis(snap.visits[0])}</td><td>${fVis(snap.visits[1])}</td><td>${fVis(snap.visits[2])}</td></tr>`;
}
function renderFrame() {
  const s1 = P1[frame], s2 = P2[frame];
  document.getElementById('epoch').textContent = `t = ${s1.epoch}`;
  slider.value = frame;
  // iterated dots
  const [x1, y1] = bary2xy(s1.iterated);
  const [x2, y2] = bary2xy(s2.iterated);
  p1dot.setAttribute('cx', x1); p1dot.setAttribute('cy', y1);
  p2dot.setAttribute('cx', x2); p2dot.setAttribute('cy', y2);
  // averaged dots
  const [ax1, ay1] = bary2xy(s1.averaged);
  const [ax2, ay2] = bary2xy(s2.averaged);
  p1avgdot.setAttribute('cx', ax1); p1avgdot.setAttribute('cy', ay1);
  p2avgdot.setAttribute('cx', ax2); p2avgdot.setAttribute('cy', ay2);
  // trails
  if (frame > 0) {
    const prev1 = bary2xy(P1[frame-1].iterated);
    p1trail.appendChild(el('circle', { cx: prev1[0], cy: prev1[1], r: '1.5', fill: '#6699ff', opacity: '0.4' }));
    const prev2 = bary2xy(P2[frame-1].iterated);
    p2trail.appendChild(el('circle', { cx: prev2[0], cy: prev2[1], r: '1.5', fill: '#ff6666', opacity: '0.4' }));
    const aprev1 = bary2xy(P1[frame-1].averaged);
    p1avg.appendChild(el('line', { x1: aprev1[0], y1: aprev1[1], x2: ax1, y2: ay1, stroke: '#6699ff', 'stroke-width': '1', opacity: '0.6' }));
    const aprev2 = bary2xy(P2[frame-1].averaged);
    p2avg.appendChild(el('line', { x1: aprev2[0], y1: aprev2[1], x2: ax2, y2: ay2, stroke: '#ff6666', 'stroke-width': '1', opacity: '0.6' }));
  }
  // visibility
  const show1 = showPlayer !== 'p2';
  const show2 = showPlayer !== 'p1';
  p1trail.style.display  = show1 ? '' : 'none';
  p1avg.style.display    = show1 ? '' : 'none';
  p1dot.style.display    = show1 ? '' : 'none';
  p1avgdot.style.display = show1 ? '' : 'none';
  p2trail.style.display  = show2 ? '' : 'none';
  p2avg.style.display    = show2 ? '' : 'none';
  p2dot.style.display    = show2 ? '' : 'none';
  p2avgdot.style.display = show2 ? '' : 'none';
  // data panel
  updatePolicyTable('iterated-table', s1, s2, 'iterated');
  updatePolicyTable('averaged-table', s1, s2, 'averaged');
  updateTable('p1-table', s1);
  updateTable('p2-table', s2);
}
// controls
document.getElementById('play').onclick = () => { playing = true; document.getElementById('play').classList.add('active'); document.getElementById('pause').classList.remove('active'); };
document.getElementById('pause').onclick = () => { playing = false; document.getElementById('pause').classList.add('active'); document.getElementById('play').classList.remove('active'); };
slider.oninput = () => { frame = parseInt(slider.value); clearTrails(); rebuildTrails(); renderFrame(); };
document.getElementById('speed1').onclick = () => { speed = 1; updateSpeedBtns(); };
document.getElementById('speed2').onclick = () => { speed = 4; updateSpeedBtns(); };
document.getElementById('speed4').onclick = () => { speed = 16; updateSpeedBtns(); };
function updateSpeedBtns() {
  for (const b of [document.getElementById('speed1'), document.getElementById('speed2'), document.getElementById('speed4')]) b.classList.remove('active');
  if (speed === 1) document.getElementById('speed1').classList.add('active');
  if (speed === 4) document.getElementById('speed2').classList.add('active');
  if (speed >= 16) document.getElementById('speed4').classList.add('active');
}
document.getElementById('player-toggle').onclick = () => {
  const btn = document.getElementById('player-toggle');
  if (showPlayer === 'both') { showPlayer = 'p1'; btn.textContent = 'Show: P1'; }
  else if (showPlayer === 'p1') { showPlayer = 'p2'; btn.textContent = 'Show: P2'; }
  else { showPlayer = 'both'; btn.textContent = 'Show: Both'; }
  renderFrame();
};
function clearTrails() { p1trail.innerHTML = ''; p2trail.innerHTML = ''; p1avg.innerHTML = ''; p2avg.innerHTML = ''; }
function rebuildTrails() {
  for (let i = 0; i < frame; i++) {
    const [cx, cy] = bary2xy(P1[i].iterated);
    p1trail.appendChild(el('circle', { cx, cy, r: '1.5', fill: '#6699ff', opacity: '0.4' }));
    const [cx2, cy2] = bary2xy(P2[i].iterated);
    p2trail.appendChild(el('circle', { cx: cx2, cy: cy2, r: '1.5', fill: '#ff6666', opacity: '0.4' }));
    if (i > 0) {
      const [px, py] = bary2xy(P1[i-1].averaged);
      const [nx, ny] = bary2xy(P1[i].averaged);
      p1avg.appendChild(el('line', { x1: px, y1: py, x2: nx, y2: ny, stroke: '#6699ff', 'stroke-width': '1', opacity: '0.6' }));
      const [px2, py2] = bary2xy(P2[i-1].averaged);
      const [nx2, ny2] = bary2xy(P2[i].averaged);
      p2avg.appendChild(el('line', { x1: px2, y1: py2, x2: nx2, y2: ny2, stroke: '#ff6666', 'stroke-width': '1', opacity: '0.6' }));
    }
  }
}
renderFrame();
let tick = 0;
setInterval(() => {
  if (!playing || frame >= P1.length - 1) return;
  tick++;
  if (tick % Math.max(1, Math.round(4 / speed)) === 0) {
    frame++;
    renderFrame();
  }
}, 16);
</script>
</body>
</html>
"#;

/// Standalone Three.js 3D simplex viewer.
pub const SIMPLEX_3D: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>RPS 3D Simplex Trajectory</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: #0a0a0a; color: #e0e0e0; font-family: 'SF Mono', 'Fira Code', monospace; display: flex; height: 100vh; overflow: hidden; }
#canvas { flex: 1; position: relative; }
#canvas canvas { display: block; width: 100% !important; height: 100% !important; }
#panel { width: 380px; padding: 24px; border-left: 1px solid #222; overflow-y: auto; font-size: 13px; }
.transport { display: flex; gap: 6px; align-items: center; margin-bottom: 14px; }
.transport input[type=range] { flex: 1; accent-color: #667; min-width: 80px; }
.ctrl-row { display: flex; gap: 12px; align-items: flex-end; margin-bottom: 16px; }
.ctrl-group { display: flex; flex-direction: column; gap: 3px; }
.ctrl-label { font-size: 9px; color: #555; text-transform: uppercase; letter-spacing: 0.5px; padding-left: 2px; }
button { background: #1a1a2e; border: 1px solid #333; color: #ccc; padding: 5px 12px; cursor: pointer; font-family: inherit; font-size: 12px; border-radius: 3px; }
button:hover { background: #2a2a4e; }
button.active { background: #3a3a6e; border-color: #667; }
.btn-group { display: flex; }
.btn-group button { border-radius: 0; border-right-width: 0; padding: 5px 10px; }
.btn-group button:first-child { border-radius: 3px 0 0 3px; }
.btn-group button:last-child { border-radius: 0 3px 3px 0; border-right-width: 1px; }
.btn-pair { display: flex; }
.btn-pair button { border-radius: 0; border-right-width: 0; padding: 5px 10px; min-width: 44px; text-align: center; }
.btn-pair button:first-child { border-radius: 3px 0 0 3px; }
.btn-pair button:last-child { border-radius: 0 3px 3px 0; border-right-width: 1px; }
.cycle-btn { min-width: 62px; text-align: center; }
.label { color: #888; font-size: 11px; text-transform: uppercase; letter-spacing: 1px; margin: 14px 0 6px; }
table { width: 100%; border-collapse: collapse; margin: 4px 0; table-layout: fixed; }
td, th { padding: 2px 0; text-align: right; font-size: 12px; font-variant-numeric: tabular-nums; white-space: pre; }
th { color: #555; font-weight: normal; }
th:first-child { text-align: left; color: #666; width: 36px; }
.p1 { color: #6699ff; }
.p2 { color: #ff6666; }
tr.p1-row th { color: #6699ff; }
tr.p1-row td { color: #8ab4ff; }
tr.p2-row th { color: #ff6666; }
tr.p2-row td { color: #ff9999; }
.nash { color: #44dd88; }
#epoch { font-size: 22px; font-weight: bold; margin-bottom: 12px; letter-spacing: -0.5px; }
.sep { border-top: 1px solid #1a1a1a; margin: 6px 0; }
.label-3d { position: absolute; color: #888; font-family: 'SF Mono', 'Fira Code', monospace; font-size: 14px; pointer-events: none; user-select: none; }
</style>
<script type="importmap">
{
  "imports": {
    "three": "https://cdn.jsdelivr.net/npm/three@0.170.0/build/three.module.js",
    "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.170.0/examples/jsm/"
  }
}
</script>
</head>
<body>
<div id="canvas"></div>
<div id="panel">
<div id="epoch">t = 0</div>
<div class="transport">
  <div class="btn-pair">
    <button id="play" class="active">Play</button>
    <button id="pause">Pause</button>
  </div>
  <input type="range" id="slider" min="0" max="0" value="0">
</div>
<div class="ctrl-row">
  <div class="ctrl-group">
    <span class="ctrl-label">Speed</span>
    <div class="btn-group">
      <button id="speed1">1x</button>
      <button id="speed2" class="active">4x</button>
      <button id="speed4">16x</button>
    </div>
  </div>
  <div class="ctrl-group">
    <span class="ctrl-label">Mode</span>
    <button id="mode-toggle" class="cycle-btn">Policy</button>
  </div>
  <div class="ctrl-group">
    <span class="ctrl-label">Show</span>
    <button id="player-toggle" class="cycle-btn">Both</button>
  </div>
  <div class="ctrl-group">
    <span class="ctrl-label">&nbsp;</span>
    <button id="rotate-toggle" class="active">Spin</button>
  </div>
</div>
<div class="label">Iterated Policy</div>
<table id="iterated-table">
<thead><tr><th></th><th>R</th><th>P</th><th>S</th></tr></thead>
<tbody></tbody>
</table>
<div class="label">Averaged Strategy</div>
<table id="averaged-table">
<thead><tr><th></th><th>R</th><th>P</th><th>S</th></tr></thead>
<tbody></tbody>
</table>
<div class="sep"></div>
<div class="label p1">P1 Encounters</div>
<table id="p1-table"><thead><tr><th></th><th>R</th><th>P</th><th>S</th></tr></thead><tbody></tbody></table>
<div class="label p2">P2 Encounters</div>
<table id="p2-table"><thead><tr><th></th><th>R</th><th>P</th><th>S</th></tr></thead><tbody></tbody></table>
</div>
<script type="module">
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { CSS2DRenderer, CSS2DObject } from 'three/addons/renderers/CSS2DRenderer.js';

// ═══════════════════ DATA LAYER ═══════════════════

const P1 = __P1_DATA__;
const P2 = __P2_DATA__;
const N = P1.length;
const FADE_WINDOW = 200;
const BG = { r: 0.04, g: 0.04, b: 0.04 };

function computeRegretScale(p1, p2) {
  let mx = 0.001;
  for (const snap of p1) for (const v of snap.regrets) mx = Math.max(mx, Math.abs(v));
  for (const snap of p2) for (const v of snap.regrets) mx = Math.max(mx, Math.abs(v));
  return 1.0 / mx;
}
function policyPos(snap)        { return [snap.iterated[0], snap.iterated[1], snap.iterated[2]]; }
function avgPos(snap)           { return [snap.averaged[0], snap.averaged[1], snap.averaged[2]]; }
function regretPos(snap, scale) { return [snap.regrets[0] * scale, snap.regrets[1] * scale, snap.regrets[2] * scale]; }
function projectToSimplex(regrets) {
  const pos = regrets.map(r => Math.max(0, r));
  const sum = pos[0] + pos[1] + pos[2];
  return sum < 1e-9 ? [1/3, 1/3, 1/3] : [pos[0] / sum, pos[1] / sum, pos[2] / sum];
}
function setVec3(arr, i, xyz) {
  arr[i * 3]     = xyz[0];
  arr[i * 3 + 1] = xyz[1];
  arr[i * 3 + 2] = xyz[2];
}

const REGRET_SCALE = computeRegretScale(P1, P2);

// ═══════════════════ SCENE SETUP ══════════════════

function initRenderers(container) {
  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.setSize(container.clientWidth, container.clientHeight);
  renderer.setClearColor(0x0a0a0a);
  container.appendChild(renderer.domElement);
  const labelRenderer = new CSS2DRenderer();
  labelRenderer.setSize(container.clientWidth, container.clientHeight);
  labelRenderer.domElement.style.position = 'absolute';
  labelRenderer.domElement.style.top = '0';
  labelRenderer.domElement.style.pointerEvents = 'none';
  container.appendChild(labelRenderer.domElement);
  const camera = new THREE.PerspectiveCamera(50, container.clientWidth / container.clientHeight, 0.01, 100);
  camera.position.set(0, 1.5, 2.5);
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.target.set(0, 1 / Math.sqrt(3), 0);
  controls.enableDamping = true;
  controls.dampingFactor = 0.08;
  controls.autoRotate = true;
  controls.autoRotateSpeed = 0.5;
  controls.update();
  window.addEventListener('resize', () => {
    const w = container.clientWidth, h = container.clientHeight;
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    renderer.setSize(w, h);
    labelRenderer.setSize(w, h);
  });
  return { renderer, labelRenderer, camera, controls };
}
function buildSceneGroup(scene) {
  const group = new THREE.Group();
  const from = new THREE.Vector3(1, 1, 1).normalize();
  const to = new THREE.Vector3(0, 1, 0);
  group.quaternion.copy(new THREE.Quaternion().setFromUnitVectors(from, to));
  scene.add(group);
  return group;
}
function buildAxes(group) {
  const len = 1.15;
  const names = ['R', 'P', 'S'];
  const ends = [new THREE.Vector3(len, 0, 0), new THREE.Vector3(0, len, 0), new THREE.Vector3(0, 0, len)];
  for (let i = 0; i < 3; i++) {
    const geo = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(0, 0, 0), ends[i]]);
    group.add(new THREE.Line(geo, new THREE.LineBasicMaterial({ color: 0x555555 })));
    const div = document.createElement('div');
    div.className = 'label-3d';
    div.textContent = names[i];
    const lbl = new CSS2DObject(div);
    lbl.position.copy(ends[i]);
    group.add(lbl);
  }
}
function buildSimplex(group) {
  const wire = new THREE.Line(
    new THREE.BufferGeometry().setFromPoints([
      new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0),
      new THREE.Vector3(0, 0, 1), new THREE.Vector3(1, 0, 0)
    ]),
    new THREE.LineBasicMaterial({ color: 0x333333 })
  );
  group.add(wire);
  const fillGeo = new THREE.BufferGeometry();
  fillGeo.setAttribute('position', new THREE.Float32BufferAttribute([1, 0, 0, 0, 1, 0, 0, 0, 1], 3));
  fillGeo.setIndex([0, 1, 2]);
  const fill = new THREE.Mesh(fillGeo, new THREE.MeshBasicMaterial({
    color: 0x334455, transparent: true, opacity: 0.08, side: THREE.DoubleSide
  }));
  group.add(fill);
  return { wire, fill };
}
function buildNash(group) {
  const pos = new THREE.Vector3(0.4, 0.4, 0.2);
  const sphere = new THREE.Mesh(
    new THREE.SphereGeometry(0.015, 16, 16),
    new THREE.MeshBasicMaterial({ color: 0x44dd88 })
  );
  sphere.position.copy(pos);
  group.add(sphere);
  const ring = new THREE.Mesh(
    new THREE.SphereGeometry(0.025, 16, 16),
    new THREE.MeshBasicMaterial({ color: 0x44dd88, wireframe: true, transparent: true, opacity: 0.4 })
  );
  ring.position.copy(pos);
  group.add(ring);
  return { sphere, ring };
}

// ═══════════════════ TRAIL GEOMETRY ═══════════════

function createTrailPoints(n, color, group) {
  const positions = new Float32Array(n * 3);
  const colors = new Float32Array(n * 3);
  const baseColor = new THREE.Color(color);
  for (let i = 0; i < n; i++) {
    colors[i * 3]     = baseColor.r;
    colors[i * 3 + 1] = baseColor.g;
    colors[i * 3 + 2] = baseColor.b;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  geo.setDrawRange(0, 0);
  const pts = new THREE.Points(geo, new THREE.PointsMaterial({
    size: 0.012, sizeAttenuation: true, vertexColors: true, transparent: true, opacity: 0.8
  }));
  group.add(pts);
  return { geo, positions, colors, pts, baseColor };
}
function createTrailLine(n, color, group) {
  const positions = new Float32Array(n * 3);
  const colors = new Float32Array(n * 3);
  const baseColor = new THREE.Color(color);
  for (let i = 0; i < n; i++) {
    colors[i * 3]     = baseColor.r;
    colors[i * 3 + 1] = baseColor.g;
    colors[i * 3 + 2] = baseColor.b;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  geo.setDrawRange(0, 0);
  const line = new THREE.Line(geo, new THREE.LineBasicMaterial({
    vertexColors: true, transparent: true, opacity: 0.7
  }));
  group.add(line);
  return { geo, positions, colors, line, baseColor };
}
function createProjectionLines(n, group) {
  const p1pos = new Float32Array(n * 2 * 3);
  const p2pos = new Float32Array(n * 2 * 3);
  const p1geo = new THREE.BufferGeometry();
  p1geo.setAttribute('position', new THREE.BufferAttribute(p1pos, 3));
  p1geo.setDrawRange(0, 0);
  const p1lines = new THREE.LineSegments(p1geo, new THREE.LineBasicMaterial({
    color: 0x6699ff, transparent: true, opacity: 0.15
  }));
  group.add(p1lines);
  const p2geo = new THREE.BufferGeometry();
  p2geo.setAttribute('position', new THREE.BufferAttribute(p2pos, 3));
  p2geo.setDrawRange(0, 0);
  const p2lines = new THREE.LineSegments(p2geo, new THREE.LineBasicMaterial({
    color: 0xff6666, transparent: true, opacity: 0.15
  }));
  group.add(p2lines);
  return {
    p1: { geo: p1geo, positions: p1pos, lines: p1lines },
    p2: { geo: p2geo, positions: p2pos, lines: p2lines }
  };
}

// ═══════════════════ MARKERS ═════════════════════

function createMarkers(group) {
  const p1dot  = new THREE.Mesh(new THREE.SphereGeometry(0.02, 16, 16),  new THREE.MeshBasicMaterial({ color: 0x6699ff }));
  const p2dot  = new THREE.Mesh(new THREE.SphereGeometry(0.02, 16, 16),  new THREE.MeshBasicMaterial({ color: 0xff6666 }));
  const p1ring = new THREE.Mesh(new THREE.SphereGeometry(0.025, 16, 16), new THREE.MeshBasicMaterial({ color: 0x6699ff, wireframe: true }));
  const p2ring = new THREE.Mesh(new THREE.SphereGeometry(0.025, 16, 16), new THREE.MeshBasicMaterial({ color: 0xff6666, wireframe: true }));
  group.add(p1dot); group.add(p2dot); group.add(p1ring); group.add(p2ring);
  return { p1dot, p2dot, p1ring, p2ring };
}

// ═══════════════════ POSITION & COLOR ════════════

function fillPositions(mode, trails) {
  const { p1iter, p2iter, p1avg, p2avg, proj } = trails;
  for (let i = 0; i < N; i++) {
    const s1 = P1[i], s2 = P2[i];
    const ipos1 = (mode === 'regret' || mode === 'both') ? regretPos(s1, REGRET_SCALE) : policyPos(s1);
    const ipos2 = (mode === 'regret' || mode === 'both') ? regretPos(s2, REGRET_SCALE) : policyPos(s2);
    setVec3(p1iter.positions, i, ipos1);
    setVec3(p2iter.positions, i, ipos2);
    const apos1 = mode === 'policy' ? avgPos(s1) : regretPos(s1, REGRET_SCALE);
    const apos2 = mode === 'policy' ? avgPos(s2) : regretPos(s2, REGRET_SCALE);
    setVec3(p1avg.positions, i, apos1);
    setVec3(p2avg.positions, i, apos2);
    if (mode === 'both') {
      const rp1 = regretPos(s1, REGRET_SCALE);
      const sp1 = projectToSimplex(s1.regrets);
      setVec3(proj.p1.positions, i * 2, rp1);
      setVec3(proj.p1.positions, i * 2 + 1, sp1);
      const rp2 = regretPos(s2, REGRET_SCALE);
      const sp2 = projectToSimplex(s2.regrets);
      setVec3(proj.p2.positions, i * 2, rp2);
      setVec3(proj.p2.positions, i * 2 + 1, sp2);
    }
  }
  p1iter.geo.attributes.position.needsUpdate = true;
  p2iter.geo.attributes.position.needsUpdate = true;
  p1avg.geo.attributes.position.needsUpdate = true;
  p2avg.geo.attributes.position.needsUpdate = true;
  if (mode === 'both') {
    proj.p1.geo.attributes.position.needsUpdate = true;
    proj.p2.geo.attributes.position.needsUpdate = true;
  }
}
function updateFade(frame, trail) {
  for (let i = 0; i <= frame; i++) {
    const t = Math.max(0, 1 - (frame - i) / FADE_WINDOW);
    trail.colors[i * 3]     = BG.r + (trail.baseColor.r - BG.r) * t;
    trail.colors[i * 3 + 1] = BG.g + (trail.baseColor.g - BG.g) * t;
    trail.colors[i * 3 + 2] = BG.b + (trail.baseColor.b - BG.b) * t;
  }
  trail.geo.attributes.color.needsUpdate = true;
}

// ═══════════════════ PANEL ═══════════════════════

function fProb(v) { return (v >= 0 ? ' ' : '') + v.toFixed(3); }
function fReg(v)  { return (v >= 0 ? '+' : '') + v.toFixed(2); }
function fSci(v)  { return v.toExponential(2).replace('e+', 'e'); }
function fEv(v)   { return (v >= 0 ? '+' : '') + v.toFixed(4); }
function fVis(v)  { return String(v).padStart(7); }
function policyRow(cls, label, arr) {
  return `<tr class="${cls}-row"><th>${label}</th><td>${fProb(arr[0])}</td><td>${fProb(arr[1])}</td><td>${fProb(arr[2])}</td></tr>`;
}
function updatePolicyTable(id, s1, s2, field) {
  document.querySelector(`#${id} tbody`).innerHTML =
    policyRow('p1', 'P1', s1[field]) +
    policyRow('p2', 'P2', s2[field]);
}
function updateTable(id, snap) {
  document.querySelector(`#${id} tbody`).innerHTML =
    `<tr><th>reg</th><td>${fReg(snap.regrets[0])}</td><td>${fReg(snap.regrets[1])}</td><td>${fReg(snap.regrets[2])}</td></tr>` +
    `<tr><th>wgt</th><td>${fSci(snap.weights[0])}</td><td>${fSci(snap.weights[1])}</td><td>${fSci(snap.weights[2])}</td></tr>` +
    `<tr><th>ev</th><td>${fEv(snap.payoffs[0])}</td><td>${fEv(snap.payoffs[1])}</td><td>${fEv(snap.payoffs[2])}</td></tr>` +
    `<tr><th>vis</th><td>${fVis(snap.visits[0])}</td><td>${fVis(snap.visits[1])}</td><td>${fVis(snap.visits[2])}</td></tr>`;
}

// ═══════════════════ FRAME RENDERING ═════════════

function renderFrame(state, trails, markers, simplex, nash) {
  const { frame, mode, showPlayer } = state;
  const s1 = P1[frame], s2 = P2[frame];
  document.getElementById('epoch').textContent = `t = ${s1.epoch}`;
  state.slider.value = frame;
  const { p1iter, p2iter, p1avg, p2avg, proj } = trails;
  p1iter.geo.setDrawRange(0, frame + 1);
  p2iter.geo.setDrawRange(0, frame + 1);
  p1avg.geo.setDrawRange(0, frame + 1);
  p2avg.geo.setDrawRange(0, frame + 1);
  proj.p1.geo.setDrawRange(0, (frame + 1) * 2);
  proj.p2.geo.setDrawRange(0, (frame + 1) * 2);
  // dissolving trail colors
  updateFade(frame, p1iter);
  updateFade(frame, p2iter);
  updateFade(frame, p1avg);
  updateFade(frame, p2avg);
  // current position markers
  const iterFn = (snap) => (mode === 'regret' || mode === 'both') ? regretPos(snap, REGRET_SCALE) : policyPos(snap);
  const [ix1, iy1, iz1] = iterFn(s1);
  const [ix2, iy2, iz2] = iterFn(s2);
  markers.p1dot.position.set(ix1, iy1, iz1);
  markers.p2dot.position.set(ix2, iy2, iz2);
  if (mode === 'both') {
    const sp1 = projectToSimplex(s1.regrets);
    const sp2 = projectToSimplex(s2.regrets);
    markers.p1ring.position.set(sp1[0], sp1[1], sp1[2]);
    markers.p2ring.position.set(sp2[0], sp2[1], sp2[2]);
  } else {
    const avgFn = (snap) => mode === 'policy' ? avgPos(snap) : regretPos(snap, REGRET_SCALE);
    const [ax1, ay1, az1] = avgFn(s1);
    const [ax2, ay2, az2] = avgFn(s2);
    markers.p1ring.position.set(ax1, ay1, az1);
    markers.p2ring.position.set(ax2, ay2, az2);
  }
  // visibility: player filter
  const show1 = showPlayer !== 'p2';
  const show2 = showPlayer !== 'p1';
  p1iter.pts.visible       = show1;
  p2iter.pts.visible       = show2;
  markers.p1dot.visible    = show1;
  markers.p2dot.visible    = show2;
  p1avg.line.visible       = show1;
  p2avg.line.visible       = show2;
  markers.p1ring.visible   = show1;
  markers.p2ring.visible   = show2;
  // projection lines: only in both mode
  proj.p1.lines.visible    = show1 && mode === 'both';
  proj.p2.lines.visible    = show2 && mode === 'both';
  // simplex + nash: visible in policy and both, hidden in regret
  const showSimplex        = mode !== 'regret';
  simplex.wire.visible     = showSimplex;
  simplex.fill.visible     = showSimplex;
  nash.sphere.visible      = showSimplex;
  nash.ring.visible        = showSimplex;
  // data panel
  updatePolicyTable('iterated-table', s1, s2, 'iterated');
  updatePolicyTable('averaged-table', s1, s2, 'averaged');
  updateTable('p1-table', s1);
  updateTable('p2-table', s2);
}

// ═══════════════════ CONTROLS ════════════════════

function updateSpeedBtns(speed) {
  for (const id of ['speed1', 'speed2', 'speed4']) document.getElementById(id).classList.remove('active');
  if (speed === 1)  document.getElementById('speed1').classList.add('active');
  if (speed === 4)  document.getElementById('speed2').classList.add('active');
  if (speed >= 16)  document.getElementById('speed4').classList.add('active');
}
function setupControls(state, trails, markers, simplex, nash, controls) {
  const render = () => renderFrame(state, trails, markers, simplex, nash);
  document.getElementById('play').onclick = () => {
    state.playing = true;
    document.getElementById('play').classList.add('active');
    document.getElementById('pause').classList.remove('active');
  };
  document.getElementById('pause').onclick = () => {
    state.playing = false;
    document.getElementById('pause').classList.add('active');
    document.getElementById('play').classList.remove('active');
  };
  state.slider.oninput = () => { state.frame = parseInt(state.slider.value); render(); };
  document.getElementById('speed1').onclick = () => { state.speed = 1;  updateSpeedBtns(1); };
  document.getElementById('speed2').onclick = () => { state.speed = 4;  updateSpeedBtns(4); };
  document.getElementById('speed4').onclick = () => { state.speed = 16; updateSpeedBtns(16); };
  document.getElementById('player-toggle').onclick = () => {
    const btn = document.getElementById('player-toggle');
    if (state.showPlayer === 'both')     { state.showPlayer = 'p1';   btn.textContent = 'P1'; }
    else if (state.showPlayer === 'p1')  { state.showPlayer = 'p2';   btn.textContent = 'P2'; }
    else                                 { state.showPlayer = 'both'; btn.textContent = 'Both'; }
    render();
  };
  document.getElementById('mode-toggle').onclick = () => {
    const btn = document.getElementById('mode-toggle');
    if (state.mode === 'policy')      { state.mode = 'regret'; btn.textContent = 'Regret'; }
    else if (state.mode === 'regret') { state.mode = 'both';   btn.textContent = 'Both'; }
    else                              { state.mode = 'policy'; btn.textContent = 'Policy'; }
    fillPositions(state.mode, trails);
    render();
  };
  document.getElementById('rotate-toggle').onclick = () => {
    controls.autoRotate = !controls.autoRotate;
    document.getElementById('rotate-toggle').classList.toggle('active');
  };
}

// ═══════════════════ ANIMATION ═══════════════════

function animate(state, renderer, labelRenderer, camera, scene, controls, trails, markers, simplex, nash) {
  let tick = 0;
  function loop() {
    requestAnimationFrame(loop);
    controls.update();
    if (state.playing && state.frame < N - 1) {
      tick++;
      if (tick % Math.max(1, Math.round(4 / state.speed)) === 0) {
        state.frame++;
        renderFrame(state, trails, markers, simplex, nash);
      }
    }
    renderer.render(scene, camera);
    labelRenderer.render(scene, camera);
  }
  loop();
}

// ═══════════════════ MAIN ════════════════════════

(function main() {
  const container = document.getElementById('canvas');
  const scene = new THREE.Scene();
  const { renderer, labelRenderer, camera, controls } = initRenderers(container);
  const group   = buildSceneGroup(scene);
  buildAxes(group);
  const simplex = buildSimplex(group);
  const nash    = buildNash(group);
  const p1iter  = createTrailPoints(N, 0x6699ff, group);
  const p2iter  = createTrailPoints(N, 0xff6666, group);
  const p1avg   = createTrailLine(N, 0x6699ff, group);
  const p2avg   = createTrailLine(N, 0xff6666, group);
  const proj    = createProjectionLines(N, group);
  const trails  = { p1iter, p2iter, p1avg, p2avg, proj };
  const markers = createMarkers(group);
  const slider  = document.getElementById('slider');
  slider.max = N - 1;
  const state = { frame: 0, playing: true, speed: 4, showPlayer: 'both', mode: 'policy', slider };
  fillPositions(state.mode, trails);
  setupControls(state, trails, markers, simplex, nash, controls);
  renderFrame(state, trails, markers, simplex, nash);
  animate(state, renderer, labelRenderer, camera, scene, controls, trails, markers, simplex, nash);
})();
</script>
</body>
</html>
"#;

/// Dual-panel viewer: 2D policy simplex + 3D regret vectors, synchronized.
pub const SIMPLEX_DUAL: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>RPS Simplex: Policy + Regret</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: #0a0a0a; color: #e0e0e0; font-family: 'SF Mono', 'Fira Code', monospace; overflow: hidden; }
#viewport { width: 100vw; height: 100vh; position: relative; }
#data-overlay {
  position: absolute; bottom: 100px; right: 16px;
  background: rgba(10,10,10,0.85); border: 1px solid #333;
  border-radius: 6px; padding: 8px 12px; font-size: 11px; z-index: 5;
  display: none; opacity: 0;
}
#data-overlay table { border-collapse: collapse; }
#data-overlay td, #data-overlay th { padding: 1px 0; font-variant-numeric: tabular-nums; white-space: pre; font-size: 11px; }
#data-overlay th { color: #555; font-weight: normal; }
.p1-val { color: #8ab4ff; }
.p2-val { color: #ff9999; }
.p1-hdr { color: #6699ff; font-weight: bold; }
.p2-hdr { color: #ff6666; font-weight: bold; }
.vertex-label {
  font-family: 'SF Mono', 'Fira Code', monospace; font-size: 10px;
  pointer-events: none; user-select: none; white-space: pre;
  background: rgba(10,10,10,0.75); border-radius: 3px; padding: 3px 5px;
  line-height: 1.4;
}
#controls {
  position: absolute; bottom: 16px; left: 50%; transform: translateX(-50%);
  background: rgba(10,10,10,0.85); border: 1px solid #333;
  border-radius: 8px; padding: 10px 16px;
  display: flex; flex-direction: column; gap: 8px; z-index: 10;
}
.ctrl-row { display: flex; gap: 8px; align-items: center; }
#epoch { font-size: 16px; font-weight: bold; letter-spacing: -0.5px; min-width: 90px; }
.ctrl-row input[type=range] { width: 200px; accent-color: #667; }
button {
  background: #1a1a2e; border: 1px solid #333; color: #ccc;
  padding: 4px 10px; cursor: pointer; font-family: inherit;
  font-size: 11px; border-radius: 3px; white-space: nowrap;
}
button:hover { background: #2a2a4e; }
button.active { background: #3a3a6e; border-color: #667; }
.btn-group { display: flex; }
.btn-group button { border-radius: 0; border-right-width: 0; }
.btn-group button:first-child { border-radius: 3px 0 0 3px; }
.btn-group button:last-child { border-radius: 0 3px 3px 0; border-right-width: 1px; }
</style>
<script type="importmap">
{ "imports": {
    "three": "https://cdn.jsdelivr.net/npm/three@0.170.0/build/three.module.js",
    "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.170.0/examples/jsm/"
} }
</script>
</head>
<body>
<div id="viewport">
  <div id="data-overlay"><table id="overlay-table"></table></div>
  <div id="controls">
    <div class="ctrl-row">
      <span id="epoch">t = 0</span>
      <button id="restart">&#x23EE;</button>
      <button id="playpause">&#x23F8;</button>
      <input type="range" id="slider" min="0" max="0" value="0">
    </div>
    <div class="ctrl-row">
      <div class="btn-group" id="speed-group">
        <button data-speed="1">&times;1</button>
        <button data-speed="4" class="active">&times;4</button>
        <button data-speed="16">&times;16</button>
      </div>
      <div class="btn-group" id="view-group">
        <button data-view="policy" class="active">Policy</button>
        <button data-view="regret">Regret</button>
      </div>
      <button id="player-toggle">Both</button>
      <button id="spin-toggle" class="active">Spin</button>
    </div>
  </div>
</div>
<script type="module">
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { CSS2DRenderer, CSS2DObject } from 'three/addons/renderers/CSS2DRenderer.js';

// ═══════════════════ DATA ═══════════════════════
const P1 = __P1_DATA__;
const P2 = __P2_DATA__;
const N = P1.length;
const FADE_WINDOW = 200;
const BG = { r: 0.04, g: 0.04, b: 0.04 };

// ═══════════════════ FORMATTERS ═════════════════
const PW = 6;
const SW = 8;
const EW = 7;
const G = '  ';
function fPol(v)  { return ((v * 100).toFixed(1) + '%').padStart(PW); }
function fSgn(v)  { return ((v >= 0 ? '+' : '-') + Math.abs(v).toExponential(1)).padStart(SW); }
function fEval(v) { return ((v >= 0 ? '+' : '-') + Math.abs(v).toFixed(3)).padStart(EW); }

// ═══════════════════ STATE ═════════════════════
const state = { frame: 0, playing: true, speed: 4, mode: 'policy', showPlayer: 'both', spin: true };
const sliderEl = document.getElementById('slider');
sliderEl.max = N - 1;

// ═══════════════════ CAMERA PRESETS ════════════
const POLICY_CAM    = new THREE.Vector3(0, 3.5, 0.01);
const POLICY_TARGET = new THREE.Vector3(0, 0.577, 0);
const REGRET_CAM    = new THREE.Vector3(0, 1.5, 2.5);
const REGRET_TARGET = new THREE.Vector3(0, 0.3, 0);

// ═══════════════════ SCENE SETUP ═══════════════
const vp = document.getElementById('viewport');
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setPixelRatio(window.devicePixelRatio);
renderer.setSize(vp.clientWidth, vp.clientHeight);
renderer.setClearColor(0x0a0a0a);
vp.insertBefore(renderer.domElement, vp.firstChild);
const labelRenderer = new CSS2DRenderer();
labelRenderer.setSize(vp.clientWidth, vp.clientHeight);
labelRenderer.domElement.style.cssText = 'position:absolute;top:0;left:0;pointer-events:none;';
vp.insertBefore(labelRenderer.domElement, vp.children[1]);
const camera = new THREE.PerspectiveCamera(50, vp.clientWidth / vp.clientHeight, 0.01, 100);
camera.position.copy(POLICY_CAM);
const controls = new OrbitControls(camera, renderer.domElement);
controls.target.copy(POLICY_TARGET);
controls.enableDamping = true;
controls.dampingFactor = 0.08;
controls.autoRotate = false;
controls.autoRotateSpeed = 0.5;
controls.minPolarAngle = 0.3;
controls.maxPolarAngle = Math.PI * 0.45;
controls.enabled = false;
controls.update();
const scene = new THREE.Scene();

// ═══════════════════ GROUP ROTATION ════════════
const group = new THREE.Group();
const baseQ = new THREE.Quaternion().setFromUnitVectors(
  new THREE.Vector3(1, 1, 1).normalize(), new THREE.Vector3(0, 1, 0));
const sAfterQ = new THREE.Vector3(0, 0, 1).applyQuaternion(baseQ);
const spinQ = new THREE.Quaternion().setFromAxisAngle(
  new THREE.Vector3(0, 1, 0), Math.atan2(sAfterQ.x, -sAfterQ.z));
group.quaternion.copy(spinQ.multiply(baseQ));
scene.add(group);
const simplexVerts = [new THREE.Vector3(1,0,0), new THREE.Vector3(0,1,0), new THREE.Vector3(0,0,1)];

// ═══════════════════ SIMPLEX WIRE + FILL ═══════
const simplexWire = new THREE.Line(
  new THREE.BufferGeometry().setFromPoints([...simplexVerts, simplexVerts[0]]),
  new THREE.LineBasicMaterial({ color: 0x444444, transparent: true, opacity: 1 }));
group.add(simplexWire);
const fillGeo = new THREE.BufferGeometry();
fillGeo.setAttribute('position', new THREE.Float32BufferAttribute([1,0,0, 0,1,0, 0,0,1], 3));
fillGeo.setIndex([0, 1, 2]);
const simplexFill = new THREE.Mesh(fillGeo, new THREE.MeshBasicMaterial({
  color: 0x334455, transparent: true, opacity: 0.08, side: THREE.DoubleSide, depthWrite: false }));
group.add(simplexFill);

// ═══════════════════ AXES (positive only) ═════
const axisLen = 1.15;
const axisNames = ['R', 'P', 'S'];
const axisEnds = [new THREE.Vector3(axisLen,0,0), new THREE.Vector3(0,axisLen,0), new THREE.Vector3(0,0,axisLen)];
const axisLines = [], axisLabelObjs = [];
for (let i = 0; i < 3; i++) {
  const line = new THREE.Line(
    new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(0,0,0), axisEnds[i]]),
    new THREE.LineBasicMaterial({ color: 0x555555, transparent: true, opacity: 0 }));
  line.visible = false;
  group.add(line);
  axisLines.push(line);
  const div = document.createElement('div');
  div.style.cssText = 'color:#888;font:14px SF Mono,Fira Code,monospace;pointer-events:none;user-select:none;';
  div.textContent = axisNames[i];
  const lbl = new CSS2DObject(div);
  lbl.position.copy(axisEnds[i]);
  lbl.visible = false;
  group.add(lbl);
  axisLabelObjs.push(lbl);
}

// ═══════════════════ ZERO-PLANES ═══════════════
const L = 2.0, zeroPlanes = [];
for (const v of [[0,0,0,0,L,0,0,0,L],[0,0,0,L,0,0,0,0,L],[0,0,0,L,0,0,0,L,0]]) {
  const g = new THREE.BufferGeometry();
  g.setAttribute('position', new THREE.Float32BufferAttribute(v, 3));
  g.setIndex([0, 1, 2]);
  const m = new THREE.Mesh(g, new THREE.MeshBasicMaterial({
    color: 0x335577, transparent: true, opacity: 0, side: THREE.DoubleSide, depthWrite: false }));
  m.visible = false;
  group.add(m);
  zeroPlanes.push(m);
}

// ═══════════════════ ORIGIN MARKER ═════════════
const originSphere = new THREE.Mesh(
  new THREE.SphereGeometry(0.012, 12, 12), new THREE.MeshBasicMaterial({ color: 0x444444 }));
originSphere.visible = false;
group.add(originSphere);

// ═══════════════════ TRAIL POINT CLOUDS ════════
function createTrail(color) {
  const positions = new Float32Array(N * 3);
  const colors = new Float32Array(N * 3);
  const baseColor = new THREE.Color(color);
  for (let i = 0; i < N; i++) {
    colors[i*3] = baseColor.r; colors[i*3+1] = baseColor.g; colors[i*3+2] = baseColor.b;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  geo.setDrawRange(0, 0);
  const pts = new THREE.Points(geo, new THREE.PointsMaterial({
    size: 0.012, sizeAttenuation: true, vertexColors: true, transparent: true, opacity: 0.8 }));
  pts.renderOrder = 1;
  group.add(pts);
  return { geo, positions, colors, pts, baseColor };
}
function computeRegretScale() {
  let mx = 0.001;
  for (const s of P1) for (const v of s.regrets) mx = Math.max(mx, Math.abs(v));
  for (const s of P2) for (const v of s.regrets) mx = Math.max(mx, Math.abs(v));
  return 1.0 / mx;
}
const RS = computeRegretScale();
const p1PolTrail = createTrail(0x6699ff), p2PolTrail = createTrail(0xff6666);
const p1RegTrail = createTrail(0x6699ff), p2RegTrail = createTrail(0xff6666);
for (let i = 0; i < N; i++) {
  const s1 = P1[i], s2 = P2[i], j = i * 3;
  p1PolTrail.positions[j]=s1.iterated[0]; p1PolTrail.positions[j+1]=s1.iterated[1]; p1PolTrail.positions[j+2]=s1.iterated[2];
  p2PolTrail.positions[j]=s2.iterated[0]; p2PolTrail.positions[j+1]=s2.iterated[1]; p2PolTrail.positions[j+2]=s2.iterated[2];
  p1RegTrail.positions[j]=s1.regrets[0]*RS; p1RegTrail.positions[j+1]=s1.regrets[1]*RS; p1RegTrail.positions[j+2]=s1.regrets[2]*RS;
  p2RegTrail.positions[j]=s2.regrets[0]*RS; p2RegTrail.positions[j+1]=s2.regrets[1]*RS; p2RegTrail.positions[j+2]=s2.regrets[2]*RS;
}
for (const t of [p1PolTrail, p2PolTrail, p1RegTrail, p2RegTrail]) t.geo.attributes.position.needsUpdate = true;
p1RegTrail.pts.visible = false;
p2RegTrail.pts.visible = false;

// ═══════════════════ CURRENT DOTS ══════════════
function makeDot(color) {
  const m = new THREE.Mesh(new THREE.SphereGeometry(0.02, 16, 16), new THREE.MeshBasicMaterial({ color }));
  m.renderOrder = 10;
  group.add(m);
  return m;
}
const p1PolDot = makeDot(0x6699ff), p2PolDot = makeDot(0xff6666);
const p1RegDot = makeDot(0x6699ff), p2RegDot = makeDot(0xff6666);
p1RegDot.visible = false; p2RegDot.visible = false;

// ═══════════════════ AVERAGED CROSSES ══════════
function makeCross(color) {
  const hw = 0.025, ht = 0.004, s = new THREE.Shape();
  s.moveTo(-hw,-ht); s.lineTo(-hw,ht); s.lineTo(-ht,ht);
  s.lineTo(-ht,hw); s.lineTo(ht,hw); s.lineTo(ht,ht);
  s.lineTo(hw,ht); s.lineTo(hw,-ht); s.lineTo(ht,-ht);
  s.lineTo(ht,-hw); s.lineTo(-ht,-hw); s.lineTo(-ht,-ht);
  s.closePath();
  const mesh = new THREE.Mesh(new THREE.ShapeGeometry(s), new THREE.MeshBasicMaterial({
    color, transparent: true, opacity: 1, side: THREE.DoubleSide, depthTest: false }));
  mesh.renderOrder = 5;
  group.add(mesh);
  return mesh;
}
const p1cross = makeCross(0x6699ff), p2cross = makeCross(0xff6666);

// ═══════════════════ VERTEX LABELS ═════════════
const vertexLabels = [];
const labelDefs = [
  { name: 'R', idx: 0, pos: simplexVerts[0], transform: 'translate(-50%, 12px)' },
  { name: 'P', idx: 1, pos: simplexVerts[1], transform: 'translate(-50%, 12px)' },
  { name: 'S', idx: 2, pos: simplexVerts[2], transform: 'translate(-50%, calc(-100% - 12px))' },
];
for (const ld of labelDefs) {
  const wrapper = document.createElement('div');
  wrapper.style.cssText = 'width:0;height:0;overflow:visible;';
  const inner = document.createElement('div');
  inner.className = 'vertex-label';
  inner.style.cssText = `position:absolute;left:0;width:max-content;transform:${ld.transform};`;
  wrapper.appendChild(inner);
  const obj = new CSS2DObject(wrapper);
  obj.position.copy(ld.pos);
  group.add(obj);
  vertexLabels.push({ name: ld.name, idx: ld.idx, wrapper, div: inner });
}

// ═══════════════════ FADE COLORS ═══════════════
function updateFade(frame, trail) {
  for (let i = 0; i <= frame; i++) {
    const t = Math.max(0, 1 - (frame - i) / FADE_WINDOW);
    trail.colors[i*3]   = BG.r + (trail.baseColor.r - BG.r) * t;
    trail.colors[i*3+1] = BG.g + (trail.baseColor.g - BG.g) * t;
    trail.colors[i*3+2] = BG.b + (trail.baseColor.b - BG.b) * t;
  }
  trail.geo.attributes.color.needsUpdate = true;
}

// ═══════════════════ MORPH / CROSSFADE ═════════
let morphing = false, morphStart = 0, morphToMode = 'policy';
const MORPH_MS = 600;
const morphFrom = { pos: null, tgt: null }, morphTo = { pos: null, tgt: null };
function ease(t) { return t < 0.5 ? 4*t*t*t : 1 - Math.pow(-2*t+2, 3) / 2; }

function startMorph(toMode) {
  if (morphing || state.mode === toMode) return;
  morphing = true;
  morphStart = performance.now();
  morphToMode = toMode;
  morphFrom.pos = camera.position.clone();
  morphFrom.tgt = controls.target.clone();
  morphTo.pos = (toMode === 'policy' ? POLICY_CAM : REGRET_CAM).clone();
  morphTo.tgt = (toMode === 'policy' ? POLICY_TARGET : REGRET_TARGET).clone();
  controls.enabled = false;
  // show everything for crossfade
  const s1 = state.showPlayer !== 'p2', s2 = state.showPlayer !== 'p1';
  simplexWire.visible = true; simplexFill.visible = true;
  p1PolTrail.pts.visible = s1; p2PolTrail.pts.visible = s2;
  p1RegTrail.pts.visible = s1; p2RegTrail.pts.visible = s2;
  p1PolDot.visible = s1; p2PolDot.visible = s2;
  p1RegDot.visible = s1; p2RegDot.visible = s2;
  p1cross.visible = s1; p2cross.visible = s2;
  for (const a of axisLines) a.visible = true;
  for (const l of axisLabelObjs) l.visible = true;
  for (const p of zeroPlanes) p.visible = true;
  for (const vl of vertexLabels) vl.wrapper.style.display = '';
  document.getElementById('data-overlay').style.display = 'block';
  originSphere.visible = true;
}

function updateMorph(now) {
  if (!morphing) return;
  const raw = Math.min(1, (now - morphStart) / MORPH_MS);
  const t = ease(raw);
  const toP = morphToMode === 'policy';
  camera.position.lerpVectors(morphFrom.pos, morphTo.pos, t);
  controls.target.lerpVectors(morphFrom.tgt, morphTo.tgt, t);
  const pOp = toP ? t : 1 - t;
  const rOp = toP ? 1 - t : t;
  simplexWire.material.opacity = pOp;
  simplexFill.material.opacity = pOp * 0.08;
  p1PolTrail.pts.material.opacity = pOp * 0.8;
  p2PolTrail.pts.material.opacity = pOp * 0.8;
  p1cross.material.opacity = pOp;
  p2cross.material.opacity = pOp;
  for (const vl of vertexLabels) vl.div.style.opacity = pOp;
  for (const a of axisLines) a.material.opacity = rOp * 0.6;
  for (const p of zeroPlanes) p.material.opacity = rOp * 0.05;
  p1RegTrail.pts.material.opacity = rOp * 0.8;
  p2RegTrail.pts.material.opacity = rOp * 0.8;
  document.getElementById('data-overlay').style.opacity = rOp;
  if (raw >= 1) {
    morphing = false;
    state.mode = morphToMode;
    controls.enabled = !toP;
    updateVisibility();
  }
}

// ═══════════════════ VISIBILITY ═════════════════
function updateVisibility() {
  const pol = state.mode === 'policy';
  const s1 = state.showPlayer !== 'p2', s2 = state.showPlayer !== 'p1';
  simplexWire.visible = pol; simplexWire.material.opacity = pol ? 1 : 0;
  simplexFill.visible = pol; simplexFill.material.opacity = pol ? 0.08 : 0;
  p1PolTrail.pts.visible = pol && s1; p2PolTrail.pts.visible = pol && s2;
  p1PolTrail.pts.material.opacity = 0.8; p2PolTrail.pts.material.opacity = 0.8;
  p1PolDot.visible = pol && s1; p2PolDot.visible = pol && s2;
  p1cross.visible = pol && s1; p2cross.visible = pol && s2;
  p1cross.material.opacity = 1; p2cross.material.opacity = 1;
  for (const vl of vertexLabels) { vl.wrapper.style.display = pol ? '' : 'none'; vl.div.style.opacity = pol ? 1 : 0; }
  for (const a of axisLines) { a.visible = !pol; a.material.opacity = !pol ? 0.6 : 0; }
  for (const l of axisLabelObjs) l.visible = !pol;
  for (const p of zeroPlanes) { p.visible = !pol; p.material.opacity = !pol ? 0.05 : 0; }
  p1RegTrail.pts.visible = !pol && s1; p2RegTrail.pts.visible = !pol && s2;
  p1RegTrail.pts.material.opacity = 0.8; p2RegTrail.pts.material.opacity = 0.8;
  p1RegDot.visible = !pol && s1; p2RegDot.visible = !pol && s2;
  originSphere.visible = !pol;
  controls.autoRotate = !pol && state.spin;
  const ov = document.getElementById('data-overlay');
  ov.style.display = !pol ? 'block' : 'none'; ov.style.opacity = !pol ? 1 : 0;
}

// ═══════════════════ VERTEX LABEL UPDATE ═══════
function updateVertexLabels() {
  const s1 = P1[state.frame], s2 = P2[state.frame];
  const sh1 = state.showPlayer !== 'p2', sh2 = state.showPlayer !== 'p1';
  function row(cls, label, snap, idx) {
    const c = s => `<span class="${cls}-val">${s}</span>`;
    return `<span class="${cls}-hdr">${label}</span>${c(fPol(snap.iterated[idx]))}${G}${c(fSgn(snap.regrets[idx]))}${G}${c(fSgn(snap.weights[idx]))}${G}${c(fEval(snap.payoffs[idx]))}\n`;
  }
  for (const vl of vertexLabels) {
    const i = vl.idx;
    let h = `<span style="color:#888;font-size:11px;font-weight:bold">${vl.name}</span>\n`;
    h += `<span style="color:#555">   ${'pol'.padStart(PW)}${G}${'reg'.padStart(SW)}${G}${'wgt'.padStart(SW)}${G}${'ev'.padStart(EW)}</span>\n`;
    if (sh1) h += row('p1', 'P1 ', s1, i);
    if (sh2) h += row('p2', 'P2 ', s2, i);
    vl.div.innerHTML = h.trimEnd();
  }
}

// ═══════════════════ DATA OVERLAY ═══════════════
function updateOverlay() {
  const s1 = P1[state.frame], s2 = P2[state.frame];
  const sh1 = state.showPlayer !== 'p2', sh2 = state.showPlayer !== 'p1';
  let h = `<tr><th>  </th><th style="color:#666">${G}${'pol'.padStart(PW)}</th><th style="color:#666">${G}${'reg'.padStart(SW)}</th><th style="color:#666">${G}${'wgt'.padStart(SW)}</th><th style="color:#666">${G}${'ev'.padStart(EW)}</th></tr>`;
  function aRow(cls, label, snap, i) {
    return `<tr><th class="${cls}-hdr">${label}</th><td class="${cls}-val">${G}${fPol(snap.iterated[i])}</td><td class="${cls}-val">${G}${fSgn(snap.regrets[i])}</td><td class="${cls}-val">${G}${fSgn(snap.weights[i])}</td><td class="${cls}-val">${G}${fEval(snap.payoffs[i])}</td></tr>`;
  }
  if (sh1) { h += aRow('p1','R ',s1,0); h += aRow('p1','P ',s1,1); h += aRow('p1','S ',s1,2); }
  if (sh1 && sh2) h += '<tr><td colspan="5" style="border-top:1px solid #222;padding:2px"></td></tr>';
  if (sh2) { h += aRow('p2','R ',s2,0); h += aRow('p2','P ',s2,1); h += aRow('p2','S ',s2,2); }
  document.getElementById('overlay-table').innerHTML = h;
}

// ═══════════════════ RENDER FRAME ═══════════════
function renderFrame() {
  const f = state.frame, s1 = P1[f], s2 = P2[f];
  p1PolDot.position.set(s1.iterated[0], s1.iterated[1], s1.iterated[2]);
  p2PolDot.position.set(s2.iterated[0], s2.iterated[1], s2.iterated[2]);
  p1RegDot.position.set(s1.regrets[0]*RS, s1.regrets[1]*RS, s1.regrets[2]*RS);
  p2RegDot.position.set(s2.regrets[0]*RS, s2.regrets[1]*RS, s2.regrets[2]*RS);
  p1cross.position.set(s1.averaged[0], s1.averaged[1], s1.averaged[2]);
  p2cross.position.set(s2.averaged[0], s2.averaged[1], s2.averaged[2]);
  p1cross.lookAt(camera.position);
  p2cross.lookAt(camera.position);
  for (const t of [p1PolTrail, p2PolTrail, p1RegTrail, p2RegTrail]) {
    const start = Math.max(0, f - FADE_WINDOW);
    t.geo.setDrawRange(start, f - start + 1);
  }
  updateFade(f, p1PolTrail); updateFade(f, p2PolTrail);
  updateFade(f, p1RegTrail); updateFade(f, p2RegTrail);
  updateVertexLabels();
  updateOverlay();
  document.getElementById('epoch').textContent = `t = ${s1.epoch}`;
  sliderEl.value = f;
}

// ═══════════════════ CONTROLS ═══════════════════
document.getElementById('playpause').onclick = () => {
  state.playing = !state.playing;
  document.getElementById('playpause').innerHTML = state.playing ? '&#x23F8;' : '&#x25B6;';
};
document.getElementById('restart').onclick = () => { state.frame = 0; renderFrame(); };
sliderEl.oninput = () => { state.frame = parseInt(sliderEl.value); renderFrame(); };
for (const b of document.querySelectorAll('#speed-group button'))
  b.onclick = () => {
    state.speed = parseInt(b.dataset.speed);
    for (const x of document.querySelectorAll('#speed-group button')) x.classList.remove('active');
    b.classList.add('active');
  };
for (const b of document.querySelectorAll('#view-group button'))
  b.onclick = () => {
    startMorph(b.dataset.view);
    for (const x of document.querySelectorAll('#view-group button')) x.classList.remove('active');
    b.classList.add('active');
  };
document.getElementById('player-toggle').onclick = () => {
  const b = document.getElementById('player-toggle');
  if (state.showPlayer === 'both') { state.showPlayer = 'p1'; b.textContent = 'P1'; }
  else if (state.showPlayer === 'p1') { state.showPlayer = 'p2'; b.textContent = 'P2'; }
  else { state.showPlayer = 'both'; b.textContent = 'Both'; }
  if (!morphing) updateVisibility();
  renderFrame();
};
document.getElementById('spin-toggle').onclick = () => {
  state.spin = !state.spin;
  controls.autoRotate = state.spin && state.mode !== 'policy';
  document.getElementById('spin-toggle').classList.toggle('active');
};

// ═══════════════════ RESIZE ═════════════════════
window.addEventListener('resize', () => {
  const w = vp.clientWidth, h = vp.clientHeight;
  camera.aspect = w / h; camera.updateProjectionMatrix();
  renderer.setSize(w, h); labelRenderer.setSize(w, h);
});

// ═══════════════════ ANIMATION LOOP ════════════
renderFrame();
updateVisibility();
let tick = 0;
(function loop() {
  requestAnimationFrame(loop);
  updateMorph(performance.now());
  controls.update();
  if (state.playing && state.frame < N - 1 && ++tick % 4 === 0) {
    state.frame = Math.min(N - 1, state.frame + state.speed);
    renderFrame();
  }
  renderer.render(scene, camera);
  labelRenderer.render(scene, camera);
})();
</script>
</body>
</html>
"#;
