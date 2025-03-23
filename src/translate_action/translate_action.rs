use crate::Chips;

/// Provides the return type for translate_action() - see below for more details.
#[allow(dead_code)]
#[derive(Debug)]
pub struct TranslatedAction {
    smaller_size: Chips,
    larger_size: Chips,
    smaller_percentage: f64,
    larger_percentage: f64,
}

/// Translates an incoming bet size to a randomized choice of a smaller and larger bet size.
///
/// # Arguments
/// * `pot_size` - The current size of the pot
/// * `opponent_bet` - The actual bet made by the opponent
/// * `action_abstraction` - The list of allowed sizes to translate to.
///
/// # Returns
/// * Translated Action - contains action sizes and the percentage to use both respective size (out
///   of 1.0). When returning for only one size, smaller_size and larger_size will be set to the
///   same value, smaller_percentage will be set to 1.0, and larger_size will bet set to 0.0.
///
/// # Panics
/// * When pot_size is less than 1 chip
/// * When opponent_bet is less than 1 chip
/// * When action_abstraction isn't sorted in ascending order, doesn't contain only unique values,
///   contains less than two actions, or contains any actions less than 1 chip.
pub fn translate_action(
    pot_size: Chips,
    opponent_bet: Chips,
    action_abstraction: &[Chips],
) -> TranslatedAction {
    if pot_size <= 1 || opponent_bet < 1 || action_abstraction.into_iter().any(|&size| size < 1) {
        panic!("pot_size, opponent_bet, and all action_abstraction sizes must be at least 1 chip.")
    }
    if action_abstraction.len() < 2 {
        panic!("action_abstraction must have at least 2 elements.")
    }
    if !(action_abstraction
        .into_iter()
        .find(|&&bet_size| bet_size < 1)
        .is_none())
    {
        panic!("action_abstraction actions must all be at least 1 chip.")
    }

    for chunk in action_abstraction.windows(2) {
        if chunk[0] >= chunk[1] {
            panic!("action_abstraction must be sorted in ascending order and contain no repeated values.")
        }
    }

    // If opponent_bet is itself an option in action_abstraction then no need to randomize!
    if action_abstraction.contains(&opponent_bet) {
        return TranslatedAction {
            smaller_size: opponent_bet,
            larger_size: opponent_bet,
            smaller_percentage: 1.0,
            larger_percentage: 0.0,
        };
    }

    // If opponent_bet is outside the range of bets in the action_abstraction then there's no
    // point in randomizing. As in the paper, all we can do in such cases is to simply use
    // the closet bet (i.e. smallest or largest size) 100% of the time.
    if opponent_bet < action_abstraction[0] {
        return TranslatedAction {
            smaller_size: action_abstraction[0],
            larger_size: action_abstraction[0],
            smaller_percentage: 1.0,
            larger_percentage: 0.0,
        };
    }
    let largest_abstraction_size = action_abstraction
        .last()
        .copied()
        .expect("Should never see this; we verified above that it has >= 2 elements.");
    if opponent_bet > largest_abstraction_size {
        return TranslatedAction {
            smaller_size: largest_abstraction_size,
            larger_size: largest_abstraction_size,
            smaller_percentage: 1.0,
            larger_percentage: 0.0,
        };
    }

    // Now we've finally verified the inputs are good and that we cannot early return. Therefore:
    // we can, and need to, choose two actions to translate opponent_bet to (as well as calculate
    // the percentages we should randomize between them).

    let actions_to_randomize_between = action_abstraction
        .windows(2)
        .find(|&chunk| opponent_bet > chunk[0] && opponent_bet < chunk[1])
        .expect("Early returns above should have made it impossible to not find a match here.");

    let pot_size_f64 = pot_size as f64;
    let probabilty_smaller_bet = calc_pseudo_harmonic_mapping(
        // Scale down everything by pot to ensure we provide scale invariance. (This is not
        // explained super clearly in the paper, but I believe it's *why* they scaled everything
        // down to be relative to a pot size of 1 when they were actually analyzing things.)
        actions_to_randomize_between[0] as f64 / pot_size_f64,
        actions_to_randomize_between[1] as f64 / pot_size_f64,
        opponent_bet as f64 / pot_size_f64,
    );

    TranslatedAction {
        smaller_size: actions_to_randomize_between[0],
        larger_size: actions_to_randomize_between[1],

        smaller_percentage: probabilty_smaller_bet,
        larger_percentage: 1.0 - probabilty_smaller_bet,
    }
}

