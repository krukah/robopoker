use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Semaphore;

const BASE: &str = "https://slumbot.com/slumbot/api";

/// Shared reqwest client for the whole process. Clones are cheap (the
/// struct is `Arc`-backed internally) and reusing it lets the TCP
/// connection pool and TLS session cache be shared across every
/// variant benchmark + error-retry reconnect. Fresh `reqwest::Client`
/// instances would each do their own TLS handshake.
static HTTP: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

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

#[derive(Debug, Serialize)]
struct NewHandRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
}

#[derive(Debug, Serialize)]
struct ActRequest {
    token: String,
    incr: String,
}

/// Shared concurrency limiter for the slumbot HTTP surface. When the
/// one-container runner spawns multiple variant benchmarks in parallel,
/// they each hold a clone of the same `Throttle` and serialize their
/// requests through it — no variant can exceed the global in-flight cap.
///
/// A zero-capacity throttle is never constructed (at least one permit is
/// guaranteed by [`Throttle::new`]), so every acquire call eventually
/// succeeds.
#[derive(Clone)]
pub struct Throttle(Arc<Semaphore>);

impl Throttle {
    /// Global cap on simultaneous in-flight requests across all holders
    /// of this clone.
    pub fn new(max_inflight: usize) -> Self {
        Self(Arc::new(Semaphore::new(max_inflight.max(1))))
    }
}

pub struct Client {
    http: reqwest::Client,
    token: Option<String>,
    throttle: Option<Throttle>,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Self {
            http: HTTP.clone(),
            token: None,
            throttle: None,
        }
    }

    pub fn with_throttle(mut self, throttle: Throttle) -> Self {
        self.throttle = Some(throttle);
        self
    }

    async fn acquire(&self) -> Option<tokio::sync::OwnedSemaphorePermit> {
        match self.throttle.as_ref() {
            Some(t) => t.0.clone().acquire_owned().await.ok(),
            None => None,
        }
    }

    pub async fn new_hand(&mut self) -> anyhow::Result<Response> {
        let permit = self.acquire().await;
        let text = self
            .http
            .post(format!("{}/new_hand", BASE))
            .json(&NewHandRequest {
                token: self.token.clone(),
            })
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        drop(permit);
        let resp = serde_json::from_str::<Response>(&text)
            .map_err(|e| anyhow::anyhow!("new_hand decode: {e}\nbody: {text}"))?;
        self.token = resp.token.clone().or(self.token.take());
        Ok(resp)
    }

    pub async fn act(&mut self, incr: &str) -> anyhow::Result<Response> {
        let permit = self.acquire().await;
        let text = self
            .http
            .post(format!("{}/act", BASE))
            .json(&ActRequest {
                token: self.token.clone().ok_or(anyhow::anyhow!("no token"))?,
                incr: incr.to_string(),
            })
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        drop(permit);
        let resp = serde_json::from_str::<Response>(&text)
            .map_err(|e| anyhow::anyhow!("act decode: {e}\nbody: {text}"))?;
        self.token = resp.token.clone().or(self.token.take());
        Ok(resp)
    }
}
