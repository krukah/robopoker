#![allow(dead_code)]

use std::hash::Hash;

/// Regret Minimization in Games with Incomplete Information. Advances in Neural Information Processing Systems, 20.
/// Zinkevich, M., Bowling, M., Burch, N., Cao, Y., Johanson, M., Tamblyn, I., & Rocco, M. (2007).

// Marker types
type Utility = f32;
type Probability = f32;

// A finite set N of players, including chance
trait Player: Eq {}

// A finite set of possible actions
trait Action: Eq {
    type Player;

    fn player(&self) -> &Self::Player;
}

enum NodeKind {
    Decision,
    Terminal,
    Chance,
}
// Omnipotent, complete state of current game
trait Node: NodeBounds {
    // fn kind(&self) -> NodeKind;
    // fn info(&self) -> &Self::Info;
    // fn history(&self) -> Vec<&Self::Action>;

    fn parent(&self) -> Option<&Self>;
    fn precedent(&self) -> Option<&Self::Action>;

    fn children(&self) -> Vec<&Self>;
    fn available(&self) -> Vec<&Self::Action>;

    fn player(&self) -> &Self::Player;
    fn value(&self, _: &Self::Player) -> Utility;

    fn follow(&self, action: &Self::Action) -> &Self {
        self.children()
            .iter()
            .find(|child| child.precedent().unwrap() == action)
            .unwrap()
    }
    fn descendants(&self) -> Vec<&Self> {
        match self.children().len() {
            0 => vec![&self],
            _ => self
                .children()
                .iter()
                .map(|child| child.descendants())
                .flatten()
                .collect(),
        }
    }
}
// All known information at a given node, up to any abstractions. Think of it as a distribution over the unknown game state.
trait Info: InfoBounds + Hash {
    fn roots(&self) -> Vec<&Self::Node>;

    fn endpoints(&self) -> Vec<&Self::Node> {
        self.roots()
            .iter()
            .map(|node| node.descendants())
            .flatten()
            .collect()
    }
    fn available(&self) -> Vec<&Self::Action> {
        self.roots().into_iter().next().unwrap().available()
    }
    fn player(&self) -> &Self::Player {
        self.roots().iter().next().unwrap().player()
    }
}

// A policy is a distribution over A(Ii)
trait Policy: PolicyBounds {
    fn weight(&self, action: &Self::Action) -> Probability;
}

// A strategy of player i σi in an extensive game is a function that assigns a policy to each h ∈ H, therefore Ii ∈ Ii
trait Strategy: StrategyBounds {
    fn policy(&self, node: &Self::Node) -> &Self::Policy;
}

// A profile σ consists of a strategy for each player, σ1,σ2,..., equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
trait Profile: ProfileBounds {
    // fn always(&self, action: &Self::Action) -> &Self;
    fn strategy(&self, player: &Self::Player) -> &Self::Strategy;

    /// aka instantaneous regret.
    /// utility flows FROM the future, we integrate over all info::(root, action, leaf) space for the infoset
    fn gain(&self, info: &Self::Info, action: &Self::Action) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.decision_value(root, action) - self.weighted_value(root))
            .sum::<Utility>()
    }
    fn weighted_value(&self, root: &Self::Node) -> Utility {
        root.available()
            .iter()
            .map(|action| self.decision_value(root, action) * self.reach(root, action))
            .sum::<Utility>()
    }
    fn decision_value(&self, root: &Self::Node, action: &Self::Action) -> Utility {
        root.follow(action)
            .descendants()
            .iter()
            .map(|leaf| self.terminal_value(root, leaf))
            .sum::<Utility>()
    }
    fn terminal_value(&self, root: &Self::Node, leaf: &Self::Node) -> Utility {
        leaf.value(root.player()) //                U_i(h)
            * self.cfactual_reach(root) //         -π_i(h)
            * self.relative_reach(root, leaf) //   +π_i(h -> z)
    }

    /// probability flows INTO the future, we measure each node's "alignment" with the profile. reach = dot product of policy and birth
    fn reach(&self, node: &Self::Node, action: &Self::Action) -> Probability {
        self.strategy(node.player()).policy(node).weight(action)
    }
    fn historic_reach(&self, node: &Self::Node) -> Probability {
        // alternative implemenation: iterate over node.history().zip(node.ancestors())
        let mut reach = 1.0;
        let mut child = node;
        while let Some(parent) = child.parent() {
            reach *= self.reach(parent, child.precedent().unwrap());
            child = parent;
        }
        reach
    }
    fn cfactual_reach(&self, node: &Self::Node) -> Probability {
        // alternative implemenation: iterate over node.history().zip(node.ancestors())
        let mut reach = 1.0;
        let mut child = node;
        while let Some(parent) = child.parent() {
            if parent.player() != node.player() {
                reach *= self.reach(parent, child.precedent().unwrap());
            }
            child = parent;
        }
        reach
    }
    fn relative_reach(&self, root: &Self::Node, leaf: &Self::Node) -> Probability {
        // this could use an optimization
        self.historic_reach(leaf) / self.historic_reach(root) //? DIV BY ZERO
    }
}

