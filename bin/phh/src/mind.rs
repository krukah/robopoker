//! A [`Brain`] as an [`Oracle`] — the binding between the bot zoo and the
//! evaluation.
//!
//! `phh` asks its questions through [`phh::Oracle`], which is one synchronous
//! method. Every cell of the bot-config hypercube already answers exactly that
//! question through [`Brain`], so [`Mind`] is a newtype and one delegation, and
//! [`mind`] mirrors `parlor::zoo` to pick the cell.
//!
//! The nesting layer `zoo` adds under `Translation::Exact` is deliberately not
//! mirrored: it exists to handle off-tree villain bets, and a rollout only ever
//! produces on-tree ones.

use parlor::Blueprint;
use parlor::Brain;
use parlor::Depth;
use parlor::Dirac;
use parlor::Mount;
use parlor::Tag;
use parlor::VariantExt;
use parlor::World;
use phh::Oracle;
use phh::Policy;
use pokerkit::Variant;

/// One cell of the bot-config hypercube, answering [`Oracle`].
pub struct Mind<B>
where
    B: Brain,
{
    brain: B,
    solve: bool,
}

impl<B> Oracle for Mind<B>
where
    B: Brain,
{
    fn policy(&self, witness: &kicker::Witness) -> Policy {
        if self.solve {
            self.brain.distrib(witness).into_iter().collect()
        } else {
            self.brain.policy(witness).into_iter().collect()
        }
    }
}

impl<B> Mind<B>
where
    B: Brain + Mount + 'static,
{
    fn boxed(tag: Tag, model: &'static nlhe::Flagship, solve: bool) -> Box<dyn Oracle> {
        Box::new(Self {
            brain: B::mount(tag, model),
            solve,
        })
    }
}

/// Looks a bot up by its `--variant` token.
///
/// `solve` chooses which half of the brain answers. [`Brain::policy`] is the
/// in-memory blueprint lookup — microseconds. [`Brain::distrib`] is that plus a
/// fresh subgame CFR re-solve on any variant that carries one — seconds. A duel
/// over the whole corpus makes on the order of a million calls, so `--solve` is
/// only ever affordable against a small `--limit`; without it every cell reduces
/// to its blueprint and the variant token changes nothing.
#[rustfmt::skip]
pub fn mind(variant: Variant, model: &'static nlhe::Flagship, solve: bool) -> Option<Box<dyn Oracle>> {
    let tag = variant.tag()?;
    Some(match (tag.config.depth, tag.config.world, tag.config.dirac) {
        (false, false, false) => Mind::<                  Blueprint   >::boxed(tag, model, solve),
        (false, false, true ) => Mind::<Dirac<            Blueprint  >>::boxed(tag, model, solve),
        (true,  false, false) => Mind::<            Depth<Blueprint>  >::boxed(tag, model, solve),
        (true,  false, true ) => Mind::<Dirac<      Depth<Blueprint> >>::boxed(tag, model, solve),
        (false, true,  false) => Mind::<      World<      Blueprint > >::boxed(tag, model, solve),
        (false, true,  true ) => Mind::<Dirac<World<      Blueprint >>>::boxed(tag, model, solve),
        (true,  true,  false) => Mind::<      World<Depth<Blueprint>> >::boxed(tag, model, solve),
        (true,  true,  true ) => Mind::<Dirac<World<Depth<Blueprint>>>>::boxed(tag, model, solve),
    })
}
