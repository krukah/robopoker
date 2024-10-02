use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::cards::strength::Strength;
use crate::clustering::histogram::Histogram;

#[allow(dead_code)]
struct Potential;
impl std::fmt::Display for Potential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let observation = Observation::from(Street::Turn);
        let distribution = Histogram::from(observation.clone());
        let strength = Strength::from(observation.clone());
        let equity = distribution.equity();
        // Display the histogram
        writeln!(f, "{}", distribution)?;
        // Mark the point on the x-axis corresponding to the value "ev"
        let n_x_bins = 32;
        let x = (equity * n_x_bins as f32).floor() as usize;
        let x = x.min(n_x_bins - 1);
        for i in 0..n_x_bins {
            if i == x {
                write!(f, "^")?;
            } else {
                write!(f, " ")?;
            }
        }
        writeln!(f)?;
        writeln!(f, "{}", observation)?;
        writeln!(f, "{}", strength)?;
        writeln!(f, "{:.2}%", equity * 100.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn am_i_pretty() {
        println!("{}", Potential);
    }
}
