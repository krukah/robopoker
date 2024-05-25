#![allow(dead_code)]

/// Type alias encapsulates numberical precision for units of utility.
pub(crate) type Utility = f32;

/// Type alias encapsulates numberical precision for units of probability.
pub(crate) type Probability = f32;

/// An element of the finite set N of players, including chance.
pub(crate) trait Player: Eq {}

/// An element of the finite set of possible actions.
pub(crate) trait Action: Eq + Copy {
    // required
    fn player(&self) -> &Self::APlayer;

    type APlayer: Player;
}

/// A node,  history, game state, etc. Omnipotent, complete state of current game.
pub(crate) trait Node<'t> {
    // required
    fn parent(&'t self) -> Option<&'t Self>;
    fn precedent(&'t self) -> Option<&'t Self::NAction>;
    fn children(&'t self) -> &Vec<&'t Self>;
    fn available(&'t self) -> &Vec<&'t Self::NAction>;
    fn chooser(&'t self) -> &'t Self::NPlayer;
    fn utility(&'t self, player: &Self::NPlayer) -> Utility;

    // provided
    fn follow(&'t self, action: &Self::NAction) -> &'t Self {
        self.children()
            .iter()
            .find(|child| action == child.precedent().unwrap())
            .unwrap()
    }
    fn descendants(&'t self) -> Vec<&'t Self> {
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

    type NPlayer: Player;
    type NAction: Action<APlayer = Self::NPlayer>;
}

/// A set of indistinguishable nodes compatible with the player's information, up to any abstraction. Intuitively, this is the support of the distribution over information unknown to the player whose turn to act.
pub(crate) trait Info<'t> {
    // required
    fn roots(&'t self) -> &Vec<&'t Self::INode>;

    // provided
    fn endpoints(&'t self) -> Vec<&'t Self::INode> {
        self.roots()
            .iter()
            .map(|node| node.descendants())
            .flatten()
            .collect()
    }
    fn available(&'t self) -> &Vec<&'t Self::IAction> {
        self.roots().iter().next().unwrap().available()
    }

    type IPlayer: Player;
    type IAction: Action<APlayer = Self::IPlayer>;
    type INode: Node<'t, NAction = Self::IAction> + Node<'t, NPlayer = Self::IPlayer>;
}

/// The owner all the Nodes, Actions, and Players in the context of a Solution. It also constrains the lifetime of references returned by its owned types. A vanilla implementation should build the full tree for small games. Monte Carlo implementations may sample paths conditional on given Profile, Solver, or other constraints. The only contract is that the Tree must be able to partition decision nodes into Info sets.
pub(crate) trait Tree<'t> {
    // required
    fn infos(&'t self) -> &Vec<Self::TInfo>;

    type TPlayer: Player;
    type TAction: Action<APlayer = Self::TPlayer>;
    type TNode: Node<'t, NAction = Self::TAction> + Node<'t, NPlayer = Self::TPlayer>;
    type TInfo: Info<'t>
        + Info<'t, INode = Self::TNode>
        + Info<'t, IAction = Self::TAction>
        + Info<'t, IPlayer = Self::TPlayer>;
}

/// A policy (P: node -> prob) is a distribution over A(Ii). Easily implemented as a HashMap<Aaction, Probability>.
pub(crate) trait Policy {
    // required
    fn weights(&self, action: &Self::PAction) -> Probability;

    type PAction: Action;
}

/// A strategy (σ: player -> policy) is a function that assigns a policy to each h ∈ H, and therefore Ii ∈ Ii. Easily implemented as a HashMap<Info, Policy>.
pub(crate) trait Strategy<'t> {
    // required
    fn policy(&'t self, node: &'t Self::SNode) -> &'t Self::SPolicy;

    type SPlayer: Player;
    type SAction: Action<APlayer = Self::SPlayer>;
    type SPolicy: Policy<PAction = Self::SAction>;
    type SNode: Node<'t, NAction = Self::SAction> + Node<'t, NPlayer = Self::SPlayer>;
}

/// A profile σ consists of a strategy for each player, σ1,σ2,..., equivalently a matrix indexed by (player, action) or (i,a) ∈ N × A
pub(crate) trait Profile<'t> {
    // required
    fn strategy(&'t self, player: &'t Self::PPlayer) -> &'t Self::PStrategy;

    // provided
    // utility calculations
    fn cfactual_value(&'t self, root: &'t Self::PNode, action: &'t Self::PAction) -> Utility {
        self.cfactual_reach(root)
            * root //                                       suppose you're here on purpose, counterfactually
                .follow(action) //                          suppose you're here on purpose, counterfactually
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn expected_value(&'t self, root: &'t Self::PNode) -> Utility {
        self.expected_reach(root)
            * root
                .descendants() //                           O(depth) recursive downtree
                .iter() //                                  duplicated calculation
                .map(|leaf| self.relative_value(root, leaf))
                .sum::<Utility>()
    }
    fn relative_value(&'t self, root: &'t Self::PNode, leaf: &'t Self::PNode) -> Utility {
        leaf.utility(root.chooser())
            * self.relative_reach(root, leaf)
            * self.sampling_reach(root, leaf)
    }
    // probability calculations
    fn weight(&'t self, node: &'t Self::PNode, action: &'t Self::PAction) -> Probability {
        self.strategy(node.chooser()).policy(node).weights(action)
    }
    fn cfactual_reach(&'t self, node: &'t Self::PNode) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.cfactual_reach(parent)
                    * if node.chooser() == parent.chooser() {
                        1.0
                    } else {
                        self.weight(parent, node.precedent().unwrap())
                    }
            }
        }
    }
    fn expected_reach(&'t self, node: &'t Self::PNode) -> Probability {
        match node.parent() {
            None => 1.0,
            Some(parent) => {
                self.expected_reach(parent) * self.weight(parent, node.precedent().unwrap())
            }
        }
    }
    fn relative_reach(&'t self, root: &'t Self::PNode, leaf: &'t Self::PNode) -> Probability {
        //? gotta optimize out integration over shared ancestors that cancels out in this division. Node: Eq? Hash?
        self.expected_reach(leaf) / self.expected_reach(root)
    }
    fn sampling_reach(&'t self, _oot: &'t Self::PNode, _eaf: &'t Self::PNode) -> Probability {
        1.0
    }

    type PPlayer: Player;
    type PAction: Action<APlayer = Self::PPlayer>;
    type PPolicy: Policy<PAction = Self::PAction>;
    type PNode: Node<'t, NAction = Self::PAction> + Node<'t, NPlayer = Self::PPlayer>;
    type PInfo: Info<'t>
        + Info<'t, INode = Self::PNode>
        + Info<'t, IAction = Self::PAction>
        + Info<'t, IPlayer = Self::PPlayer>;
    type PStrategy: Strategy<'t>
        + Strategy<'t, SNode = Self::PNode>
        + Strategy<'t, SPolicy = Self::PPolicy>
        + Strategy<'t, SPlayer = Self::PPlayer>
        + Strategy<'t, SAction = Self::PAction>;
}

/// A Solver will take a Profile and a Tree and iteratively consume/replace a new Profile on each iteration.
pub(crate) trait Solver<'t>: Iterator {
    // required
    fn traverser(&'t self) -> &'t Self::SPlayer; //? struct lookup
    fn tree(&'t self) -> &'t Self::STree; //? struct lookup // use Cell for mutable reference in self.update
    fn step(&'t self) -> &'t Self::SStep; //? struct lookup // use Cell for mutable reference in self.update
    fn num_steps(&'t self) -> usize; //? struct lookup
    fn max_steps(&'t self) -> usize; //? struct lookup

    // provided
    // (info) -> profile.strategy.policy update
    fn update_policy(&'t self, info: &'t Self::SInfo) -> Self::SPolicy;
    fn update_vector(&'t self, info: &'t Self::SInfo) -> Vec<(Self::SAction, Probability)> {
        info.available()
            .iter()
            .map(|action| **action)
            .zip(self.policy_vector(info).into_iter())
            .collect::<Vec<(Self::SAction, Probability)>>()
    }
    fn policy_vector(&'t self, info: &'t Self::SInfo) -> Vec<Probability> {
        let regrets = self.regret_vector(info);
        let sum = regrets.iter().sum::<Utility>();
        regrets.iter().map(|regret| regret / sum).collect()
    }
    fn regret_vector(&'t self, info: &'t Self::SInfo) -> Vec<Utility> {
        info.available()
            .iter()
            .map(|action| self.next_regret(info, action))
            .map(|regret| regret.max(Utility::MIN_POSITIVE))
            .collect()
    }
    // (info, action) -> regret
    fn gain(&'t self, root: &'t Self::SNode, action: &'t Self::SAction) -> Utility {
        self.step().cfactual_value(root, action) - self.step().expected_value(root)
    }
    fn next_regret(&'t self, info: &'t Self::SInfo, action: &'t Self::SAction) -> Utility {
        self.prev_regret(info, action) + self.curr_regret(info, action) //? Linear CFR weighting
    }
    fn curr_regret(&'t self, info: &'t Self::SInfo, action: &'t Self::SAction) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.gain(root, action))
            .sum::<Utility>()
    }
    fn prev_regret(&'t self, info: &'t Self::SInfo, action: &'t Self::SAction) -> Utility; //? struct lookup

    type SPlayer: Player;
    type SAction: Action<APlayer = Self::SPlayer>;
    type SPolicy: Policy<PAction = Self::SAction>;
    type SNode: Node<'t, NAction = Self::SAction> + Node<'t, NPlayer = Self::SPlayer>;
    type SInfo: Info<'t>
        + Info<'t, INode = Self::SNode>
        + Info<'t, IAction = Self::SAction>
        + Info<'t, IPlayer = Self::SPlayer>;
    type SStrategy: Strategy<'t>
        + Strategy<'t, SNode = Self::SNode>
        + Strategy<'t, SAction = Self::SAction>
        + Strategy<'t, SPlayer = Self::SPlayer>
        + Strategy<'t, SPolicy = Self::SPolicy>;
    type STree: Tree<'t>
        + Tree<'t, TInfo = Self::SInfo>
        + Tree<'t, TNode = Self::SNode>
        + Tree<'t, TAction = Self::SAction>
        + Tree<'t, TPlayer = Self::SPlayer>;
    type SStep: Profile<'t>
        + Profile<'t, PInfo = Self::SInfo>
        + Profile<'t, PStrategy = Self::SStrategy>
        + Profile<'t, PNode = Self::SNode>
        + Profile<'t, PAction = Self::SAction>
        + Profile<'t, PPolicy = Self::SPolicy>
        + Profile<'t, PPlayer = Self::SPlayer>;
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