// Training happens over discrete time steps, so we'll index steps into it's own data structure.
trait Step: StepBounds {
    fn new(profile: Self::Profile) -> Self;
    fn profile(&self) -> &Self::Profile; //? mutable or immutable?
}

// A full solver has a sequence of steps, and a final profile
trait Solver: SolverBounds {
    fn steps(&self) -> &mut Vec<Self::Step>;
    fn next(&self) -> Option<Self::Step>;

    /// Loops over simple n_iter < max_iter convergence criteria and returns ~ Nash Equilibrium
    fn solve(&self) -> &Self::Profile {
        while let Some(step) = self.next() {
            self.steps().push(step);
        }
        self.steps().last().unwrap().profile()
    }
    /// aka average cumulative regret. backward pass through game tree propagates regret
    fn regret(&self, info: &Self::Info, action: &Self::Action) -> Utility {
        self.steps()
            .iter()
            .map(|step| step.profile())
            .map(|profile| profile.gain(info, action))
            .enumerate()
            .map(|(i, gain)| i as Utility * gain) // linear CFR
            .sum::<Utility>()
            / self.num_steps() as Utility //? DIV BY ZERO
    }

    /// Convergence progress
    fn num_steps(&self) -> usize {
        self.steps().len()
    }
    fn max_steps(&self) -> usize {
        10_000
    }
}

trait NodeBounds {
    type Info: Info<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;
}
trait InfoBounds {
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;
}

trait PolicyBounds {
    type Action: Action<Player = Self::Player>;
    type Player: Player;
}
trait StrategyBounds {
    type Policy: Policy<Player = Self::Player, Action = Self::Action>;
    type Info: Info<Player = Self::Player, Action = Self::Action, Node = Self::Node>;
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;
}
trait ProfileBounds {
    type Strategy: Strategy<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
    >;
    type Info: Info<Player = Self::Player, Action = Self::Action, Node = Self::Node>;
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;
}

trait StepBounds {
    type Profile: Profile<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
        Strategy = Self::Strategy,
    >;
    type Strategy: Strategy<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
    >;
    type Info: Info<Player = Self::Player, Action = Self::Action, Node = Self::Node>;
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;
}
trait SolverBounds {
    type Step: Step<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
        Strategy = Self::Strategy,
        Profile = Self::Profile,
    >;
    type Profile: Profile<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
        Strategy = Self::Strategy,
    >;
    type Strategy: Strategy<
        Player = Self::Player,
        Action = Self::Action,
        Node = Self::Node,
        Info = Self::Info,
    >;
    type Info: Info<Player = Self::Player, Action = Self::Action, Node = Self::Node>;
    type Node: Node<Player = Self::Player, Action = Self::Action>;
    type Action: Action<Player = Self::Player>;
    type Player: Player;
}

