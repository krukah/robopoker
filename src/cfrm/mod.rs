pub mod cfr;

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
