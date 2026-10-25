use super::*;

#[derive(Clone, Debug)]
pub(crate) struct Row {
    case: String,
    hand: String,
    history: String,
    queried: String,
    spr: String,
    legal: String,
    verdict: Verdict,
}

impl Row {
    pub(crate) fn new(
        case: String,
        hand: String,
        history: String,
        queried: String,
        spr: String,
        legal: String,
        verdict: Verdict,
    ) -> Self {
        Self {
            case,
            hand,
            history,
            queried,
            spr,
            legal,
            verdict,
        }
    }

    pub fn case(&self) -> &String {
        &self.case
    }

    pub fn hand(&self) -> &String {
        &self.hand
    }

    pub fn history(&self) -> &String {
        &self.history
    }

    pub fn queried(&self) -> &String {
        &self.queried
    }

    pub fn spr(&self) -> &String {
        &self.spr
    }

    pub fn legal(&self) -> &String {
        &self.legal
    }

    pub fn verdict(&self) -> &Verdict {
        &self.verdict
    }
}
