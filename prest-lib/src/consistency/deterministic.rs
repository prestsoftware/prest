use std::result;
use std::fmt::{self, Display};
use std::collections::{BTreeMap,HashSet,HashMap};
use std::collections::btree_map::Entry;
use num::{zero, one};
use std::io::{Read,Write};

use crate::alt::Alt;
use crate::alt_set::AltSet;
use num_traits::identities::Zero;
use num_bigint::BigUint;
use crate::common::{ChoiceRow,Subject};
use crate::codec::{self,Encode,Decode,Packed};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash, Copy)]
struct Edge(pub u32);  // choice row index

#[derive(PartialEq, Eq, Debug)]
struct Multigraph {
    vertices : u32,
    edges : Vec<Vec<Edge>>,  // NxN matrix of edges between (i,j)
}

impl Display for Multigraph {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "     ")?;
        for j in Alt::all(self.vertices) {
            write!(f, "(,{}) ", j)?;
        }
        writeln!(f)?;

        for i in Alt::all(self.vertices) {
            write!(f, "({},) ", i)?;
            for j in Alt::all(self.vertices) {
                write!(f, "{:>4} ", self.edges(i, j).len())?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Multigraph {
    fn new(vertices : u32) -> Multigraph {
        Multigraph {
            vertices,
            edges: vec![Vec::new(); (vertices*vertices) as usize],
        }
    }

    // edge(i, j) means x_i ≤ x_j
    // i.e. there's an arrow x_i -> x_j
    fn has_edge(&self, i : Alt, j : Alt) -> bool {
        !self.edges(i,j).is_empty()
    }

    fn edges(&self, Alt(i) : Alt, Alt(j) : Alt) -> &[Edge] {
        &self.edges[(self.vertices*i + j) as usize]
    }

    fn edges_mut(&mut self, Alt(i) : Alt, Alt(j) : Alt) -> &mut Vec<Edge> {
        &mut self.edges[(self.vertices*i + j) as usize]
    }
}

fn add_choice_row(strict : &mut Multigraph, non_strict : &mut Multigraph, cr : &ChoiceRow, edge : &Edge) {
    let choice = cr.choice.view();
    let menu = cr.menu.view();

    for i in menu.iter() {
        for j in choice.iter() {
            non_strict.edges_mut(i, j).push(*edge);

            if !choice.contains(i) {
                strict.edges_mut(i, j).push(*edge);
            }
        }
    }
}

fn build_graphs(alt_count : u32, choices : &[ChoiceRow]) -> (Multigraph, Multigraph) {
    let mut strict = Multigraph::new(alt_count);
    let mut non_strict = Multigraph::new(alt_count);

    for (idx, cr) in choices.iter().enumerate() {
        add_choice_row(&mut strict, &mut non_strict, cr, &Edge(idx as u32));
    }

    (strict, non_strict)
}

trait HasSize {
    fn size(&self) -> u32;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
struct Cycle {
    // always min-rotated
    vertices : Vec<Alt>,
}

impl Cycle {
    fn new(vertices : &[Alt]) -> Cycle {
        Cycle {
            vertices: rotate_min(vertices),
        }
    }

    fn edges(&self) -> Edges {
        Edges {
            last: self.vertices[self.vertices.len()-1],
            upcoming: &self.vertices[..],
        }
    }

    fn multiplicity_in(&self, g : &Multigraph) -> BigUint {
        let mut result = one();

        for (u, v) in self.edges() {
            result *= g.edges(u, v).len();
        }

        result
    }

    fn has_edge_in(&self, g : &Multigraph) -> bool {
        self.edges().any(|(u,v)| g.has_edge(u, v))
    }

    fn garp_multiplicity_in(&self, strict : &Multigraph, non_strict : &Multigraph) -> BigUint {
        fn multiplicity_from(
            strict : &Multigraph,
            non_strict : &Multigraph,
            got_strict_edge : bool,
            edges : &[(Alt, Alt)]
        ) -> BigUint {
            if edges.is_empty() {
                if got_strict_edge {
                    return one();
                } else {
                    return zero();
                }
            }

            let (u, v) = edges[0];
            assert!(non_strict.has_edge(u, v));

            if strict.has_edge(u, v) {
                // we may choose a strict edge here (or not)
                let strict_edges = strict.edges(u, v).len();
                let remaining_edges = non_strict.edges(u, v).len() - strict_edges;  // non_strict includes strict, we want the rest

                strict_edges * multiplicity_from(strict, non_strict, true, &edges[1..])
                + remaining_edges * multiplicity_from(strict, non_strict, got_strict_edge, &edges[1..])
            } else {
                // we must choose a non-strict edge here
                //
                // this branch is a special case of the previous one
                // but is more efficient because it does not branch execution
                non_strict.edges(u, v).len() * multiplicity_from(strict, non_strict, got_strict_edge, &edges[1..])
            }
        }

        let edges : Vec<_> = self.edges().collect();
        multiplicity_from(strict, non_strict, false, &edges)
    }

    fn len(&self) -> u32 {
        self.vertices.len() as u32
    }
}

impl HasSize for Cycle {
    fn size(&self) -> u32 {
        self.len()
    }
}

struct Edges<'a> {
    last : Alt,
    upcoming : &'a [Alt],
}

impl<'a> Iterator for Edges<'a> {
    type Item = (Alt, Alt);

    fn next(&mut self) -> Option<(Alt, Alt)> {
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
    untouched : &mut HashSet<Alt>,
    g : &Multigraph,
    history : &mut Vec<Alt>,
    root : Alt
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
    for next in Alt::all(g.vertices) {
        if !g.has_edge(root, next) || next == root {
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
                let mut new_cycles = find_cycles_from(untouched, g, history, next);
                result.extend(new_cycles.drain());
            }
        }
    }
    history.pop();

    result
}

fn find_cycles(g : &Multigraph) -> HashSet<Cycle> {
    let mut untouched : HashSet<Alt> = Alt::all(g.vertices).collect();
    let mut result = HashSet::new();

    // pop a vertex
    while let Some(&root) = untouched.iter().next() {
        // create history for this run
        let mut history = Vec::new();

        // launch loop search
        let mut new_cycles = find_cycles_from(&mut untouched, g, &mut history, root);
        result.extend(new_cycles.drain());
    }

    result
}

#[derive(Debug)]
pub enum Error {
    TooManyTuples,
    RepeatedMenus,
}

impl Encode for Error {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        match self {
            Error::TooManyTuples => 0u8.encode(f),
            Error::RepeatedMenus => 1u8.encode(f)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::TooManyTuples => write!(f, "too many tuples"),
            Error::RepeatedMenus => write!(f, "dataset contains repeated menus"),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Request {
    subject : Packed<Subject>,
    allow_repeated_menus : bool,  // for testing
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            subject: Decode::decode(f)?,
            allow_repeated_menus: false,
        })
    }
}

// scores for one particular cycle length
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Row {
    cycle_length : u32,
    garp : BigUint,
    sarp : BigUint,
    garp_binary_menus : BigUint,
    sarp_binary_menus : BigUint,
    binary_intransitivities : BigUint,
}

impl Row {
    fn new(cycle_length : u32) -> Row {
        Row {
            cycle_length,
            garp: zero(),
            sarp: zero(),
            garp_binary_menus: zero(),
            sarp_binary_menus: zero(),
            binary_intransitivities: zero(),
        }
    }
}

impl Encode for Row {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (
            &self.cycle_length,
            &self.garp,
            &self.sarp,
            &self.garp_binary_menus,
            &self.sarp_binary_menus,
            &self.binary_intransitivities,
        ).encode(f)
    }
}

