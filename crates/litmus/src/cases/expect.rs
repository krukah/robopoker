use super::*;
use serde::{Deserialize, Serialize};

/// Tolerance specification. Different fields apply to different `kind`s.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Expect {
    pub acceptable_max: Option<f32>,
    pub acceptable_min: Option<f32>,
    pub max_abs_diff: Option<f32>,
    pub direction: Option<Direction>,
}
impl Expect {
    pub fn acceptable_max(&self) -> Option<f32> {
        self.acceptable_max
    }

    pub fn acceptable_min(&self) -> Option<f32> {
        self.acceptable_min
    }

    pub fn max_abs_diff(&self) -> Option<f32> {
        self.max_abs_diff
    }

    pub fn direction(&self) -> Option<Direction> {
        self.direction
    }

    /// Merge category default with case override; case fields win.
    pub fn merged(default: Option<&Expect>, override_: Option<&Expect>) -> Expect {
        override_
            .into_iter()
            .fold(default.cloned().unwrap_or_default(), Self::overlay)
    }

    fn overlay(self, o: &Expect) -> Expect {
        Expect {
            acceptable_max: o.acceptable_max().or(self.acceptable_max()),
            acceptable_min: o.acceptable_min().or(self.acceptable_min()),
            max_abs_diff: o.max_abs_diff().or(self.max_abs_diff()),
            direction: o.direction().or(self.direction()),
        }
    }
}
