use crate::*;
use kuhn::*;
use neural::*;
use roshambo::*;
use wasm_bindgen::prelude::*;

/// One training run, owned by one Web Worker. [`DeepHyperParams`] is process-global and set
/// once, so a new run with new knobs needs a new worker.
#[wasm_bindgen]
pub struct Session(Box<dyn Lab>);

#[wasm_bindgen]
impl Session {
    /// `game` is `kuhn` or `rps`; `net` is `candle` or `mock`; `knobs` is a JSON [`Knobs`].
    #[wasm_bindgen(constructor)]
    pub fn new(game: &str, net: &str, knobs: &str) -> Result<Session, JsError> {
        DeepHyperParams::from(serde_json::from_str::<Knobs>(knobs)?)
            .init()
            .map_err(|_| JsError::new("knobs are already set in this worker"))?;
        match (game, net) {
            ("kuhn", "candle") => Ok(Self(Box::new(Run::<KuhnEmbedder, CfrNet<KuhnEmbedder>>::default()))),
            ("kuhn", "mock") => Ok(Self(Box::new(Run::<KuhnEmbedder, MockNet<KuhnInfo>>::default()))),
            ("rps", "candle") => Ok(Self(Box::new(Run::<RpsEmbedder, CfrNet<RpsEmbedder>>::default()))),
            ("rps", "mock") => Ok(Self(Box::new(Run::<RpsEmbedder, MockNet<RpsTurn>>::default()))),
            _ => Err(JsError::new(&format!("no run for game {game} and net {net}"))),
        }
    }

    /// Runs `epochs` epochs and returns a JSON [`Frame`].
    pub fn step(&mut self, epochs: usize) -> String {
        self.0.step(epochs).json()
    }

    /// Fits the strategy net and returns a JSON [`Frame`] with averages and exploitability.
    pub fn checkpoint(&mut self) -> String {
        self.0.checkpoint().json()
    }
}