/*

19. Superhuman AI for multiplayer poker. (https://science.sciencemag.org/content/early/2019/07/10/science.aay2400) Science, July 11th.
19. Regret Circuits: Composability of Regret Minimizers. In Proceedings of the International Conference on Machine Learning (ICML), 2019. arXiv version. (https://arxiv.org/abs/1811.02540)
19. Stable-Predictive Optimistic Counterfactual Regret Minimization. In ICML. arXiv version. (https://arxiv.org/pdf/1902.04982.pdf)
19. Deep Counterfactual Regret Minimization In ICML. Early version (https://arxiv.org/pdf/1811.00164.pdf) in NeurIPS-18 Deep RL Workshop, 2018.
19. Solving Imperfect-Information Games via Discounted Regret Minimization (https://arxiv.org/pdf/1809.04040.pdf). In Proceedings of the AAAI Conference on Artificial Intelligence (AAAI). Outstanding Paper Honorable Mention, one of four papers receiving special recognition out of 1,150 accepted papers and 7,095 submissions.
19. Online Convex Optimization for Sequential Decision Processes and Extensive-Form Games (http://www.cs.cmu.edu/~gfarina/2018/laminar-regret-aaai19/). In Proceedings of the AAAI Conference on Artificial Intelligence (AAAI).
19. Quasi-Perfect Stackelberg Equilibrium (http://www.cs.cmu.edu/~gfarina/2018/qp-stackelberg-aaai19/). In Proceedings of the AAAI Conference on Artificial Intelligence (AAAI).
19. Stable-Predictive Optimistic Counterfactual Regret Minimization (https://arxiv.org/pdf/1902.04982.pdf). arXiv.
18. Superhuman AI for heads-up no-limit poker: Libratus beats top professionals. (http://science.sciencemag.org/content/early/2017/12/15/science.aao1733) Science, full Research Article.
18. Deep Counterfactual Regret Minimization (https://arxiv.org/pdf/1811.00164.pdf). NeurIPS Deep Reinforcement Learning Workshop. *Oral Presentation*.
18. Faster algorithms for extensive-form game solving via improved smoothing functions. (https://rdcu.be/8EyP) Mathematical Programming, Series A. Abstract published in EC-17.
18. Depth-Limited Solving for Imperfect-Information Games. (https://arxiv.org/pdf/1805.08195.pdf) In Proc. Neural Information Processing Systems (NeurIPS).
18. A Unified Framework for Extensive-Form Game Abstraction with Bounds. In NIPS. Early version (http://www.cs.cmu.edu/~ckroer/papers/unified_abstraction_framework_ai_cubed.pdf) in IJCAI-18 AI^3 workshop.
18. Practical Exact Algorithm for Trembling-Hand Equilibrium Refinements in Games. (http://www.cs.cmu.edu/~gfarina/2017/trembling-lp-refinements-nips18/) In NeurIPS.
18. Solving Large Sequential Games with the Excessive Gap Technique. (https://arxiv.org/abs/1810.03063) In NeurIPS. Also Spotlight presentation.
18. Ex Ante Coordination and Collusion in Zero-Sum Multi-Player Extensive-Form Games. (http://www.cs.cmu.edu/~gfarina/2018/collusion-3players-nips18/) In NeurIPS.
18. Trembling-Hand Perfection in Extensive-Form Games with Commitment. (http://www.cs.cmu.edu/~ckroer/papers/stackelberg_perfection_ijcai18.pdf) In IJCAI.
18. Robust Stackelberg Equilibria in Extensive-Form Games and Extension to Limited Lookahead. (http://www.cs.cmu.edu/~ckroer/papers/robust.aaai18.pdf) In Proc. AAAI Conference on AI (AAAI).
17. Safe and Nested Subgame Solving for Imperfect-Information Games. (https://www.cs.cmu.edu/~noamb/papers/17-NIPS-Safe.pdf) In NIPS. * *Best Paper Award, out of 3,240 submissions.
17. Regret Minimization in Behaviorally-Constrained Zero-Sum Games. (http://www.cs.cmu.edu/~sandholm/behavioral.icml17.pdf) In Proc. International Conference on Machine Learning (ICML).
17. Reduced Space and Faster Convergence in Imperfect-Information Games via Pruning. (http://www.cs.cmu.edu/~sandholm/reducedSpace.icml17.pdf) In ICML.
17. Smoothing Method for Approximate Extensive-Form Perfect Equilibrium. (http://www.cs.cmu.edu/~sandholm/smoothingEFPE.ijcai17.pdf) In IJCAI. ArXiv version. (http://arxiv.org/abs/1705.09326)
17. Dynamic Thresholding and Pruning for Regret Minimization. (http://www.cs.cmu.edu/~sandholm/dynamicThresholding.aaai17.pdf) In AAAI.
16. Imperfect-Recall Abstractions with Bounds in Games. (http://www.cs.cmu.edu/~sandholm/imperfect-recall-abstraction-with-bounds.ec16.pdf) In Proc. ACM Conference on Economics and Computation (EC).
16. Strategy-Based Warm Starting for Regret Minimization in Games. In AAAI. Extended version with appendix. (http://www.cs.cmu.edu/~sandholm/warmStart.aaai16.withAppendixAndTypoFix.pdf)
15. Regret-Based Pruning in Extensive-Form Games. (http://www.cs.cmu.edu/~sandholm/cs15-892F15) In NIPS. Extended version. (http://www.cs.cmu.edu/~sandholm/regret-basedPruning.nips15.withAppendix.pdf)
15. Simultaneous Abstraction and Equilibrium Finding in Games. (http://www.cs.cmu.edu/~sandholm/simultaneous.ijcai15.pdf) In IJCAI.
15. Limited Lookahead in Imperfect-Information Games. (http://www.cs.cmu.edu/~sandholm/limited-look-ahead.ijcai15.pdf) IJCAI.
15. Faster First-Order Methods for Extensive-Form Game Solving. (http://www.cs.cmu.edu/~sandholm/faster.ec15.pdf) In EC.
15. Hierarchical Abstraction, Distributed Equilibrium Computation, and Post-Processing, with Application to a Champion No-Limit Texas Hold’em Agent. (http://www.cs.cmu.edu/~sandholm/hierarchical.aamas15.pdf) In Proc. Internat. Conference on Autonomous Agents and Multiagent Systems (AAMAS).
15. Discretization of Continuous Action Spaces in Extensive-Form Games. (http://www.cs.cmu.edu/~sandholm/discretization.aamas15.fromACM.pdf) In AAMAS.
15. Endgame Solving in Large Imperfect-Information Games. (http://www.cs.cmu.edu/~sandholm/endgame.aamas15.fromACM.pdf) In AAMAS.
14. Extensive-Form Game Abstraction With Bounds. (http://www.cs.cmu.edu/~sandholm/extensiveGameAbstraction.ec14.pdf) In EC.
14. Regret Transfer and Parameter Optimization. (http://www.cs.cmu.edu/~sandholm/regret_transfer.aaai14.pdf) In AAAI.
14. Potential-Aware Imperfect-Recall Abstraction with Earth Mover’s Distance in Imperfect-Information Games. (http://www.cs.cmu.edu/~sandholm/potential-aware_imperfect-recall.aaai14.pdf) In AAAI.
13. Action Translation in Extensive-Form Games with Large Action Spaces: Axioms, Paradoxes, and the Pseudo-Harmonic Mapping. (http://www.cs.cmu.edu/~sandholm/reverse%20mapping.ijcai13.pdf) In IJCAI.
12. Lossy Stochastic Game Abstraction with Bounds. (http://www.cs.cmu.edu/~sandholm/lossyStochasticGameAbstractionWBounds.ec12.pdf) In EC.
12. First-Order Algorithm with O(ln(1/epsilon)) Convergence for epsilon-Equilibrium in Two-Person Zero-Sum Games. (http://www.cs.cmu.edu/~sandholm/restart.MathProg12.pdf) Mathematical Programming 133(1-2), 279-298. Subsumes our AAAI-08 paper.
12. Strategy Purification and Thresholding: Effective Non-Equilibrium Approaches for Playing Large Games. (http://www.cs.cmu.edu/~sandholm/StrategyPurification_AAMAS2012_camera_ready_2.pdf) In AAMAS.
12. Tartanian5: A Heads-Up No-Limit Texas Hold'em Poker-Playing Program. (http://www.cs.cmu.edu/~sandholm/Tartanian_ACPC12_CR.pdf) Computer Poker Symposium at AAAI.
10. Smoothing techniques for computing Nash equilibria of sequential games. (http://www.cs.cmu.edu/~sandholm/proxtreeplex.MathOfOR.pdf) Mathematics of Operations Research 35(2), 494-512.
10. Computing Equilibria by Incorporating Qualitative Models (http://www.cs.cmu.edu/~sandholm/qualitative.aamas10.pdf). In AAMAS. Extended version (http://www.cs.cmu.edu/~sandholm/qualitative.TR10.pdf): CMU technical report CMU-CS-10-105.
10. Speeding Up Gradient-Based Algorithms for Sequential Games (Extended Abstract) (http://www.cs.cmu.edu/~sandholm/speedup.aamas10.pdf). In AAMAS.
09. Computing Equilibria in Multiplayer Stochastic Games of Imperfect Information (http://www.cs.cmu.edu/~sandholm/stochgames.ijcai09.pdf). In IJCAI.

 */