pub struct Response {
    subject_name : String,
    rows : Vec<Row>,

    warp_pairs : u32,
    warp : BigUint,

    contraction_consistency_pairs : u32,
    contraction_consistency_all : u32,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (
            &self.subject_name,
            &self.rows,
            &self.warp_pairs,
            &self.warp,
            &self.contraction_consistency_pairs,
            &self.contraction_consistency_all,
        ).encode(f)
    }
}

trait MakeEmpty {
    fn make_empty(size : u32) -> Self;
}

impl MakeEmpty for Row {
    fn make_empty(size : u32) -> Self {
        Row::new(size)
    }
}

fn summarise<I, F, T, R>(rows : &mut BTreeMap<u32, R>, items : I, add_item : F)
    where
        I : IntoIterator<Item=T>,
        F : Fn(&mut R, T),
        T : HasSize,
        R : MakeEmpty
{
    for item in items {
        // not using or_insert() because we don't want to allocate
        // a zero Row every time, whether we need it or not
        let size = item.size();
        match rows.entry(size) {
            Entry::Vacant(e) => {
                let row = e.insert(R::make_empty(size));
                add_item(row, item);
            }

            Entry::Occupied(mut e) => {
                let row = e.get_mut();
                add_item(row, item);
            }
        }
    }
}

