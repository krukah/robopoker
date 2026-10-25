//! Slumbot's two-endpoint HTTP surface.

use super::*;
use bout::Throttle;
use serde::Serialize;
use std::sync::LazyLock;

const BASE: &str = "https://slumbot.com/slumbot/api";

/// Shared reqwest client for the whole process. Clones are cheap (the struct
/// is `Arc`-backed internally) and reusing it lets the TCP connection pool
/// and TLS session cache be shared across every variant benchmark and every
/// error-retry reconnect. Fresh `reqwest::Client` instances would each do
/// their own TLS handshake.
static HTTP: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// One session's connection: the shared HTTP pool, this session's token, and
/// the process-wide in-flight cap.
pub struct Client {
    http: reqwest::Client,
    token: Option<String>,
    throttle: Throttle,
}

impl Client {
    /// A session with no token yet — the first `new_hand` mints one.
    pub fn new(throttle: Throttle) -> Self {
        Self {
            http: HTTP.clone(),
            token: None,
            throttle,
        }
    }

    fn http(&self) -> &reqwest::Client {
        &self.http
    }

    fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn token_mut(&mut self) -> &mut Option<String> {
        &mut self.token
    }

    fn throttle(&self) -> &Throttle {
        &self.throttle
    }

    /// Start a hand, carrying the session token forward if we have one.
    pub async fn new_hand(&mut self) -> anyhow::Result<Response> {
        let token = self.token().map(str::to_owned);
        self.post("new_hand", &NewHand { token }).await
    }

    /// Send hero's action as a Slumbot increment string.
    pub async fn act(&mut self, incr: &str) -> anyhow::Result<Response> {
        let token = self
            .token()
            .map(str::to_owned)
            .ok_or_else(|| anyhow::anyhow!("no token"))?;
        let incr = incr.to_string();
        self.post("act", &Act { token, incr }).await
    }

    /// POST a body, decode the response, and refresh the session token.
    /// Decode failures carry the raw body — Slumbot answers some malformed
    /// requests with prose, and the text is the whole diagnosis.
    async fn post<B>(&mut self, route: &str, body: &B) -> anyhow::Result<Response>
    where
        B: Serialize,
    {
        let permit = self.throttle().acquire().await;
        let text = self
            .http()
            .post(format!("{BASE}/{route}"))
            .json(body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        drop(permit);
        let response = serde_json::from_str::<Response>(&text)
            .map_err(|e| anyhow::anyhow!("{route} decode: {e}\nbody: {text}"))?;
        *self.token_mut() = response.token().map(str::to_owned).or(self.token_mut().take());
        Ok(response)
    }
}
