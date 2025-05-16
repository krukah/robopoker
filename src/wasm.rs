use crate::cards::*;
use crate::gameplay::*;
use wasm_bindgen::prelude::*;

// Re-export types for JavaScript

#[wasm_bindgen]
pub struct WasmRank(Rank);

#[wasm_bindgen]
pub struct WasmSuit(Suit);

#[wasm_bindgen]
pub struct WasmCard(Card);

#[wasm_bindgen]
pub struct WasmHand(Hand);

#[wasm_bindgen]
pub struct WasmHole(Hole);

#[wasm_bindgen]
pub struct WasmDeck(Deck);

#[wasm_bindgen]
pub struct WasmGame(Game);

#[wasm_bindgen]
pub struct WasmBoard(Board);

#[wasm_bindgen]
pub struct WasmStreet(Street);

#[wasm_bindgen]
pub struct WasmAction(Action);

#[wasm_bindgen]
pub struct WasmKickers(Kickers);

#[wasm_bindgen]
pub struct WasmRanking(Ranking);

#[wasm_bindgen]
pub struct WasmStrength(Strength);

#[wasm_bindgen]
pub struct WasmEvaluator(Evaluator);

#[wasm_bindgen]
pub struct WasmObservation(Observation);

#[wasm_bindgen]
pub struct WasmAbstraction(Abstraction);