fn compute_warp_pairs(alt_count : u32, g_strict : &Multigraph, g_non_strict : &Multigraph) -> u32 {
    let mut menu_pairs = HashSet::new();

    for v in Alt::all(alt_count) {
        for w in Alt::all(alt_count) {
            if v == w {
                continue;
            }

            // at least one edge strict
            // count unique menu pairs
            for &Edge(cr_forth) in g_strict.edges(v, w) {
                for &Edge(cr_back) in g_non_strict.edges(w, v) {
                    if cr_forth < cr_back {
                        menu_pairs.insert((cr_forth, cr_back));
                    } else {
                        menu_pairs.insert((cr_back, cr_forth));
                    }
                }
            }
        }
    }

    menu_pairs.len() as u32
}

fn contraction_consistency(choices : &[ChoiceRow]) -> (u32, u32) {
    let mut all = 0;
    let mut pairs = 0;

    #[allow(non_snake_case)]
    for cr_A in choices {
        for cr_B in choices {
            if !cr_A.menu.view().is_strict_subset_of(cr_B.menu.view()) {
                continue;
            }

            let mut violations = 0;
            for a in cr_B.choice.view() {
                if cr_A.menu.view().contains(a) && !cr_A.choice.view().contains(a) {
                    violations += 1;
                }
            }

            all += violations;
            pairs += (violations > 0) as u32;
        }
    }

    (all, pairs)
}

struct BinaryIntransitivity {
    length : u32,
    multiplicity : BigUint,
}

impl HasSize for BinaryIntransitivity {
    fn size(&self) -> u32 {
        self.length
    }
}

fn binary_intransitivities(alt_count : u32, g : &Multigraph, choices : &[ChoiceRow]) -> Vec<BinaryIntransitivity> {
    fn paths_from(prefix : &[Alt], avail : &mut AltSet) -> Vec<Vec<Alt>> {
        let mut all_paths = Vec::new();
        let mut path = Vec::from(prefix);

        for x in avail.clone().view() {
            let only_x = AltSet::singleton(x);
            path.push(x);
            *avail -= only_x.view();

            all_paths.extend(
                paths_from(&path, avail)
            );

            *avail |= only_x.view();
            path.pop();
        }

        all_paths.push(path);
        all_paths
    }

    let candidates : HashMap<(Alt, Alt), u32> = {
        let mut candidates = HashMap::new();
        for cr in choices {
            // consider only binary menus
            if cr.menu.size() != 2 {
                continue;
            }

            for x in cr.menu.view() {
                if cr.choice.view().contains(x) {
                    // can't cause an intransitivity
                    continue;
                }

                // so here we know that x \in A, but x \notin C(A)

                for y in cr.menu.view() {
                    // if we have a path x > ... > y
                    // then observation `cr` would cause an intransitivity
                    // because the subject failed to choose x here
                    // hence it's a candidate
                    *candidates.entry((x,y)).or_insert(0) += 1;
                }
            }
        }
        candidates
    };

    let mut paths = Vec::new();

    let mut avail = AltSet::from_iter(
        Alt::all(alt_count)
    );
    for path in paths_from(&[], &mut avail) {
        // we're interested only in paths of 1+ edges
        if path.len() < 2 {
            continue;
        }

        let mut multiplicity : BigUint = num::one();
        for (&x, &y) in path.iter().zip(path.iter().skip(1)) {
            multiplicity *= g.edges(y, x).len();  // traverse edges y≤x
        }

        if multiplicity.is_zero() {
            continue;
        }

        // check if this is a candidate for intransitivity
        let (u, v) = (*path.first().unwrap(), *path.last().unwrap());
        if let Some(cnt) = candidates.get(&(u,v)) {
            multiplicity *= *cnt;
        } else {
            continue;
        }

        /*
        println!("intrans: {} × {:?}", multiplicity, path);
        println!("  {},{} -> {:?}", u, v, candidates.get(&(u,v)));
        for (&x, &y) in path.iter().zip(path.iter().skip(1)) {
            println!("  {:?}", g.edges(y,x));
        }
        */
        paths.push(BinaryIntransitivity {
            length: path.len() as u32,  // #vertices to match GARP cycles, etc.
            multiplicity,
        });
    }

    paths
}

