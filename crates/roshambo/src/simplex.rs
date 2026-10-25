use crate::*;
use mccfr::*;
use pokerkit::*;

/// Barycentric coordinates on a 2-simplex (equilateral triangle).
///
/// Encodes a probability distribution over (R, P, S) and provides
/// conversion to 2D Cartesian coordinates for visualization.
/// Vertices: R=(0,0), P=(1,0), S=(0.5, sqrt(3)/2).
#[derive(Debug, Clone, Copy)]
pub struct Simplex([Probability; 3]);

impl Simplex {
    pub fn cartesian(&self) -> (f32, f32) {
        let [_, p, s] = self.0;
        let x = p + 0.5 * s;
        let y = s * 3f32.sqrt() / 2.0;
        (x, y)
    }

    pub fn coords(&self) -> [Probability; 3] {
        self.0
    }
}

impl From<&Over<RpsEdge, Policy>> for Simplex {
    fn from(policy: &Over<RpsEdge, Policy>) -> Self {
        let mut coords = [0.0; 3];
        for &(edge, prob) in policy {
            match edge {
                RpsEdge::R => coords[0] = prob,
                RpsEdge::P => coords[1] = prob,
                RpsEdge::S => coords[2] = prob,
            }
        }
        Self(coords)
    }
}

impl Simplex {
    /// Capture a snapshot for a given player from the solver profile.
    pub fn snapshot<R, W, S>(solver: &Tabular<RpsEncoder, R, W, S, 1>, turn: RpsTurn) -> Snapshot
    where
        R: RegretSchedule,
        W: WeightSchedule,
        S: SamplingScheme,
    {
        let profile = solver.profile();
        let iterated = Simplex::from(&Profile::iterated_strategy(profile, &turn));
        let averaged = Simplex::from(&Profile::averaged_strategy(profile, &turn));
        let edges = [RpsEdge::R, RpsEdge::P, RpsEdge::S];
        Snapshot::new(
            profile.t(),
            iterated,
            averaged,
            edges.map(|e| profile.cum_regret(&turn, &e)),
            edges.map(|e| profile.cum_weight(&turn, &e)),
            edges.map(|e| profile.cum_payoff(&turn, &e)),
            edges.map(|e| profile.cum_visits(&turn, &e)),
        )
    }

    /// Generate a self-contained HTML file with SVG simplex visualization.
    pub fn generate(p1: &[Snapshot], p2: &[Snapshot], path: &str) {
        let html = include_str!("../assets/simplex2d.html")
            .replace("__P1_DATA__", &Simplex::snapshots_json(p1))
            .replace("__P2_DATA__", &Simplex::snapshots_json(p2));
        std::fs::write(path, html).expect("failed to write HTML");
    }

    /// Generate a self-contained HTML file with Three.js 3D simplex visualization.
    pub fn generate_3d(p1: &[Snapshot], p2: &[Snapshot], path: &str) {
        let html = include_str!("../assets/simplex3d.html")
            .replace("__P1_DATA__", &Simplex::snapshots_json(p1))
            .replace("__P2_DATA__", &Simplex::snapshots_json(p2));
        std::fs::write(path, html).expect("failed to write 3D HTML");
    }

    /// Generate a dual-panel HTML: 2D policy simplex + 3D regret vectors, synchronized.
    pub fn generate_dual(p1: &[Snapshot], p2: &[Snapshot], path: &str) {
        let html = include_str!("../assets/simplex-dual.html")
            .replace("__P1_DATA__", &Simplex::snapshots_json(p1))
            .replace("__P2_DATA__", &Simplex::snapshots_json(p2));
        std::fs::write(path, html).expect("failed to write dual HTML");
    }

    fn snapshots_json(snaps: &[Snapshot]) -> String {
        let entries = snaps
            .iter()
            .map(|s| {
                format!(
                    r#"{{"epoch":{},"iterated":[{:.6},{:.6},{:.6}],"averaged":[{:.6},{:.6},{:.6}],"regrets":[{:.4},{:.4},{:.4}],"weights":[{:.4},{:.4},{:.4}],"payoffs":[{:.6},{:.6},{:.6}],"visits":[{},{},{}]}}"#,
                    s.epoch(),
                    s.iterated().coords()[0], s.iterated().coords()[1], s.iterated().coords()[2],
                    s.averaged().coords()[0], s.averaged().coords()[1], s.averaged().coords()[2],
                    s.regrets()[0], s.regrets()[1], s.regrets()[2],
                    s.weights()[0], s.weights()[1], s.weights()[2],
                    s.payoffs()[0], s.payoffs()[1], s.payoffs()[2],
                    s.visits()[0], s.visits()[1], s.visits()[2],
                )
            })
            .collect::<Vec<String>>()
            .join(",");
        format!("[{entries}]")
    }
}
