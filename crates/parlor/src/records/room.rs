/// Marker type for room identity.
/// The actual Room implementation lives in parlor.
/// This marker allows records to use `ID<Room>` without circular dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Room;
