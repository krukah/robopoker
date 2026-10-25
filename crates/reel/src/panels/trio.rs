use crate::*;

/// Horizontal gap between composed films.
pub const SPACER: f64 = 24.0;

/// Several panels composed side by side on one canvas — how the hero sheet,
/// the policy dial, and the villain sheet become one triplet. Panels share
/// the reel clock; marks are pure data, so composition is translation.
pub struct Trio {
    frames: Vec<Frame>,
    size: (f64, f64),
}

impl Trio {
    pub fn new(panels: &[&dyn Panel]) -> Self {
        let strips = panels
            .iter()
            .map(|panel| (panel.frames(), panel.size()))
            .collect::<Vec<(Vec<Frame>, (f64, f64))>>();
        let count = strips.iter().map(|(frames, _)| frames.len()).min().unwrap_or(0);
        let high = strips.iter().map(|(_, (_, h))| *h).fold(0.0, f64::max);
        let wide = strips.iter().map(|(_, (w, _))| *w).sum::<f64>() + SPACER * (strips.len() as f64 - 1.0);
        let frames = (0..count)
            .map(|i| {
                let mut dx = 0.0;
                Frame::new(strips.iter().fold(Vec::new(), |mut marks, (frames, (w, h))| {
                    marks.extend(frames[i].clone().shift(dx, (high - h) / 2.0).marks().to_vec());
                    dx += w + SPACER;
                    marks
                }))
            })
            .collect();
        Self {
            frames,
            size: (wide, high),
        }
    }
}

impl Panel for Trio {
    fn size(&self) -> (f64, f64) {
        self.size
    }

    fn frames(&self) -> Vec<Frame> {
        self.frames.clone()
    }
}
