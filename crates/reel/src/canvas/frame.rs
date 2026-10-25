use crate::*;

/// One rendered instant of the reel: an ordered set of pure marks. Frame
/// equality is structural, which is what powers the no-jitter dedup (a
/// frame never repeats its predecessor).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Frame(Vec<Mark>);

impl Frame {
    pub fn new(marks: Vec<Mark>) -> Self {
        Self(marks)
    }

    pub fn marks(&self) -> &[Mark] {
        &self.0
    }

    /// This frame translated by (dx, dy) — for composing films on one canvas.
    pub fn shift(self, dx: f64, dy: f64) -> Self {
        Self(self.0.into_iter().map(|mark| mark.shift(dx, dy)).collect())
    }

    /// The concatenated SVG elements of every mark.
    pub fn body(&self) -> String {
        self.0.iter().map(Mark::svg).collect()
    }

    /// A complete standalone SVG document of this single frame.
    pub fn document(&self, w: f64, h: f64, bg: &str) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w:.0} {h:.0}" width="{w:.0}" height="{h:.0}" font-family="{MONO}"><rect x="0" y="0" width="{w:.0}" height="{h:.0}" fill="{bg}"/>{}</svg>"#,
            self.body(),
        )
    }
}

impl Frame {
    /// The animated flipbook of a frame sequence: every frame in one SVG,
    /// revealed by discrete SMIL opacity — declarative, so it plays inline on
    /// GitHub. Shared by [`crate::Film::svg`] and the asset emitter.
    pub fn book(frames: &[Frame], w: f64, h: f64, bg: &str) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w:.0} {h:.0}" width="{w:.0}" height="{h:.0}" font-family="{MONO}"><rect x="0" y="0" width="{w:.0}" height="{h:.0}" fill="{bg}"/>{}</svg>"#,
            frames
                .iter()
                .enumerate()
                .map(|(i, frame)| Frame::blink(i, frames.len(), frame))
                .collect::<String>(),
        )
    }

    /// One frame's SMIL group: visible only during its slice of the loop,
    /// flipped discretely so playback never fades.
    fn blink(i: usize, n: usize, frame: &Frame) -> String {
        let dur = n as f64 / crate::FPS as f64;
        let (t0, t1) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
        let (first, wave) = match i {
            0 => ("1", format!("values=\"1;0;0\" keyTimes=\"0;{t1:.4};1\"")),
            _ => ("0", format!("values=\"0;1;0;0\" keyTimes=\"0;{t0:.4};{t1:.4};1\"")),
        };
        format!(
            r#"<g opacity="{first}"><animate attributeName="opacity" dur="{dur}s" repeatCount="indefinite" calcMode="discrete" {wave}/>{}</g>"#,
            frame.body(),
        )
    }
}
