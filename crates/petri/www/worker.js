// Owns one petri Session. DeepHyperParams is set once per wasm instance, so the page starts a
// fresh worker for every run instead of reusing this one.
import init, { Session } from "./pkg/petri.js";

const ready = init();
const run = { session: null, playing: false, looping: false, budget: 0, every: 1, pace: "watch", epoch: 0, cost: 0 };

self.onmessage = async ({ data }) => {
    await ready;
    if (data.type === "start") start(data);
    if (data.type === "play") play();
    if (data.type === "pause") run.playing = false;
    if (data.type === "step" && run.session && run.epoch < run.budget) tick(1);
    if (data.type === "pace") run.pace = data.pace;
};

function start({ game, net, knobs, budget, every, pace, play: autoplay }) {
    try {
        run.session = new Session(game, net, JSON.stringify(knobs));
    } catch (e) {
        self.postMessage({ type: "error", message: String(e?.message ?? e) });
        return;
    }
    Object.assign(run, { budget, every, pace, epoch: 0, cost: 0 });
    checkpoint();
    self.postMessage({ type: "ready", play: autoplay });
    if (autoplay) play();
}

function play() {
    run.playing = true;
    loop();
}

async function loop() {
    if (run.looping) return;
    run.looping = true;
    while (run.playing && run.epoch < run.budget) {
        tick(run.pace === "max" ? Math.max(1, Math.floor(40 / Math.max(run.cost, 0.01))) : 1);
        await new Promise((resolve) => setTimeout(resolve, run.pace === "max" ? 0 : 25));
    }
    run.looping = false;
}

function tick(limit) {
    const k = Math.min(limit, run.every - (run.epoch % run.every), run.budget - run.epoch);
    const t0 = performance.now();
    const frame = run.session.step(k);
    const ms = performance.now() - t0;
    run.epoch += k;
    run.cost = ms / k;
    self.postMessage({ type: "frame", frame, epochs: k, ms });
    if (run.epoch % run.every === 0 || run.epoch === run.budget) checkpoint();
    if (run.epoch === run.budget) {
        run.playing = false;
        self.postMessage({ type: "done" });
    }
}

function checkpoint() {
    const t0 = performance.now();
    const frame = run.session.checkpoint();
    self.postMessage({ type: "checkpoint", frame, ms: performance.now() - t0 });
}
