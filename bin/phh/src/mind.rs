//! A [`Brain`] as an [`Oracle`] — the binding between the bot zoo and the evaluation.
//!
//! `phh` asks its questions through [`phh::Oracle`], which is one synchronous method. Every
//! cell of the bot-config hypercube already answers exactly that question through [`Brain`], so
//! [`Mind`] is a newtype and one delegation, and [`crate::minds::Minds::mind`] mirrors `parlor::zoo` to pick the
//! cell.

use parlor::Brain;
use parlor::Mount;
use parlor::Tag;
use phh::Oracle;
use phh::Policy;

/// One cell of the bot-config hypercube, answering [`Oracle`].
pub struct Mind<B>
where
    B: Brain,
{
    brain: B,
    solve: bool,
}

impl<B> Mind<B>
where
    B: Brain,
{
    fn brain(&self) -> &B {
        &self.brain
    }

    fn solve(&self) -> bool {
        self.solve
    }
}

impl<B> Oracle for Mind<B>
where
    B: Brain,
{
    fn policy(&self, witness: &kicker::Witness) -> Policy {
        if self.solve() {
            self.brain().distrib(witness).into_iter().collect()
        } else {
            self.brain().policy(witness).into_iter().collect()
        }
    }
}

impl<B> Mind<B>
where
    B: Brain + Mount + 'static,
{
    pub(crate) fn boxed(tag: Tag, model: &'static nlhe::Flagship, solve: bool) -> Box<dyn Oracle> {
        Box::new(Self {
            brain: B::mount(tag, model),
            solve,
        })
    }
}
