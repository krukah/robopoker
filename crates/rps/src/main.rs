use rbp_mccfr::*;
use rbp_rps::simplex;
use rbp_rps::*;

const T: usize = 250_000;
const SAMPLE_INTERVAL: usize = 10;

fn main() {
    let mut solver = Rps::<DiscountedRegret, LinearWeight, PluribusSampling>::default();
    let mut p1snaps = Vec::new();
    let mut p2snaps = Vec::new();
    p1snaps.push(simplex::snapshot(&solver, RpsTurn::P1));
    p2snaps.push(simplex::snapshot(&solver, RpsTurn::P2));
    for _ in 0..T {
        solver.step();
        if solver.profile().t() % SAMPLE_INTERVAL == 0 {
            p1snaps.push(simplex::snapshot(&solver, RpsTurn::P1));
            p2snaps.push(simplex::snapshot(&solver, RpsTurn::P2));
        }
    }
    simplex::generate(&p1snaps, &p2snaps, "target/simplex.html");
    simplex::generate_3d(&p1snaps, &p2snaps, "target/simplex3d.html");
    simplex::generate_dual(&p1snaps, &p2snaps, "target/simplex-dual.html");
    println!(
        "wrote target/simplex{{,.3d,-dual}}.html ({} snapshots)",
        p1snaps.len()
    );
}
