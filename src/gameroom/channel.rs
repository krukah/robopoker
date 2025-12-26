/// Coupled sender/receiver pair for bidirectional communication.
/// Ensures the channel endpoints stay together and share the same type.
#[derive(Debug)]
pub struct Channel<T> {
    tx: tokio::sync::mpsc::UnboundedSender<T>,
    rx: tokio::sync::mpsc::UnboundedReceiver<T>,
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self { tx, rx }
    }
}

impl<T> Channel<T> {
    pub fn tx(&mut self) -> &mut tokio::sync::mpsc::UnboundedSender<T> {
        &mut self.tx
    }

    pub fn rx(&mut self) -> &mut tokio::sync::mpsc::UnboundedReceiver<T> {
        &mut self.rx
    }
}
