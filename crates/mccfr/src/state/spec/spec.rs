use crate::*;

/// A game spec, named by its info type `F`; encoders, profiles and solvers declare theirs in one
/// line. [`InfoOf`], [`GameOf`], [`EdgeOf`], [`TurnOf`] are routes down from `F`. Pin `F` to fix
/// everything, or label one edge (`F: CfrInfo<X: CfrPublic<G = …>>`) to fix the game below it.
pub trait CfrSpec: Sized {
    type F: CfrInfo;
}

/// An info type is its own spec, so [`EdgeOf`] and the rest read any info type directly.
/// This rules out a forwarding `impl CfrSpec for &X`: a downstream `&T: CfrInfo` would overlap it.
impl<I> CfrSpec for I
where
    I: CfrInfo,
{
    type F = I;
}

pub type InfoOf<X> = <X as CfrSpec>::F;

pub type GameOf<X> = <<InfoOf<X> as CfrInfo>::X as CfrPublic>::G;

pub type EdgeOf<X> = <GameOf<X> as CfrGame>::E;

pub type TurnOf<X> = <GameOf<X> as CfrGame>::T;

pub type PublicOf<X> = <InfoOf<X> as CfrInfo>::X;

pub type SecretOf<X> = <InfoOf<X> as CfrInfo>::Y;
