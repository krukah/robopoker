pub struct PolicyAction {
    pub action: Action,
    pub weight: f32,
}

pub struct PolicyVector {
    pub actions: Vec<PolicyAction>,
}
