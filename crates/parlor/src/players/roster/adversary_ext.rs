use bouncer::Member;
use pokerkit::ID;

/// Gameroom-only methods on [`pokerkit::Adversary`]. Same split as [`crate::VariantExt`]:
/// the identity is pure data in `pokerkit`, the `bouncer` projection of
/// it lives here. An external opponent is a pseudo-member — it never
/// authenticates, it just owns the villain side of every recorded hand.
pub trait AdversaryExt {
    fn id(self) -> ID<Member>;
    fn member(self) -> Member;
}
