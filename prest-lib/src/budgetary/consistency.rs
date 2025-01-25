use std::result;
use std::fmt;
use std::io::Write;
use std::iter::FromIterator;
use std::collections::{HashSet,BTreeMap};

use crate::budgetary::Subject;
use crate::matrix::{Matrix,dot_product};
use crate::codec::{self,Encode};
use crate::common::Log;
use crate::set_cover;

#[derive(Debug, Clone)]
pub enum Error {
}

impl Encode for Error {
    fn encode<W : Write>(&self, _f : &mut W) -> codec::Result<()> {
        match *self { }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _f : &mut fmt::Formatter) -> fmt::Result {
        match *self { }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub type Request = Subject;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violations {
    garp : usize,
    sarp : usize,
}

impl Encode for Violations {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (self.garp, self.sarp).encode(f)
    }
}

#[derive(Debug, Clone)]
pub struct BoundEstimate<T> {
    lower : T,
    upper : T,
}

impl<T : Encode> Encode for BoundEstimate<T> {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.lower, &self.upper).encode(f)
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    name : String,
    violations : Vec<(usize, Violations)>, // (cycle_length, counts)
    warp_strict : usize,
    warp_nonstrict : usize,
    hm_garp : BoundEstimate<usize>,
    hm_sarp : BoundEstimate<usize>,
    hm_warp_strict : BoundEstimate<usize>,
    hm_warp_nonstrict : BoundEstimate<usize>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.name,
         &self.violations,
         &self.warp_strict,
         &self.warp_nonstrict,
         &self.hm_garp, &self.hm_sarp,
         &self.hm_warp_strict,
         &self.hm_warp_nonstrict,
        ).encode(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Vertex {
    pub number : usize
}

impl Vertex {
    fn all(n : usize) -> impl Iterator<Item=Vertex> {
        (0..n).map(|number| Vertex{number})
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Cycle {
    // always min-rotated
    pub vertices : Vec<Vertex>
}

impl Cycle {
    fn new(vertices : &[Vertex]) -> Cycle {
        Cycle {
            vertices: rotate_min(vertices),
        }
    }

    fn len(&self) -> usize {
        self.vertices.len()
    }

    fn edges(&self) -> Edges {
        Edges {
            last: self.vertices[self.vertices.len()-1],
            upcoming: &self.vertices[..],
        }
    }
}

struct Edges<'a> {
    last : Vertex,
    upcoming : &'a [Vertex],
}

impl<'a> Iterator for Edges<'a> {
    type Item = (Vertex, Vertex);

    fn next(&mut self) -> Option<(Vertex, Vertex)> {
        if self.upcoming.is_empty() {
            None
        } else {
            let next = self.upcoming[0];
            let edge = (self.last, next);

            self.last = next;
            self.upcoming = &self.upcoming[1..];

            Some(edge)
        }
    }
}
fn rotate_min<T : Ord+Clone>(xs : &[T]) -> Vec<T> {
    if xs.is_empty() {
        return Vec::new();
    }
    // in the code below, we can assume that xs is non-empty

    let minimum = xs.iter().min().unwrap().clone();  // there's at least one
    let min_idx = find(minimum, xs).unwrap();  // we know it's there

    let mut result = Vec::with_capacity(xs.len());
    result.extend_from_slice(&xs[min_idx..]);
    result.extend_from_slice(&xs[0..min_idx]);

    result
}

fn find<T : PartialEq>(x : T, xs : &[T]) -> Option<usize> {
    for (index, y) in xs.iter().enumerate() {
        if x == *y {
            return Some(index);
        }
    }

    None
}

fn find_cycles_from(
    untouched : &mut HashSet<Vertex>,
    edges : &Matrix<bool>,
    history : &mut Vec<Vertex>,
    root : Vertex
) -> HashSet<Cycle> {
    //println!("find_cycles_from {}, history = {:?}", root, history);

    let mut result = HashSet::new();
    untouched.remove(&root);

    /* There are 3 kinds of neighbours:
     * - untouched: recurse and depth-first-search further
     * - in history (and touched): close loop
     * - touched but not in history: no (new) loop possible
     */

    // traverse all neighbours
    history.push(root);
    for next in Vertex::all(edges.nrows) {
        if !edges.get(root.number, next.number) || next == root {
            // no edge or self-loop
            continue;
        }

        //println!("inspecting edge {:?} -> {}", history, next);

        match find(next, history) { // since history never repeats, find == rfind
            Some(idx) => {
                // we've been there in our own history
                // println!("-> new cycle! {:?}", &history[idx..]);
                result.insert(Cycle::new(&history[idx..]));
            }

            None => {
                // println!("-> someone's been there already, but recursing anyway");
                let mut new_cycles = find_cycles_from(untouched, edges, history, next);
                result.extend(new_cycles.drain());
            }
        }
    }
    history.pop();

    result
}

fn find_cycles(edges : &Matrix<bool>) -> HashSet<Cycle> {
    let mut untouched = HashSet::from_iter(Vertex::all(edges.nrows));
    let mut result = HashSet::new();

    // pop a vertex
    while let Some(&root) = untouched.iter().next() {
        // create history for this run
        let mut history = Vec::new();

        // launch loop search
        result.extend(
            find_cycles_from(&mut untouched, edges, &mut history, root)
        );
    }

    result
}

fn find_intersect(components : &[HashSet<Vertex>]) -> Option<(usize, usize)> {
    for i in 0..components.len() {
        for j in i+1..components.len() {
            if !components[i].is_disjoint(&components[j]) {
                return Some((i, j));
            }
        }
    }

    None
}

fn connected_components(cycles : &[Cycle]) -> Vec<HashSet<Vertex>> {
    let mut components : Vec<HashSet<Vertex>> = cycles.iter().map(
        |c| HashSet::from_iter(c.vertices.iter().cloned())
    ).collect();

    while let Some((i,j)) = find_intersect(&components) {
        let cj = components.swap_remove(j);
        components[i].extend(cj);
    }

    components
}

fn hm_bounds(vertex_count : usize, cycles : &[Cycle]) -> BoundEstimate<usize> {
    let cycles_per_vertex : Vec<HashSet<usize>> = {
        let mut groups = vec![HashSet::new(); vertex_count];
        for (cycle_nr, cycle) in cycles.iter().enumerate() {
            for vertex in &cycle.vertices {
                groups[vertex.number].insert(cycle_nr);
            }
        }
        groups
    };

    BoundEstimate{
        lower: connected_components(cycles).len(),
        upper: set_cover::greedy(&cycles_per_vertex).len(),
    }
}

pub fn run<L : Log>(mut log : L, subject : Subject) -> Result<Response> {
    assert_eq!(subject.prices.nrows, subject.amounts.nrows);
    assert_eq!(subject.prices.ncols, subject.amounts.ncols);

    let n_obs = subject.prices.nrows;

    // (row i, column j) contains p^i x^j
    let px = {
        let mut px = Vec::new();
        for p in subject.prices.iter_rows() {
            px.extend(
                subject.amounts.iter_rows().map(
                    |x| dot_product(p, x)
                )
            );
        }
        Matrix::new(n_obs, n_obs, px)
    };

    let edges_nonstrict = Matrix::from_indices(n_obs, n_obs,
        |i, j| px.get(i, i) >= px.get(i, j)
    );
    let edges_strict = Matrix::from_indices(n_obs, n_obs,
        |i, j| px.get(i, i) > px.get(i, j)
    );
    let edges_neq = Matrix::from_indices(n_obs, n_obs,
        |i, j| subject.amounts.row(i) != subject.amounts.row(j)
    );

    let cycles = find_cycles(&edges_nonstrict);
    log.debug(format!("found {} cycles", cycles.len()));

    let mut warp_strict = 0;
    let mut warp_nonstrict = 0;
    let mut violations = BTreeMap::new();

    let mut garp_cycles = Vec::new();
    let mut sarp_cycles = Vec::new();
    let mut warp_strict_cycles = Vec::new();
    let mut warp_nonstrict_cycles = Vec::new();

    for cycle in cycles.iter() {
        /* this counts multiplicities
        let sarp_count = cycle.edges().filter(
            |(i, j)| edges_neq.get(i.number, j.number)
        ).count();

        let garp_count = cycle.edges().filter(
            |(i, j)| edges_strict.get(i.number, j.number)
        ).count();
        */

        // this counts 1 for a multi-cycle with any multiplicity
        let sarp_count = cycle.edges().any(
            |(i, j)| edges_neq.get(i.number, j.number)
        ) as usize;

        let garp_count = cycle.edges().any(
            |(i, j)| edges_strict.get(i.number, j.number)
        ) as usize;

        // add violations
        {
            let v : &mut Violations = violations.entry(cycle.len()).or_insert(
                Violations{garp: 0, sarp: 0}
            );

            v.garp += garp_count;
            v.sarp += sarp_count;
        }

        if cycle.len() == 2 {
            warp_strict += sarp_count;
            warp_nonstrict += garp_count;

            if garp_count > 0 {
                warp_nonstrict_cycles.push(cycle.clone());
            }

            if sarp_count > 0 {
                warp_strict_cycles.push(cycle.clone());
            }
        }

        if garp_count > 0 {
            garp_cycles.push(cycle.clone());
        }

        if sarp_count > 0 {
            sarp_cycles.push(cycle.clone());
        }
    }

    let hm_garp = hm_bounds(n_obs, &garp_cycles);
    let hm_sarp = hm_bounds(n_obs, &sarp_cycles);
    let hm_warp_strict = hm_bounds(n_obs, &warp_strict_cycles);
    let hm_warp_nonstrict = hm_bounds(n_obs, &warp_nonstrict_cycles);

    Ok(Response{
        name: subject.name,
        violations: Vec::from_iter(violations),
        warp_strict,
        warp_nonstrict,
        hm_garp,
        hm_sarp,
        hm_warp_strict,
        hm_warp_nonstrict,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::LogLevel;

    struct DummyLogger;

    impl Log for DummyLogger {
        fn log(&mut self, _level : LogLevel, _message : String) {}
        fn progress(&mut self, _position : u32) {}
    }

    #[test]
    fn basic() {
        let subject = Subject{
            name: String::from("subj01"),
            prices: Matrix::from_slice(4, 2, &[
                1.0, 2.0,
                2.0, 2.0,
                3.0, 4.0,
                4.0, 5.0,
            ]),
            amounts: Matrix::from_slice(4, 2, &[
                1.0, 2.0,
                2.0, 2.0,
                2.0, 1.0,
                3.0, 1.0,
            ]),
        };

        let resp = super::run(DummyLogger, subject).unwrap();

        assert_eq!(resp.name, "subj01");
        assert_eq!(resp.warp_strict, 1);
        assert_eq!(resp.warp_nonstrict, 1);
        assert_eq!(resp.violations, &[
            (2, Violations{ garp: 1, sarp: 1 }),
        ]);
        assert_eq!(resp.hm_garp.lower, 1);
        assert_eq!(resp.hm_garp.upper, 1);
        assert_eq!(resp.hm_sarp.lower, 1);
        assert_eq!(resp.hm_sarp.upper, 1);
        assert_eq!(resp.hm_warp_strict.lower, 1);
        assert_eq!(resp.hm_warp_strict.upper, 1);
        assert_eq!(resp.hm_warp_nonstrict.lower, 1);
        assert_eq!(resp.hm_warp_nonstrict.upper, 1);
    }

    #[test]
    fn basic2() {
        // data from yorgos's e-mail
        let subject = Subject{
            name: String::from("subj01"),
            prices: Matrix::from_slice(3, 3, &[
                2.0, 2.0, 3.0,
                5.0, 5.0, 1.0,
                3.0, 4.0, 2.0,
            ]),
            amounts: Matrix::from_slice(3, 3, &[
                10.0, 20.0,  5.0,
                 7.0,  2.0, 10.0,
                 8.0, 12.0,  7.0,
            ]),
        };

        let resp = super::run(DummyLogger, subject).unwrap();

        assert_eq!(resp.name, "subj01");
        assert_eq!(resp.violations, &[
        ]);
        assert_eq!(resp.warp_strict, 0);
        assert_eq!(resp.warp_nonstrict, 0);
        assert_eq!(resp.hm_garp.lower, 0);
        assert_eq!(resp.hm_garp.upper, 0);
        assert_eq!(resp.hm_sarp.lower, 0);
        assert_eq!(resp.hm_sarp.upper, 0);
        assert_eq!(resp.hm_warp_strict.lower, 0);
        assert_eq!(resp.hm_warp_strict.upper, 0);
        assert_eq!(resp.hm_warp_nonstrict.lower, 0);
        assert_eq!(resp.hm_warp_nonstrict.upper, 0);
    }
}
