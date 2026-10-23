use crate::mind::Mind;
use parlor::Blueprint;
use parlor::Depth;
use parlor::Dirac;
use parlor::VariantExt;
use parlor::World;
use phh::Oracle;
use pokerkit::Variant;

/// Oracle construction by variant.
pub(crate) struct Minds;
impl Minds {
    /// Looks a bot up by its `--variant` token.
    ///
    /// `solve` chooses which half of the brain answers. [`parlor::Brain::policy`] is the in-memory
    /// blueprint lookup — microseconds. [`parlor::Brain::distrib`] is that plus a fresh subgame CFR re-
    /// solve on any variant that carries one — seconds.
    #[rustfmt::skip]
    pub fn mind(variant: Variant, model: &'static nlhe::Flagship, solve: bool) -> Option<Box<dyn Oracle>> {
        let tag = variant.tag()?;
        Some(match (tag.config().depth(), tag.config().world(), tag.config().dirac()) {
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
}
