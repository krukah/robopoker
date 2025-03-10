use crate::cards::card::Card;
use crate::cards::hand::Hand;
use crate::cards::observation::Observation;
use crate::cards::rank::Rank;
use crate::cards::street::Street;
use crate::cards::suit::Suit;
use wasm_bindgen::prelude::*;

// Re-export types for JavaScript
#[wasm_bindgen]
pub struct WasmCard(Card);

#[wasm_bindgen]
pub struct WasmHand(Hand);

#[wasm_bindgen]
pub struct WasmObservation(Observation);

// Card implementation
#[wasm_bindgen]
impl WasmCard {
    #[wasm_bindgen(constructor)]
    pub fn new(rank: u8, suit: u8) -> Result<WasmCard, JsValue> {
        let r = Rank::try_from(rank).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let s = Suit::try_from(suit).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmCard(Card::from((r, s))))
    }

    #[wasm_bindgen]
    pub fn from_string(card_str: &str) -> Result<WasmCard, JsValue> {
        let card = Card::try_from(card_str).map_err(|e| JsValue::from_str(&e))?;
        Ok(WasmCard(card))
    }

    #[wasm_bindgen]
    pub fn rank(&self) -> u8 {
        self.0.rank().into()
    }

    #[wasm_bindgen]
    pub fn suit(&self) -> u8 {
        self.0.suit().into()
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn to_byte(&self) -> u8 {
        self.0.into()
    }
}

// Hand implementation
#[wasm_bindgen]
impl WasmHand {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmHand(Hand::empty())
    }

    #[wasm_bindgen]
    pub fn from_string(hand_str: &str) -> Result<WasmHand, JsValue> {
        let hand = Hand::try_from(hand_str).map_err(|e| JsValue::from_str(&e))?;
        Ok(WasmHand(hand))
    }

    #[wasm_bindgen]
    pub fn add_card(&mut self, card: &WasmCard) {
        let mut hand = self.0;
        hand = Hand::add(hand, Hand::from(card.0));
        self.0 = hand;
    }

    #[wasm_bindgen]
    pub fn remove_card(&mut self, card: &WasmCard) {
        self.0.remove(card.0);
    }

    #[wasm_bindgen]
    pub fn contains(&self, card: &WasmCard) -> bool {
        self.0.contains(&card.0)
    }

    #[wasm_bindgen]
    pub fn size(&self) -> usize {
        self.0.size()
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn to_card_array(&self) -> js_sys::Array {
        self.0
            .into_iter()
            .map(WasmCard)
            .map(JsValue::from)
            .collect::<js_sys::Array>()
    }
}

// Observation implementation
#[wasm_bindgen]
impl WasmObservation {
    #[wasm_bindgen(constructor)]
    pub fn new(pocket: &WasmHand, public: &WasmHand) -> Self {
        WasmObservation(Observation::from((pocket.0, public.0)))
    }

    #[wasm_bindgen]
    pub fn from_string(obs_str: &str) -> Result<WasmObservation, JsValue> {
        let obs = Observation::try_from(obs_str).map_err(|e| JsValue::from_str(&e))?;
        Ok(WasmObservation(obs))
    }

    #[wasm_bindgen]
    pub fn from_street(street: usize) -> Result<WasmObservation, JsValue> {
        Ok(WasmObservation(Observation::from(Street::from(street))))
    }

    #[wasm_bindgen]
    pub fn street(&self) -> usize {
        self.0.street() as usize
    }

    #[wasm_bindgen]
    pub fn pocket(&self) -> WasmHand {
        WasmHand(self.0.pocket().clone())
    }

    #[wasm_bindgen]
    pub fn public(&self) -> WasmHand {
        WasmHand(self.0.public().clone())
    }

    #[wasm_bindgen]
    pub fn equity(&self) -> f64 {
        self.0.equity() as f64
    }

    #[wasm_bindgen]
    pub fn estimate(&self) -> f64 {
        self.0.estimate() as f64
    }

    #[wasm_bindgen]
    pub fn equivalent(&self) -> String {
        self.0.equivalent()
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// Initialize function
#[wasm_bindgen(start)]
pub fn start() {
    // This function will be called when the WASM module is loaded
    console_error_panic_hook::set_once();
}