// Card implementation
#[wasm_bindgen]
impl WasmCard {
    #[wasm_bindgen(constructor)]
    pub fn new(rank: u8, suit: u8) -> Result<Self, JsValue> {
        let r = Rank::try_from(rank).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let s = Suit::try_from(suit).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self(Card::from((r, s))))
    }

    #[wasm_bindgen]
    pub fn rank(&self) -> WasmRank {
        WasmRank(self.0.rank())
    }

    #[wasm_bindgen]
    pub fn suit(&self) -> WasmSuit {
        WasmSuit(self.0.suit())
    }

    #[wasm_bindgen]
    pub fn from_string(s: &str) -> Result<Self, JsValue> {
        Ok(Card::try_from(s)
            .map_err(|e| JsValue::from_str(&e))
            .map(Self)?)
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn from_byte(byte: u8) -> Result<Self, JsValue> {
        Ok(Self(Card::from(byte)))
    }

    #[wasm_bindgen]
    pub fn into_byte(&self) -> u8 {
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
    pub fn from_string(s: &str) -> Result<Self, JsValue> {
        Ok(Self(Hand::try_from(s).map_err(|e| JsValue::from_str(&e))?))
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
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
        Self(Observation::from((pocket.0, public.0)))
    }

    #[wasm_bindgen]
    pub fn from_string(s: &str) -> Result<Self, JsValue> {
        Ok(Observation::try_from(s).map_err(|e| JsValue::from_str(&e))?).map(Self)
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn from_street(street: usize) -> Result<Self, JsValue> {
        Ok(Self(Observation::from(Street::from(street))))
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
    pub fn simulate(&self, n: usize) -> f64 {
        self.0.simulate(n) as f64
    }

    #[wasm_bindgen]
    pub fn equivalent(&self) -> String {
        self.0.equivalent()
    }
}

// Initialize function
#[wasm_bindgen(start)]
pub fn start() {
    // This function will be called when the WASM module is loaded
    console_error_panic_hook::set_once();
}

// Rank implementation
#[wasm_bindgen]
impl WasmRank {
    #[wasm_bindgen(constructor)]
    pub fn new(index: u8) -> Result<WasmRank, JsValue> {
        Ok(WasmRank(Rank::from(index)))
    }

    #[wasm_bindgen]
    pub fn into_u8(&self) -> u8 {
        self.0 as u8
    }

    #[wasm_bindgen]
    pub fn from_string(s: &str) -> Result<WasmRank, JsValue> {
        Rank::try_from(s)
            .map(WasmRank)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn lo(bits: u64) -> WasmRank {
        WasmRank(Rank::lo(bits))
    }

    #[wasm_bindgen]
    pub fn hi(bits: u64) -> WasmRank {
        WasmRank(Rank::hi(bits))
    }
}

// Suit implementation
#[wasm_bindgen]
impl WasmSuit {
    #[wasm_bindgen(constructor)]
    pub fn new(index: u8) -> Result<Self, JsValue> {
        Ok(Self(Suit::from(index)))
    }

    #[wasm_bindgen]
    pub fn into_u8(&self) -> u8 {
        self.0 as u8
    }

    #[wasm_bindgen]
    pub fn from_string(s: &str) -> Result<Self, JsValue> {
        Suit::try_from(s)
            .map(Self)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn all() -> js_sys::Array {
        Suit::all()
            .iter()
            .copied()
            .map(Self)
            .map(JsValue::from)
            .collect()
    }
}

// Hole implementation
#[wasm_bindgen]
impl WasmHole {
    #[wasm_bindgen(constructor)]
    pub fn new(card_a: &WasmCard, card_b: &WasmCard) -> Self {
        Self(Hole::from((card_a.0, card_b.0)))
    }

    #[wasm_bindgen]
    pub fn empty() -> Self {
        Self(Hole::empty())
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn from_string(s: &str) -> Result<Self, JsValue> {
        Hole::try_from(s)
            .map(WasmHole)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn into_hand(&self) -> WasmHand {
        WasmHand(Hand::from(self.0))
    }
}

// Deck implementation
#[wasm_bindgen]
impl WasmDeck {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(Deck::new())
    }

    #[wasm_bindgen]
    pub fn contains(&self, card: &WasmCard) -> bool {
        self.0.contains(&card.0)
    }

    #[wasm_bindgen]
    pub fn draw(&mut self) -> WasmCard {
        WasmCard(self.0.draw())
    }

    #[wasm_bindgen]
    pub fn deal(&mut self, street: &WasmStreet) -> WasmHand {
        WasmHand(self.0.deal(street.0))
    }

    #[wasm_bindgen]
    pub fn hole(&mut self) -> WasmHole {
        WasmHole(self.0.hole())
    }
}

// Board implementation
#[wasm_bindgen]
impl WasmBoard {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(Board::empty())
    }

    #[wasm_bindgen]
    pub fn add(&mut self, hand: &WasmHand) {
        self.0.add(hand.0)
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[wasm_bindgen]
    pub fn street(&self) -> WasmStreet {
        WasmStreet(self.0.street())
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// Street implementation
#[wasm_bindgen]
impl WasmStreet {
    #[wasm_bindgen(constructor)]
    pub fn new(index: usize) -> Self {
        Self(Street::from(index))
    }

    #[wasm_bindgen]
    pub fn next(&self) -> Self {
        Self(self.0.next())
    }

    #[wasm_bindgen]
    pub fn prev(&self) -> Self {
        Self(self.0.prev())
    }

    #[wasm_bindgen]
    pub fn n_observed(&self) -> usize {
        self.0.n_observed()
    }

    #[wasm_bindgen]
    pub fn n_revealed(&self) -> usize {
        self.0.n_revealed()
    }

    #[wasm_bindgen]
    pub fn index(&self) -> usize {
        self.0 as isize as usize
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn from_string(s: &str) -> Result<Self, JsValue> {
        Ok(Street::try_from(s)
            .map_err(|e| JsValue::from_str(&e))
            .map(Self)?)
    }
}

// Action implementation
#[wasm_bindgen]
impl WasmAction {
    #[wasm_bindgen(js_name = "fold")]
    pub fn fold() -> Self {
        Self(Action::Fold)
    }

    #[wasm_bindgen(js_name = "check")]
    pub fn check() -> Self {
        Self(Action::Check)
    }

    #[wasm_bindgen(js_name = "call")]
    pub fn call(amount: i32) -> Self {
        Self(Action::Call(amount as crate::Chips))
    }

    #[wasm_bindgen(js_name = "raise")]
    pub fn raise(amount: i32) -> Self {
        Self(Action::Raise(amount as crate::Chips))
    }

    #[wasm_bindgen(js_name = "shove")]
    pub fn shove(amount: i32) -> Self {
        Self(Action::Shove(amount as crate::Chips))
    }

    #[wasm_bindgen(js_name = "blind")]
    pub fn blind(amount: i32) -> Self {
        Self(Action::Blind(amount as crate::Chips))
    }

    #[wasm_bindgen(js_name = "draw")]
    pub fn draw(hand: &WasmHand) -> Self {
        Self(Action::Draw(hand.0))
    }

    #[wasm_bindgen]
    pub fn is_choice(&self) -> bool {
        self.0.is_choice()
    }

    #[wasm_bindgen]
    pub fn is_chance(&self) -> bool {
        self.0.is_chance()
    }

    #[wasm_bindgen]
    pub fn is_aggro(&self) -> bool {
        self.0.is_aggro()
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        String::from(self.0)
    }

    #[wasm_bindgen]
    pub fn from_string(s: &str) -> Result<Self, JsValue> {
        Ok(Action::try_from(s)
            .map_err(|e| JsValue::from_str(&e))
            .map(Self)?)
    }
}

// Kickers implementation
#[wasm_bindgen]
impl WasmKickers {
    #[wasm_bindgen]
    pub fn into_u16(&self) -> u16 {
        self.0.into()
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn ranks(&self) -> js_sys::Array {
        Vec::<Rank>::from(self.0)
            .into_iter()
            .map(WasmRank)
            .map(JsValue::from)
            .collect()
    }
}

// Ranking implementation
#[wasm_bindgen]
impl WasmRanking {
    #[wasm_bindgen]
    pub fn n_kickers(&self) -> usize {
        self.0.n_kickers()
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// Strength implementation
#[wasm_bindgen]
impl WasmStrength {
    #[wasm_bindgen(constructor)]
    pub fn from_hand(hand: &WasmHand) -> Self {
        Self(Strength::from(hand.0))
    }

    #[wasm_bindgen]
    pub fn kicks(&self) -> WasmKickers {
        WasmKickers(self.0.kicks)
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }
}

// Evaluator implementation
#[wasm_bindgen]
impl WasmEvaluator {
    #[wasm_bindgen(constructor)]
    pub fn new(hand: &WasmHand) -> Self {
        Self(Evaluator::from(hand.0))
    }

    #[wasm_bindgen]
    pub fn find_ranking(&self) -> WasmRanking {
        WasmRanking(self.0.find_ranking())
    }

    #[wasm_bindgen]
    pub fn find_kickers(&self, ranking: &WasmRanking) -> WasmKickers {
        WasmKickers(self.0.find_kickers(ranking.0))
    }
}

// Abstraction implementation
#[wasm_bindgen]
impl WasmAbstraction {
    #[wasm_bindgen(constructor)]
    pub fn new(street: &WasmStreet, index: usize) -> Self {
        Self(Abstraction::from((street.0, index)))
    }

    #[wasm_bindgen]
    pub fn size() -> usize {
        Abstraction::size()
    }

    #[wasm_bindgen]
    pub fn range() -> js_sys::Array {
        Abstraction::range().map(Self).map(JsValue::from).collect()
    }

    #[wasm_bindgen]
    pub fn street(&self) -> WasmStreet {
        WasmStreet(self.0.street())
    }

    #[wasm_bindgen]
    pub fn index(&self) -> usize {
        self.0.index()
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen]
    pub fn from_string(s: &str) -> Result<Self, JsValue> {
        Ok(Abstraction::try_from(s)
            .map_err(|e| JsValue::from_str(&e))
            .map(Self)?)
    }
}

// Game implementation
#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn root() -> Self {
        Self(Game::root())
    }

    #[wasm_bindgen]
    pub fn apply(&self, action: &WasmAction) -> Self {
        Self(self.0.apply(action.0))
    }

    #[wasm_bindgen]
    pub fn legal(&self) -> js_sys::Array {
        self.0
            .legal()
            .into_iter()
            .map(WasmAction)
            .map(JsValue::from)
            .collect()
    }

    #[wasm_bindgen]
    pub fn pot(&self) -> i32 {
        self.0.pot() as i32
    }

    #[wasm_bindgen]
    pub fn board(&self) -> WasmBoard {
        WasmBoard(self.0.board())
    }

    #[wasm_bindgen]
    pub fn street(&self) -> WasmStreet {
        WasmStreet(self.0.street())
    }

    #[wasm_bindgen]
    pub fn into_string(&self) -> String {
        self.0.to_string()
    }
}
