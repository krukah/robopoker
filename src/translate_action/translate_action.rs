/// Translates an incoming bet size to a randomized choice of a smaller and larger bet size.
///
/// # Arguments TODO
/// * `opponent_bet` - The actual bet (in chips) made by the opponent
/// * `pot_size` - The current size of the pot (in chips)
/// * TODO figure out the manner in which we want to accept the action abstraction
///   (e.g. min and max allowed bets + pot ratios vs something simpler)
///
/// # Returns
/// * TODO create a struct return type
///
/// # Panics
/// * TODO
use crate::Chips;

pub fn translate_action(opponent_bet: Chips, pot_size: Chips) -> f64 {
    if pot_size <= 1 {
        panic!("pot_size must be at least 1")
    }

    let smaller_bet_todo = 2;
    let larger_bet_todo = 5;

    let smaller_bet_f64 = smaller_bet_todo as f64;
    let larger_bet_f64 = larger_bet_todo as f64;
    let opponent_bet_f64 = opponent_bet as f64;
    let pot_size_f64 = pot_size as f64;

    let probabilty_smaller_bet = calc_pseudo_harmonic_mapping(
        // Scale down everything by pot to ensure we provide scale
        // invariance. (This is not explained clearly in the paper, but
        // I believe it's actually _why_ they scaled everything down so that it
        // was relative to a pot size of 1
        smaller_bet_f64 / pot_size_f64,
        larger_bet_f64 / pot_size_f64,
        opponent_bet_f64 / pot_size_f64,
    );
    let _probability_larger_bet = 1.0 - probabilty_smaller_bet;
    // TODO return both

    probabilty_smaller_bet
}

/// Calculates psuedo-harmonic mapping percentage to use the smaller bet.
///
/// NOTE: INPUTS MUST ALL BE NORMALIZED RELATIVE TO THE POT SIZE (ie pot = 1.0).
/// (Hence the '_ratio' in each name.)
///
/// Formula: f_A,B (x) = ((B - x) * (1 + A) / ((B - A) * (1 + x))
/// Where: A=smaller_bet/pot, B=larger_bet/pot, x=opponent_bet/pot
fn calc_pseudo_harmonic_mapping(
    smaller_bet_ratio: f64,
    larger_bet_ratio: f64,
    opponent_bet_ratio: f64,
) -> f64 {
    // Apply the pseudo-harmonic mapping formula with pre-scaled values
    let numerator = (larger_bet_ratio - opponent_bet_ratio) * (1.0 + smaller_bet_ratio);
    let denominator = (larger_bet_ratio - smaller_bet_ratio) * (1.0 + opponent_bet_ratio);

    // Check for division by zero in the final calculation
    if denominator.abs() < f64::EPSILON {
        panic!("Denominator evaluates to approximately zero for the given inputs");
    }

    numerator / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deliberate_fail() {
        assert_eq!(translate_action(4, 1000), 0.5);
    }

    // See "Table 1: Effect of increasing A while holding B = 1 and x = 0.25 fixed."
    // in the paper.
    #[test]
    fn replicate_paper_table_1_results() {
        let b = 1.0;
        let x = 0.25;

        assert_eq!(calc_pseudo_harmonic_mapping(0.000, b, x), 0.6);
        // The results table only reported these non-exact results out to 3 decimal places.
        let precision = 0.001;
        assert!((calc_pseudo_harmonic_mapping(0.001, b, x) - 0.601).abs() < precision);
        assert!((calc_pseudo_harmonic_mapping(0.010, b, x) - 0.612).abs() < precision);
        assert!((calc_pseudo_harmonic_mapping(0.050, b, x) - 0.663).abs() < precision);
        assert!((calc_pseudo_harmonic_mapping(0.100, b, x) - 0.733).abs() < precision);
    }
}