fn has_repeated_menus(choices : &[ChoiceRow]) -> bool {
    let mut seen = HashSet::new();

    for cr in choices {
        if seen.contains(&cr.menu) {
            return true;
        }

        seen.insert(&cr.menu);
    }

    false
}

pub fn run(request : &Request) -> Result<Response> {
    let ref subject = request.subject.unpack();
    let alt_count = subject.alternatives.len() as u32;
    let choices = &subject.choices;

    if !request.allow_repeated_menus && has_repeated_menus(choices) {
        return Err(Error::RepeatedMenus);
    }

    let (g_strict, g_non_strict) = build_graphs(alt_count, choices);
    let cycles_non_strict = find_cycles(&g_non_strict);  // will be used for GARP
    let cycles_strict = find_cycles(&g_strict);
    let mut rows = BTreeMap::new();

    // SARP (includes 2-cycles)
    summarise(
        &mut rows,
        cycles_strict,
        |r : &mut Row, c| r.sarp += c.multiplicity_in(&g_strict)
    );

    let warp = cycles_non_strict.iter().filter(|c| c.len() == 2).map(
        |c| c.garp_multiplicity_in(&g_strict, &g_non_strict)
    ).sum();

    // GARP (includes 2-cycles)
    summarise(
        &mut rows,
        cycles_non_strict,
        |r, c| r.garp += c.garp_multiplicity_in(&g_strict, &g_non_strict)
    );

    let warp_pairs = compute_warp_pairs(alt_count, &g_strict, &g_non_strict);

    let (
        contraction_consistency_all,
        contraction_consistency_pairs,
    ) = contraction_consistency(choices);

    let choices_binary = Vec::from_iter(
        choices.iter().filter(|c| c.menu.size() == 2).cloned()
    );

    let (g_strict_binary, g_non_strict_binary) = build_graphs(alt_count, &choices_binary);
    let cycles_strict_binary = find_cycles(&g_strict_binary);
    let cycles_non_strict_binary = find_cycles(&g_non_strict_binary);

    // garp_binary
    summarise(
        &mut rows,
        cycles_non_strict_binary,
        |r, c| r.garp_binary_menus += c.garp_multiplicity_in(&g_strict_binary, &g_non_strict_binary)
    );

    // sarp_binary
    summarise(
        &mut rows,
        cycles_strict_binary,
        |r, c| r.sarp_binary_menus += c.multiplicity_in(&g_strict_binary)
    );

    // binary intransitivities
    summarise(
        &mut rows,
        binary_intransitivities(alt_count, &g_non_strict_binary, choices),
        |r, bi| r.binary_intransitivities += bi.multiplicity,
    );

    Ok(Response {
        subject_name: subject.name.clone(),
        rows: rows.into_iter().map(|(_l,r)| r).collect(),

        warp,
        warp_pairs,

        contraction_consistency_all,
        contraction_consistency_pairs,
    })
}

pub fn sort<I : IntoIterator<Item=T>, T : Ord>(items : I) -> Vec<T> {
    let mut result = Vec::from_iter(items);
    result.sort();
    result
}

pub mod tuple_intrans {
    use super::{Request,Result,Error,Cycle,Multigraph,Edge,HasSize,MakeEmpty};
    use super::{build_graphs,find_cycles,summarise};
    use std::collections::{BTreeSet,HashSet,BTreeMap};
    use crate::alt_set::AltSet;
    use crate::alt::Alt;
    use std::hash::Hash;
    use crate::simulation::Menu;
    use crate::codec::{self,Encode};
    use std::io::Write;
    use std::fmt::Debug;

    pub struct RowMenus {
        tuple_size : u32,
        garp_menu_tuples : HashSet<BTreeSet<Menu>>,  // n-tuples of menus constituting an inconsistency
    }

    pub struct RowAlts {
        tuple_size : u32,
        garp_alt_tuples : HashSet<AltSet>,  // n-tuples of alternatives involved in an inconsistency
    }

    impl MakeEmpty for RowMenus {
        fn make_empty(size : u32) -> Self {
            RowMenus{
                tuple_size: size,
                garp_menu_tuples: HashSet::new(),
            }
        }
    }

    impl MakeEmpty for RowAlts {
        fn make_empty(size : u32) -> Self {
            RowAlts{
                tuple_size: size,
                garp_alt_tuples: HashSet::new(),
            }
        }
    }

