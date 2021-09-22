use super::parser::*;

use std::collections::HashMap;

// dominance computation
// Lengauer-Tarjan algorithm

struct DomUnionFind {
    pub pars: Vec<usize>,
    pub mn: Vec<usize>,
}

impl DomUnionFind {
    fn new() -> Self {
        Self {
            pars: vec![],
            mn: vec![],
        }
    }
    fn init(&mut self, n: usize) {
        for i in 0..n {
            self.pars.push(i);
            self.mn.push(i);
        }
    }
    fn find(&mut self, v: usize, sdom: &Vec<usize>) -> usize {
        if self.pars[v] == v {
            return v;
        }
        let r = self.find(self.pars[v], sdom);
        if sdom[self.mn[v]] > sdom[self.mn[self.pars[v]]] {
            self.mn[v] = self.mn[self.pars[v]];
        }
        self.pars[v] = r;
        return r;
    }
    fn eval(&mut self, v: usize, sdom: &Vec<usize>) -> usize {
        self.find(v, sdom);
        self.mn[v]
    }
    fn link(&mut self, u: usize, d: usize) {
        self.pars[u] = d;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlFlowGraph {
    pub graph: Vec<Vec<usize>>,
    pub rgraph: Vec<Vec<usize>>,
    pub vertex: Vec<usize>,
    pub weight: usize,
    pub parents: Vec<usize>,
}

impl ControlFlowGraph {
    fn new(graph: Vec<Vec<usize>>, rgraph: Vec<Vec<usize>>) -> Self {
        let lg = graph.len();
        Self {
            graph,
            rgraph,
            vertex: vec![0; lg],
            weight: 0,
            parents: vec![0; lg],
        }
    }
    fn dfs(&mut self, sdom: &mut Vec<usize>, v: usize) {
        sdom[v] = self.weight;
        self.vertex[self.weight] = v;
        self.weight += 1;
        for i in 0..self.graph[v].len() {
            let u = self.graph[v][i];
            if sdom[u] == std::usize::MAX {
                self.parents[u] = v;
                self.dfs(sdom, u);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DominatorsTree {
    pub sdom: Vec<usize>,
    pub idom: Vec<usize>,
    pub colu: Vec<usize>,
    pub bucket: Vec<Vec<usize>>,
    pub tree: Vec<Vec<usize>>,
}

impl DominatorsTree {
    fn new(n: usize) -> Self {
        Self {
            sdom: vec![std::usize::MAX; n],
            idom: vec![std::usize::MAX; n],
            colu: vec![0; n],
            bucket: vec![vec![]; n],
            tree: vec![vec![]; n],
        }
    }
    fn generate_tree(&mut self, cfg: &mut ControlFlowGraph) {
        let n = cfg.graph.len();
        let mut uf = DomUnionFind::new();
        uf.init(n);
        cfg.dfs(&mut self.sdom, 0);

        for i in (1..n).rev() {
            let v = cfg.vertex[i];
            for u in &cfg.rgraph[v] {
                let s = uf.eval(*u, &self.sdom);
                self.sdom[v] = std::cmp::min(self.sdom[v], self.sdom[s]);
            }
            self.bucket[cfg.vertex[self.sdom[v]]].push(v);
            for t in &self.bucket[cfg.parents[v]] {
                self.colu[*t] = uf.eval(*t, &self.sdom);
            }
            self.bucket[cfg.parents[v]].clear();
            uf.link(v, cfg.parents[v]);
        }

        for i in 1..n {
            let v = cfg.vertex[i];
            let u = self.colu[v];
            self.idom[v] = if self.sdom[v] == self.sdom[u] {
                self.sdom[v]
            } else {
                self.idom[u]
            };
        }

        // root == 0
        for i in 1..n {
            self.idom[i] = cfg.vertex[self.idom[i]];
        }
    }
    fn make_bb_domtree(&mut self, bbs: &mut Vec<SsaBlock>, n: usize) -> ControlFlowGraph {
        let mut bbids = HashMap::new();

        for i in 0..n {
            bbids.insert(bbs[i].lb, bbs[i].id);
        }

        // make graph and rgraph
        let mut graph = vec![vec![]; n];
        let mut rgraph = vec![vec![]; n];
        for i in 0..n {
            for translb in &bbs[i].transbbs {
                let transid = bbids.get(translb).unwrap_or_else(|| {
                    panic!("cannot find {} in bbids in make_bb_domtree", translb)
                });
                graph[bbs[i].id].push(*transid);
                rgraph[*transid].push(bbs[i].id);
            }
        }

        // control flow graph
        let mut cfg = ControlFlowGraph::new(graph, rgraph);

        // generate dominators tree for basic block graph
        self.generate_tree(&mut cfg);

        // save idom and tree structure
        for bb in bbs {
            bb.idom = self.idom[bb.id];
            if bb.idom != std::usize::MAX {
                self.tree[bb.idom].push(bb.id);
            }
        }

        cfg
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DominatorFrontier {
    pub domf: Vec<Vec<usize>>,
}

impl DominatorFrontier {
    fn new(n: usize) -> Self {
        Self {
            domf: vec![vec![std::usize::MAX; 1]; n],
        }
    }
    fn compute(
        &mut self,
        cfg: &ControlFlowGraph,
        domt: &DominatorsTree,
        bbi: usize,
        bbs: &mut Vec<SsaBlock>,
    ) {
        let mut df = vec![];

        // Y for { Y in succ(X) and IDOM(Y) != X }
        for succ_y in &cfg.graph[bbi] {
            if domt.idom[*succ_y] != bbi {
                df.push(*succ_y);
            }
        }

        // Y for (for all Z in child(X), { Y in DF(Z) and IDOM(Y) != X })
        for child_x in &domt.tree[bbi] {
            if let Some(&std::usize::MAX) = self.domf[*child_x].first() {
                self.compute(cfg, domt, *child_x, bbs);
            }
            for y in &self.domf[*child_x] {
                if domt.idom[*y] != bbi {
                    df.push(*y);
                }
            }
        }

        bbs[bbi].df = df.clone();
        self.domf[bbi] = df;
    }
}

pub fn dominators(spg: &mut SsaProgram) {
    // compute dominators structure
    for func in &mut spg.funcs {
        let n = func.bls.len();
        let mut domt = DominatorsTree::new(n);
        let cfg = domt.make_bb_domtree(&mut func.bls, n);
        let mut domf = DominatorFrontier::new(n);
        domf.compute(&cfg, &domt, 0, &mut func.bls);
        func.cfg = Some(Box::new(cfg));
    }
}
