use super::*;
use rbp_core::*;
use std::collections::HashSet;
use tokio::sync::mpsc::UnboundedSender;

/// Manages physical table state: seats, player presence, and communication.
/// Separates player lifecycle from game logic.
#[derive(Debug)]
pub struct Table {
    senders: Vec<Option<UnboundedSender<Event>>>,
    disconnected: HashSet<Position>,
}

impl Table {
    /// Creates a table with capacity for n players.
    pub fn new(n: usize) -> Self {
        Self {
            senders: vec![None; n],
            disconnected: HashSet::new(),
        }
    }
    /// Seats a player at the given position.
    pub fn sit(&mut self, pos: Position, sender: UnboundedSender<Event>) {
        if pos < self.senders.len() {
            self.senders[pos] = Some(sender);
        }
    }
    /// Marks a player as disconnected.
    pub fn disconnect(&mut self, pos: Position) {
        self.disconnected.insert(pos);
    }
    /// Checks if a player is disconnected.
    pub fn is_disconnected(&self, pos: Position) -> bool {
        self.disconnected.contains(&pos)
    }
    /// Returns the number of seats.
    pub fn seats(&self) -> usize {
        self.senders.len()
    }
    /// Returns the number of connected players.
    pub fn connected_count(&self) -> usize {
        self.senders
            .iter()
            .enumerate()
            .filter(|(i, s)| s.is_some() && !self.disconnected.contains(i))
            .count()
    }
    /// Gets the player sender at a position.
    pub fn sender(&self, pos: Position) -> Option<&UnboundedSender<Event>> {
        self.senders.get(pos).and_then(|s| s.as_ref())
    }
    /// Sends an event to a specific player.
    pub fn unicast(&self, pos: Position, event: Event) {
        log::debug!("[table] unicast to P{}: {}", pos, event);
        match self.sender(pos).map(|inbox| inbox.send(event)) {
            Some(Ok(())) => log::debug!("[table] unicast to P{} succeeded", pos),
            Some(Err(e)) => log::warn!("[table] unicast to P{} failed: {:?}", pos, e),
            None => log::warn!("[table] unicast to P{}: no such player", pos),
        }
    }
    /// Sends an event to all players.
    pub fn broadcast(&self, event: Event) {
        log::debug!("[table] broadcast: {}", event);
        self.senders.iter().enumerate().for_each(|(i, sender)| {
            if let Some(inbox) = sender {
                match inbox.send(event.clone()) {
                    Ok(()) => {}
                    Err(e) => log::warn!("[table] broadcast to P{} failed: {:?}", i, e),
                }
            }
        });
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new(rbp_core::N)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;
    #[test]
    fn table_seats() {
        let table = Table::new(2);
        assert_eq!(table.seats(), 2);
        assert_eq!(table.connected_count(), 0);
    }
    #[test]
    fn table_sit_and_disconnect() {
        let mut table = Table::new(2);
        let (tx, _) = unbounded_channel();
        table.sit(0, tx);
        assert_eq!(table.connected_count(), 1);
        assert!(!table.is_disconnected(0));
        table.disconnect(0);
        assert!(table.is_disconnected(0));
        assert_eq!(table.connected_count(), 0);
    }
}