    impl Encode for RowMenus {
        fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
            (self.tuple_size, &self.garp_menu_tuples).encode(f)
        }
    }

    impl Encode for RowAlts {
        fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
            (self.tuple_size, &self.garp_alt_tuples).encode(f)
        }
    }

    pub struct Response<R> {
        subject_name : String,
        rows : Vec<R>,
    }

    impl<R : Encode> Encode for Response<R> {
        fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
            (&self.subject_name, &self.rows).encode(f)
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    struct ChoiceRows(BTreeSet<u32>);  // indices

    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    struct Alternatives(AltSet);

    trait AddEdge {
        fn empty() -> Self;
        fn add_edge(&self, u : Alt, v : Alt, cr : u32) -> Self;
    }

    impl AddEdge for ChoiceRows {
        fn empty() -> Self {
            ChoiceRows(BTreeSet::new())
        }

        fn add_edge(&self, _u : Alt, _v : Alt, cr : u32) -> Self {
            ChoiceRows({
                let mut ixs = self.0.clone();
                ixs.insert(cr);
                ixs
            })
        }
    }

    impl HasSize for ChoiceRows {
        fn size(&self) -> u32 {
            self.0.len() as u32
        }
    }

    impl AddEdge for Alternatives {
        fn empty() -> Self {
            Alternatives(AltSet::empty())
        }

        fn add_edge(&self, u : Alt, v : Alt, _cr : u32) -> Self {
            Alternatives({
                let mut alts = self.0.clone();
                alts |= &AltSet::from_iter(&[u, v]);
                alts
            })
        }
    }

    impl HasSize for Alternatives {
        fn size(&self) -> u32 {
            self.0.size()
        }
    }

    fn garp_tuples<T>(cycle : &Cycle, g_strict : &Multigraph, g_non_strict : &Multigraph) -> HashSet<T>
        where T : AddEdge + Eq + Hash + Debug
    {
        // non-strict paths
        // they require a strict edge sometime later
        let mut paths_cond = HashSet::new();
        paths_cond.insert(AddEdge::empty());

        // paths that already involve a strict edge so they are final
        let mut paths_uncond = HashSet::new();
        paths_uncond.insert(AddEdge::empty());

        for (u, v) in cycle.edges() {
            let mut paths_uncond_extended = HashSet::new();
            for &Edge(cr) in g_strict.edges(u, v) {
                // adding a strict edge to a cond-path makes it uncond
                paths_uncond_extended.extend(
                    paths_cond.iter().map(|p : &T| p.add_edge(u,v,cr))
                );

                // adding a strict edge to an uncond path is redundant:
                // this edge is also non-strict so it's taken care of
                // by the code below
            }

            let mut paths_cond_extended = HashSet::new();
            for &Edge(cr) in g_non_strict.edges(u, v) {
                // adding a non-strict edge to an uncond path leaves it uncond
                paths_uncond_extended.extend(
                    paths_uncond.iter().map(|p : &T| p.add_edge(u,v,cr))
                );

                // adding a non-strict edge to a cond path leaves it cond
                paths_cond_extended.extend(
                    paths_cond.iter().map(|p : &T| p.add_edge(u,v,cr))
                );
            }

            // TODO: use double buffering instead of constant allocation/deallocation
            paths_cond = paths_cond_extended;
            paths_uncond = paths_uncond_extended;

        }

        // the result are exactly the paths that have a strict edge
        paths_uncond
    }

    pub fn run_menus(request : &Request) -> Result<Response<RowMenus>> {
        let ref subject = request.subject.unpack();
        let alt_count = subject.alternatives.len() as u32;
        let (g_strict, g_non_strict) = build_graphs(alt_count, &subject.choices);
        let cycles_non_strict = find_cycles(&g_non_strict);

        let mut by_length = BTreeMap::new();

        for cycle in cycles_non_strict.iter().filter(|c| c.has_edge_in(&g_strict)) {
            if cycle.len() > 24 {
                return Err(Error::TooManyTuples);
            }

            // collect choice rows for that cycle
            summarise(
                &mut by_length,
                garp_tuples::<ChoiceRows>(cycle, &g_strict, &g_non_strict),
                |r : &mut RowMenus, ChoiceRows(ixs)| {
                    r.garp_menu_tuples.insert(
                        ixs.into_iter().map(
                            |ix| subject.choices[ix as usize].menu.clone()
                        ).collect()
                    );
                }
            );
        }

        Ok(Response{
            subject_name: subject.name.clone(),
            rows: by_length.into_iter().filter_map(
                |(l,r)| if l > 0 { Some(r) } else { None }
            ).collect(),
        })
    }

    pub fn run_alts(request : &Request) -> Result<Response<RowAlts>> {
        let ref subject = request.subject.unpack();
        let alt_count = subject.alternatives.len() as u32;
        let (g_strict, g_non_strict) = build_graphs(alt_count, &subject.choices);
        let cycles_non_strict = find_cycles(&g_non_strict);

        let mut by_length = BTreeMap::new();

        for cycle in cycles_non_strict.iter().filter(|c| c.has_edge_in(&g_strict)) {
            if cycle.len() > 24 {
                return Err(Error::TooManyTuples);
            }

            // collect alternatives for that cycle
            let alts = garp_tuples::<Alternatives>(cycle, &g_strict, &g_non_strict);
            summarise(
                &mut by_length,
                alts,
                |r : &mut RowAlts, Alternatives(alts)| {
                    r.garp_alt_tuples.insert(alts);
                },
            );
        }

        Ok(Response{
            subject_name: subject.name.clone(),
            rows: by_length.into_iter().filter_map(
                |(l,r)| if l > 0 { Some(r) } else { None }
            ).collect(),
        })
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::common::{Subject,ChoiceRow};

        fn testreq(alt_count : u32, choices : Vec<ChoiceRow>) -> Request {
            Request{subject: codec::Packed(Subject{
                name: String::from("subject"),
                alternatives: (0..alt_count).map(|s| s.to_string()).collect(),
                choices,
            }), allow_repeated_menus: true}
        }

        #[test]
        fn tuple_intrans() {
            let choices = choices![
                [0,1] -> [1],
                [1,2] -> [2],
                [2,3] -> [3],
                [3,0] -> [0]
            ];

            let (strict, non_strict) = build_graphs(5, &choices);
            let cycles_non_strict = find_cycles(&non_strict);
            let cycles_strict = find_cycles(&strict);
            assert_eq!(cycles_non_strict.len(), 1);
            assert_eq!(cycles_strict.len(), 1);

            let request = testreq(4, choices);
            let response_menus = super::run_menus(&request).unwrap();
            let response_alts = super::run_alts(&request).unwrap();

            // 3->0, 4->1
            assert_eq!(
                response_menus.rows.iter().map(|r| (r.tuple_size, r.garp_menu_tuples.len())).collect::<Vec<_>>(),
                vec![(4,1)],
            );

            assert_eq!(
                response_alts.rows.iter().map(|r| (r.tuple_size, r.garp_alt_tuples.len())).collect::<Vec<_>>(),
                vec![(4,1)],
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use num::Zero;
    use crate::alt_set::AltSet;
    use std::iter::FromIterator;

    fn testreq(alt_count : u32, choices : Vec<ChoiceRow>) -> Request {
        Request{subject: codec::Packed(Subject{
            name: String::from("subject"),
            alternatives: (0..alt_count).map(|s| s.to_string()).collect(),
            choices,
        }), allow_repeated_menus: true}
    }

    #[test]
    fn strict_cycles() {
        let choices = choices![
            [0,1] -> [0,1],
            [0,2] -> [0],
            [1,2] -> [2]
        ];

        let request = testreq(3, choices);
        let response = super::run(&request).unwrap();

        assert_eq!(response.rows, vec![
            Row{
                cycle_length: 2,
                garp: BigUint::from(0u32),
                sarp: BigUint::from(0u32),
                garp_binary_menus: BigUint::from(0u32),
                sarp_binary_menus: BigUint::from(0u32),
                binary_intransitivities: BigUint::from(0u32),
            },
            Row{
                cycle_length: 3,
                garp: BigUint::from(1u32),
                sarp: BigUint::from(0u32),
                garp_binary_menus: BigUint::from(1u32),
                sarp_binary_menus: BigUint::from(0u32),
                binary_intransitivities: BigUint::from(2u32),
            }
        ]);
    }

    #[test]
    fn warp_pairs_long() {
        let choices = choices![
            [0,1] -> [1],
            [1,2] -> [2],
            [2,3] -> [3],
            [3,0] -> [0]
        ];

        let request = testreq(4, choices);
        let response = super::run(&request).unwrap();

        assert_eq!(response.warp_pairs, 0);
    }

    #[test]
    fn many_cycles() {
        let choices = choices![
            [0,1] -> [0,1],
            [0,2] -> [0],
            [0,3] -> [3],
            [0,4] -> [0],
            [0,5] -> [0,5],
            [1,2] -> [1,2],
            [1,3] -> [1,3],
            [1,4] -> [1,4],
            [1,5] -> [1,5],
            [2,3] -> [2,3],
            [2,4] -> [2],
            [2,5] -> [2,5],
            [3,4] -> [4],
            [3,5] -> [3,5],
            [4,5] -> [4,5]
        ];

        let request = testreq(6, choices);
        let detailed = run(&request).unwrap();

        assert_eq!(
            detailed.rows.into_iter().map(|r| r.garp_binary_menus).sum::<BigUint>(),
            BigUint::from(136u32),
        );
    }

    #[test]
    fn test_build() {
        let choices = choices![
            [0,1] -> [0,1],
            [1,2] -> [2],
            [0,2] -> [0],
            [1,3] -> [3],
            [2,3] -> [2],
            [2,4] -> [4],
            [4,3] -> [3]
        ];
        let (strict, non_strict) = build_graphs(5, &choices);
        assert!(strict.has_edge(Alt(1), Alt(2)));

        let cycles = find_cycles(&non_strict);

        assert_eq!(
            sort(cycles),
            &[
                Cycle::new(&[Alt(0),Alt(1)]),
                Cycle::new(&[Alt(0),Alt(1),Alt(2)]),
                Cycle::new(&[Alt(0),Alt(1),Alt(3),Alt(2)]),
                Cycle::new(&[Alt(2),Alt(4),Alt(3)]),
            ]
        );
    }

    fn column<'a, I, T, F>(it : I, col : F) -> Vec<(u32, T)>
        where
            I : IntoIterator<Item=&'a Row>,
            F : Fn(&'a Row) -> T,
            T : Zero,
    {
        it.into_iter().filter_map(
            |r| {
                let t = col(r);
                if t.is_zero() {
                    None
                } else {
                    Some((r.cycle_length, t))
                }
            }
        ).collect()
    }

    #[test]
    fn test_run() {
        let choices = choices![
            [0,1] -> [0,1],
            [1,2] -> [2],
            [0,2] -> [0],
            [1,3] -> [3],
            [2,3] -> [2],
            [2,4] -> [4],
            [4,3] -> [3],
            [1,2,3] -> [3]  // collides with [2,3] -> 2, okay with [1,2] -> 2 and [1,3] -> 3
        ];

        let request = testreq(5, choices);
        let detailed = run(&request).unwrap();

        assert_eq!(
            column(&detailed.rows, |r| r.sarp.clone()),
            &[(2, BigUint::from(1u32)), (3, BigUint::from(1u32))]
        );
        assert_eq!(detailed.warp, BigUint::from(1u32));
        assert_eq!(
            column(&detailed.rows, |r| r.garp.clone()),
            &[(2, BigUint::from(1u32)), (3, BigUint::from(2u32)), (4, BigUint::from(2u32))]
        );
        assert_eq!(
            column(&detailed.rows, |r| r.garp_binary_menus.clone()),
            &[(3, BigUint::from(2u32)), (4, BigUint::from(1u32))]
        );
    }

    #[test]
    fn warp_pairs() {
        let choices = choices![
            [0,1] -> [0,1],
            [1,2] -> [],
            [0,2] -> [0],
            [1,3] -> [3],
            [2,3] -> [2],
            [2,4] -> [4],
            [4,3] -> [3],
            [1,2,3] -> [3]  // collides with [2,3] -> 2, okay with [1,2] -> 2 and [1,3] -> 3
        ];
        let request = testreq(5, choices);
        let response = run(&request).unwrap();

        assert_eq!(response.warp_pairs, 1);
    }

    #[test]
    fn inconsistent_augmentation() {
        let choices = choices![
            [0,1,2] -> [0,1,2],
            [0,1,2,3] -> [0]
        ];

        let request = testreq(5, choices);
        let response = run(&request).unwrap();

        assert_eq!(response.warp_pairs, 1);
        assert_eq!(response.warp, BigUint::from(2u32));
    }

    #[test]
    fn binary_intransitivities_acyclic() {
        let choices = choices![
            [0,1] -> [0],
            [1,2] -> [1],
            [0,2] -> [2]
        ];

        let request = testreq(5, choices);
        let response = run(&request).unwrap();

        let mut total_binary_intransitivities : BigUint = num::zero();
        for row in &response.rows {
            total_binary_intransitivities += &row.binary_intransitivities;
        }

        assert_eq!(total_binary_intransitivities, BigUint::from(3u32));
    }

    #[test]
    fn binary_intransitivities_size2() {
        // this test contains repeated menus
        // which is outside of the scope of the spec
        // if it breaks, it does not necessarily mean that Prest is broken
        // because it may break only for repeated-menu datasets
        let choices = choices![
            [0,1] -> [0],
            [0,1] -> []
        ];

        let request = testreq(5, choices);
        let response = run(&request).unwrap();

        let mut total_binary_intransitivities : BigUint = num::zero();
        for row in &response.rows {
            total_binary_intransitivities += &row.binary_intransitivities;
        }

        assert_eq!(total_binary_intransitivities, BigUint::from(1u32));
    }

    fn contraction_consistency(
        alt_count : u32,
        e_total_binary_intransitivities : u32,
        e_contraction_consistency_pairs : u32,
        e_contraction_consistency_all : u32,
        choices : Vec<ChoiceRow>,
    ) {
        let request = testreq(alt_count, choices);
        let response = run(&request).unwrap();

        let mut total_binary_intransitivities : BigUint = num::zero();
        for row in &response.rows {
            total_binary_intransitivities += &row.binary_intransitivities;
        }

        assert_eq!(
            total_binary_intransitivities,
            e_total_binary_intransitivities.into(),
            "total binary intransitivities"
        );
        assert_eq!(
            response.contraction_consistency_pairs,
            e_contraction_consistency_pairs,
            "contraction consistency (pairs)"
        );
        assert_eq!(
            response.contraction_consistency_all,
            e_contraction_consistency_all,
            "contraction consistency (all)"
        );
    }

    #[test]
    fn contraction_consistency_basic() {
        contraction_consistency(
            5,
            0, 1, 2,
            choices![
                [0,1,2,3,4] -> [0,1],
                [0,1,2,3] -> [2,3]
            ],
        );
    }

    #[test]
    fn contraction_consistency_rep1() {
        // this test contains repeated menus
        // which is outside of the scope of the spec
        // if it breaks, it does not necessarily mean that Prest is broken
        // because it may break only for repeated-menu datasets
        contraction_consistency(
            5,
            0, 4, 8,
            choices![
                [0,1,2,3,4] -> [0,1],
                [0,1,2,3] -> [2,3],
                [0,1,2,3,4] -> [0,1],
                [0,1,2,3] -> [2,3]
            ],
        );
    }

    #[test]
    fn contraction_consistency_2() {
        contraction_consistency(
            4,
            0, 2, 2,
            choices![
                [0,1,2,3] -> [0],
                [0,1,2] -> [1],
                [0,1,3] -> [],
                [0,2,3] -> [0,2]
            ],
        );
    }

    #[test]
    fn contraction_consistency_3() {
        contraction_consistency(
            4,
            0, 2, 3,
            choices![
                [0,1,2,3] -> [0,1],
                [0,1,2] -> [1],
                [0,1,3] -> [],
                [0,2,3] -> [0,2]
            ],
        );
    }

    #[test]
    fn contraction_consistency_4() {
        contraction_consistency(
            5,
            0, 5, 6,
            choices![
                [0,1,2,3,4] -> [0,1],
                [0,1,2,3] -> [0],
                [0,1,2] -> [1],
                [0,1,3] -> [],
                [0,2,3] -> [0,2]
            ],
        );
    }

    #[test]
    fn rotation() {
        assert_eq!(rotate_min(&[3,1,2,4]), &[1,2,4,3]);
        assert_eq!(rotate_min(&[1,2,3,4]), &[1,2,3,4]);
        assert_eq!(rotate_min(&[2,3,4,1]), &[1,2,3,4]);
        assert_eq!(rotate_min(&[3,4,1,2]), &[1,2,3,4]);
        assert_eq!(rotate_min(&[2,2,2,2]), &[2,2,2,2]);
        assert_eq!(rotate_min::<u32>(&[]), &[]);
        assert_eq!(rotate_min(&[1]), &[1]);
        assert_eq!(rotate_min(&[1,2]), &[1,2]);
        assert_eq!(rotate_min(&[2,1]), &[1,2]);
    }
}
