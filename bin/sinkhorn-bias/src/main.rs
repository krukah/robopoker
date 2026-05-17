//! Sinkhorn entropic-bias diagnostic.
//!
//! Loads real Flop centroids (Histogram over Turn abs) and the Turn ground
//! metric from the DB. For each centroid μ, measures `OT_ε(μ, μ)` (which
//! should be 0 for true EMD but is biased upward by entropic regularization).
//! Compares against `OT_ε(μ, ν)` on random off-diagonal pairs.
//!
//! Run with `DB_URL` set to a DB containing populated `metric` + `transitions`
//! tables. Reports bias ratio + per-call timing so we can decide whether the
//! Sinkhorn divergence debias is worth implementing.

use rbp_cards::*;
use rbp_clustering::*;
use rbp_core::*;
use rbp_gameplay::*;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    let _telemetry = rbp_telemetry::init();
    let client = rbp_database::db().await;
    println!("connected to db");
    println!("loading Turn metric (ground distances over Turn abs)...");
    let t0 = Instant::now();
    let metric = Metric::from_street(&client, Street::Turn).await;
    println!("loaded metric in {:.2?}", t0.elapsed());
    println!("loading Flop centroids (histograms over Turn abs)...");
    let t0 = Instant::now();
    let centroids = load_centroids(&client, Street::Flop).await;
    println!(
        "loaded {} centroids in {:.2?}",
        centroids.len(),
        t0.elapsed()
    );
    if centroids.is_empty() {
        eprintln!("no centroids found — is the transitions table populated?");
        std::process::exit(1);
    }
    let n_self = centroids.len();
    let n_cross = 256;
    let mut self_costs: Vec<Energy> = Vec::with_capacity(n_self);
    let mut self_times: Vec<Duration> = Vec::with_capacity(n_self);
    let mut cross_costs: Vec<Energy> = Vec::with_capacity(n_cross);
    let mut cross_times: Vec<Duration> = Vec::with_capacity(n_cross);
    println!(
        "\nrunning self-cost OT_eps(mu, mu) on {} centroids...",
        n_self
    );
    for (k, (_abs, hist)) in centroids.iter().enumerate() {
        let t0 = Instant::now();
        let cost = metric.emd(hist, hist);
        let dt = t0.elapsed();
        self_costs.push(cost);
        self_times.push(dt);
        if k < 5 {
            println!(
                "  centroid #{}: self-cost = {:.6}, support = {}",
                k,
                cost,
                hist.n()
            );
        }
    }
    println!(
        "\nrunning cross-cost OT_eps(mu, nu) on {} random pairs...",
        n_cross
    );
    let mut sampled = 0usize;
    while sampled < n_cross {
        let i = rand::random_range(0..centroids.len());
        let j = rand::random_range(0..centroids.len());
        if i == j {
            continue;
        }
        let (_, hi) = &centroids[i];
        let (_, hj) = &centroids[j];
        let t0 = Instant::now();
        let cost = metric.emd(hi, hj);
        let dt = t0.elapsed();
        cross_costs.push(cost);
        cross_times.push(dt);
        if sampled < 5 {
            println!("  pair ({:>3}, {:>3}): cross-cost = {:.6}", i, j, cost);
        }
        sampled += 1;
    }
    println!("\n=========================================================");
    println!("RESULTS");
    println!("=========================================================");
    print_cost_stats("self-cost  OT_eps(mu, mu)", &self_costs);
    print_cost_stats("cross-cost OT_eps(mu, nu)", &cross_costs);
    let mean_self = mean(&self_costs);
    let mean_cross = mean(&cross_costs);
    let max_self = self_costs.iter().copied().fold(f32::MIN, f32::max);
    let min_cross = cross_costs.iter().copied().fold(f32::MAX, f32::min);
    println!("\n--- bias magnitude ---");
    println!("mean(self) / mean(cross) = {:.4}", mean_self / mean_cross);
    println!("max(self)  / mean(cross) = {:.4}", max_self / mean_cross);
    println!("max(self)  / min(cross)  = {:.4}", max_self / min_cross);
    println!("\n--- timing ---");
    let mean_self_us =
        self_times.iter().map(Duration::as_micros).sum::<u128>() as f64 / self_times.len() as f64;
    let mean_cross_us =
        cross_times.iter().map(Duration::as_micros).sum::<u128>() as f64 / cross_times.len() as f64;
    println!("mean Sinkhorn call (self):  {:.1} µs", mean_self_us);
    println!("mean Sinkhorn call (cross): {:.1} µs", mean_cross_us);
    println!("\n--- debias cost projection ---");
    let n_flop = Street::Flop.n_abstractions();
    let n_pref = Street::Pref.n_abstractions();
    let total_pairs_flop = n_flop * (n_flop - 1) / 2;
    let total_pairs_pref = n_pref * (n_pref - 1) / 2;
    let extra_self_calls = n_flop + n_pref;
    println!(
        "flop pairwise distances: {} crosses + {} self-debias = {:+.2}% extra Sinkhorn work",
        total_pairs_flop,
        n_flop,
        100.0 * (n_flop as f64 / total_pairs_flop as f64)
    );
    println!(
        "pref pairwise distances: {} crosses + {} self-debias = {:+.2}% extra Sinkhorn work",
        total_pairs_pref,
        n_pref,
        100.0 * (n_pref as f64 / total_pairs_pref as f64)
    );
    let extra_us = extra_self_calls as f64 * mean_self_us;
    println!(
        "absolute extra time (one-shot, all self-debias calls): ~{:.1} ms",
        extra_us / 1000.0
    );
}

