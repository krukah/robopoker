use super::*;

/// One phase's frames as a lazy, stateful iterator: exploration probes
/// grown one edge per frame — deduped against their predecessor (no jitter)
/// and trimmed to at most [`SPAN`] — then the chosen line that closes the
/// phase. The dedup memory, fill count, and chosen-line cursor that a
/// closure would smuggle live here as honest fields.
pub struct Span<'film, G, P>
where
    G: Scenic,
    P: Palette,
{
    film: &'film Film<'film, G, P>,
    phase: usize,
    chosen: Vec<Slot>,
    queue: std::vec::IntoIter<(Probe, usize)>,
    zero: Frame,
    prior: Option<Frame>,
    fill: usize,
    tail: usize,
}

impl<'film, G, P> Span<'film, G, P>
where
    G: Scenic,
    P: Palette,
{
    pub fn new(film: &'film Film<'film, G, P>, phase: usize) -> Self {
        let chosen = film.scene().stints()[phase].picks().to_vec();
        let zero = film.state(phase, &chosen, CHOSEN, 1);
        let queue = Probe::ordered(film.scene(), phase)
            .into_iter()
            .flat_map(|probe| (1..=probe.line().len()).map(move |depth| (probe.clone(), depth)))
            .collect::<Vec<(Probe, usize)>>()
            .into_iter();
        Self {
            film,
            phase,
            chosen,
            queue,
            zero,
            prior: None,
            fill: 0,
            tail: 0,
        }
    }

    fn film(&self) -> &'film Film<'film, G, P> {
        self.film
    }

    fn phase(&self) -> usize {
        self.phase
    }

    fn chosen(&self) -> &[Slot] {
        &self.chosen
    }

    fn queue_mut(&mut self) -> &mut std::vec::IntoIter<(Probe, usize)> {
        &mut self.queue
    }

    fn zero(&self) -> &Frame {
        &self.zero
    }

    fn prior(&self) -> Option<&Frame> {
        self.prior.as_ref()
    }

    fn prior_mut(&mut self) -> &mut Option<Frame> {
        &mut self.prior
    }

    fn fill(&self) -> usize {
        self.fill
    }

    fn fill_mut(&mut self) -> &mut usize {
        &mut self.fill
    }

    fn tail(&self) -> usize {
        self.tail
    }

    fn tail_mut(&mut self) -> &mut usize {
        &mut self.tail
    }
}

impl<G, P> Iterator for Span<'_, G, P>
where
    G: Scenic,
    P: Palette,
{
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        let need = SPAN - self.chosen().len();
        while self.fill() < need {
            let Some((probe, depth)) = self.queue_mut().next() else {
                break;
            };
            let body = self.film().state(self.phase(), probe.line(), probe.gain(), depth);
            if self.prior() == Some(&body) || (self.fill() == need - 1 && &body == self.zero()) {
                continue;
            }
            *self.prior_mut() = Some(body.clone());
            *self.fill_mut() += 1;
            return Some(body);
        }
        (self.tail() < self.chosen().len()).then(|| {
            *self.tail_mut() += 1;
            self.film().state(self.phase(), self.chosen(), CHOSEN, self.tail())
        })
    }
}
