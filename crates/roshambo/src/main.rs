mod consts;
use consts::*;
use mccfr::*;

use roshambo::simplex;
use roshambo::*;

fn main() {
    let mut solver = Tabular::<RpsEncoder, DiscountedRegret, LinearWeight, PluribusSampling, 1>::default();
    let mut p1snaps = Vec::new();
    let mut p2snaps = Vec::new();
    p1snaps.push(simplex::Simplex::snapshot(&solver, RpsTurn::P1));
    p2snaps.push(simplex::Simplex::snapshot(&solver, RpsTurn::P2));
    for _ in 0..T {
        solver.step();
        if solver.profile().t() % SAMPLE_INTERVAL == 0 {
            p1snaps.push(simplex::Simplex::snapshot(&solver, RpsTurn::P1));
            p2snaps.push(simplex::Simplex::snapshot(&solver, RpsTurn::P2));
        }
    }
    simplex::Simplex::generate(&p1snaps, &p2snaps, "target/simplex.html");
    simplex::Simplex::generate_3d(&p1snaps, &p2snaps, "target/simplex3d.html");
    simplex::Simplex::generate_dual(&p1snaps, &p2snaps, "target/simplex-dual.html");
    println!("wrote target/simplex{{,.3d,-dual}}.html ({} snapshots)", p1snaps.len());
}