fn mean(xs: &[f32]) -> f32 {
    xs.iter().sum::<f32>() / xs.len() as f32
}

fn percentile(xs: &mut Vec<f32>, p: f32) -> f32 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((xs.len() - 1) as f32 * p).round() as usize;
    xs[idx]
}

fn print_cost_stats(label: &str, xs: &[f32]) {
    let mut sorted = xs.to_vec();
    let m = mean(xs);
    let p10 = percentile(&mut sorted, 0.10);
    let p50 = percentile(&mut sorted, 0.50);
    let p90 = percentile(&mut sorted, 0.90);
    let mn = sorted.first().copied().unwrap_or(0.0);
    let mx = sorted.last().copied().unwrap_or(0.0);
    println!(
        "\n{}: n={}, min={:.6}, p10={:.6}, p50={:.6}, mean={:.6}, p90={:.6}, max={:.6}",
        label,
        xs.len(),
        mn,
        p10,
        p50,
        m,
        p90,
        mx,
    );
}

/// Loads centroids whose source abstraction lives on `street`. Each centroid
/// is a Histogram over `street.next()` abstractions, reconstructed from the
/// `transitions` table.
async fn load_centroids(
    client: &tokio_postgres::Client,
    street: Street,
) -> Vec<(Abstraction, Histogram)> {
    let next = street.next();
    let sql = "SELECT prev, next, dx FROM transitions";
    let rows = client.query(sql, &[]).await.expect("query transitions");
    let mut grouped: BTreeMap<i16, Vec<(i16, f32)>> = BTreeMap::new();
    for row in rows {
        let prev: i16 = row.get(0);
        let next_abs: i16 = row.get(1);
        let dx: f32 = row.get(2);
        if Abstraction::from(prev).street() == street {
            grouped.entry(prev).or_default().push((next_abs, dx));
        }
    }
    grouped
        .into_iter()
        .map(|(prev, entries)| {
            let abs = Abstraction::from(prev);
            let mut hist = Histogram::empty(next);
            for (next_abs, dx) in entries {
                let count = (dx * 1000.0).round() as usize;
                if count > 0 {
                    hist.set(Abstraction::from(next_abs), count);
                }
            }
            (abs, hist)
        })
        .collect()
}
