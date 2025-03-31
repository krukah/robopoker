/// just a wrapper for child nodes that haven't yet
/// been sampled from, so it's half Node (parent)
/// and half Game (child) with the "birthing" Edge
/// thrown in there too. everything an Encoder needs to
/// make some children.
pub type Branch<E, G> = (E, G, petgraph::graph::NodeIndex);