/// Calculates psuedo-harmonic mapping percentage to use the smaller bet.
///
/// NOTE: INPUTS MUST ALL BE NORMALIZED RELATIVE TO THE POT SIZE (ie pot = 1.0).
/// (Hence the '_ratio' in each name.)
///
/// Formula: f_A,B (x) = ((B - x) * (1 + A)) / ((B - A) * (1 + x))
/// Where: A=smaller_bet/pot, B=larger_bet/pot, x=opponent_bet/pot
fn calc_pseudo_harmonic_mapping(
    smaller_bet_ratio: f64,
    larger_bet_ratio: f64,
    opponent_bet_ratio: f64,
) -> f64 {
    let numerator = (larger_bet_ratio - opponent_bet_ratio) * (1.0 + smaller_bet_ratio);
    let denominator = (larger_bet_ratio - smaller_bet_ratio) * (1.0 + opponent_bet_ratio);

    // Prevent effective division by zero from quietly returning bad results.
    if denominator.abs() < f64::EPSILON {
        panic!("Denominator evaluates to approximately zero for the given inputs.");
    }

    numerator / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path_calculation() {
        // Arbitrarily choosing 5 decimal places.
        let precision = 0.00001;

        let translation = translate_action(100, 50, [11, 25, 75, 133, 30101].as_slice());

        assert_eq!(translation.smaller_size, 25);
        assert_eq!(translation.larger_size, 75);

        // When A=0.25, B=0.75, x=0.5,
        // f(A, B, x) = ((B - x) * (1 + A)) / ((B - A) * (1 + x))
        //            = 0.25 * 1.25 / (.5 * 1.5)
        //            = 0.3125 / 0.75
        //            = (5/16) * (4/3)
        //            = 5/12
        //            = 0.4166666666...
        assert!((translation.smaller_percentage - 0.41667).abs() < precision);
        assert!((translation.larger_percentage - 0.58333).abs() < precision);
    }

    #[test]
    fn test_no_max_i32_overflow() {
        // Arbitrarily choosing to go out to 5 decimal places.
        let precision = 0.00001;

        let translation = translate_action(1000, 500, [1, 250, 750, i16::MAX].as_slice());

        assert_eq!(translation.smaller_size, 250);
        assert_eq!(translation.larger_size, 750);
        assert!((translation.smaller_percentage - 0.41667).abs() < precision);
        assert!((translation.larger_percentage - 0.58333).abs() < precision);
    }

    #[test]
    fn test_size_in_abstraction() {
        let translation = translate_action(1000, 6, [2, 6, 9].as_slice());

        assert_eq!(translation.smaller_size, 6);
        assert_eq!(translation.larger_size, 6);
        assert!((translation.smaller_percentage - 1.0).abs() < f64::EPSILON);
        assert!(translation.larger_percentage.abs() < f64::EPSILON);
    }

    #[test]
    fn test_size_smaller_than_any_in_abstraction() {
        let translation = translate_action(1000, 8, [10, 20, 30].as_slice());

        assert_eq!(translation.smaller_size, 10);
        assert_eq!(translation.larger_size, 10);
        assert!((translation.smaller_percentage - 1.0).abs() < f64::EPSILON);
        assert!(translation.larger_percentage.abs() < f64::EPSILON);
    }
    #[test]
    fn test_size_larger_than_any_in_abstraction() {
        let translation = translate_action(1000, 37, [10, 20, 30].as_slice());

        assert_eq!(translation.smaller_size, 30);
        assert_eq!(translation.larger_size, 30);
        assert!((translation.smaller_percentage - 1.0).abs() < f64::EPSILON);
        assert!(translation.larger_percentage.abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic]
    fn test_disallows_zero_opponent_bet_size() {
        translate_action(100, 0, [5, 6].as_slice());
    }

    #[test]
    #[should_panic]
    fn test_disallows_negative_opponent_bet_size() {
        translate_action(100, -4, [5, 6].as_slice());
    }

    #[test]
    #[should_panic]
    fn test_disallows_zero_pot_size() {
        translate_action(0, 1, [5, 6].as_slice());
    }

    #[test]
    #[should_panic]
    fn test_disallows_negative_pot_size() {
        translate_action(-4, 1, [5, 6].as_slice());
    }

    #[test]
    #[should_panic]
    fn test_disallows_zero_chip_action_abstraction() {
        translate_action(1000, 1, [0, 4, 6].as_slice());
    }

    #[test]
    #[should_panic]
    fn test_disallows_negative_chip_action_abstraction() {
        translate_action(1000, 1, [-4, 4, 6].as_slice());
    }

    #[test]
    #[should_panic]
    fn test_disallows_empty_action_abstraction() {
        translate_action(1000, 1, [].as_slice());
    }

    #[test]
    #[should_panic]
    fn test_disallows_unsorted_action_abstraction() {
        translate_action(1000, 3, [2, 5, 4, 6].as_slice());
    }

    // See "Table 1: Effect of increasing A while holding B = 1 and x = 0.25 fixed".
    //
    // NOTE: Tests the internal calc_pseudo_harmonic_mapping(), not the public translate_action().
    #[test]
    fn test_replicates_paper_table_1_results() {
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
