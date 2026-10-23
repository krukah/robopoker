use serde::Deserialize;

/// Slumbot's single response shape, returned by both endpoints.
#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub token: Option<String>,
    pub client_pos: usize,
    pub hole_cards: Vec<String>,
    pub board: Vec<String>,
    pub action: String,
    pub old_action: Option<String>,
    #[serde(default)]
    pub winnings: Option<i64>,
}

impl Response {
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn client_pos(&self) -> usize {
        self.client_pos
    }

    pub fn hole_cards(&self) -> &[String] {
        &self.hole_cards
    }

    pub fn board(&self) -> &[String] {
        &self.board
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn old_action(&self) -> Option<&str> {
        self.old_action.as_deref()
    }

    pub fn winnings(&self) -> Option<i64> {
        self.winnings
    }
}
