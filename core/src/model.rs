use alt::Alt;
use preorder::Preorder;
use alt_set::{AltSet,AltSetView};
use std::result::Result;
use linear_preorders;
use precomputed::Precomputed;
use precomputed::Error as PreorderError;
use std::fmt;
use std::cmp;
use std::io::{Read,Write};
use std::iter::FromIterator;
use codec::{self,Encode,Decode};
use rpc_common::{ChoiceRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreorderParams {
    pub strict: Option<bool>,
    pub total: Option<bool>,
}

impl Encode for PreorderParams {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (self.strict, self.total).encode(f)
    }
}

impl Decode for PreorderParams {
    fn decode<R : Read>(f : &mut R) -> codec::Result<PreorderParams> {
        let (strict, total) = Decode::decode(f)?;
        Ok(PreorderParams{ strict, total })
    }
}

impl fmt::Display for PreorderParams {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        fn symbol(x : Option<bool>) -> &'static str {
            match x {
                None => &"?",
                Some(true) => &"",
                Some(false) => &"¬",
            }
        }

        write!(f, "{}S, {}T",
            symbol(self.strict),
            symbol(self.total),
        )
    }
}

impl PreorderParams {
    fn from_preorder(p : &Preorder) -> PreorderParams {
        PreorderParams {
            strict: Some(p.is_strict()),
            total: Some(p.is_total()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Model {
    PreorderMaximization(PreorderParams),
    Unattractiveness(PreorderParams),
    UndominatedChoice{ strict: bool },  // always with {total: Some(false)}

    PartiallyDominantChoice{
        /// With forced choice, all-incomparable leads to "choose all".
        /// Without FC, all-incomparable leads to "choose none".
        fc : bool,
    },  // the preorder is always strict: Some(true), total: Some(false)

    StatusQuoUndominatedChoice,
    Overload(PreorderParams),
    TopTwo,
    SequentiallyRationalizableChoice,
    HybridDomination{strict: bool},
}

impl Encode for Model {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        match self {
            &Model::PreorderMaximization(p) => (0u8, p).encode(f),
            &Model::Unattractiveness(p) => (1u8, p).encode(f),
            &Model::UndominatedChoice{strict} => (2u8,strict).encode(f),
            &Model::PartiallyDominantChoice{fc} => (3u8, fc).encode(f),
            &Model::StatusQuoUndominatedChoice => 4u8.encode(f),
            &Model::Overload(p) => (5u8, p).encode(f),
            &Model::TopTwo => 6u8.encode(f),
            &Model::SequentiallyRationalizableChoice => 7u8.encode(f),
            &Model::HybridDomination{strict} => (8u8, strict).encode(f),
        }
    }
}

impl Decode for Model {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Model> {
        match Decode::decode(f)? {
            0u8 => Ok(Model::PreorderMaximization(Decode::decode(f)?)),
            1u8 => Ok(Model::Unattractiveness(Decode::decode(f)?)),
            2u8 => Ok(Model::UndominatedChoice{strict: Decode::decode(f)?}),
            3u8 => Ok(Model::PartiallyDominantChoice{fc: Decode::decode(f)?}),
            4u8 => Ok(Model::StatusQuoUndominatedChoice),
            5u8 => Ok(Model::Overload(Decode::decode(f)?)),
            6u8 => Ok(Model::TopTwo),
            7u8 => Ok(Model::SequentiallyRationalizableChoice),
            8u8 => Ok(Model::HybridDomination{strict: Decode::decode(f)?}),
            _ => Err(codec::Error::BadEnumTag),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Instance {
    PreorderMaximization(Preorder),
    Unattractiveness {
        p : Preorder,
        mask : AltSet,
    },
    UndominatedChoice(Preorder),
    PartiallyDominantChoice {
        p : Preorder,
        fc : bool,
    },
    StatusQuoUndominatedChoice(Preorder),
    Overload {
        p : Preorder,
        limit : u32,
    },
    TopTwo(Preorder),
    SequentiallyRationalizableChoice(Preorder, Preorder),
    HybridDomination(Preorder),
}

impl Encode for Instance {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        match self {
            &Instance::PreorderMaximization(ref p)
                => (0u8, p).encode(f),

            &Instance::Unattractiveness{ref p, ref mask}
                => { (1u8, p).encode(f)?; mask.view().to_blocks().encode(f) }

            &Instance::UndominatedChoice(ref p)
                => (2u8, p).encode(f),

            &Instance::PartiallyDominantChoice{ref p, fc}
                => (3u8, p, fc).encode(f),

            &Instance::StatusQuoUndominatedChoice(ref p)
                => (4u8, p).encode(f),

            &Instance::Overload{ref p, limit}
                => (5u8, p, limit).encode(f),

            &Instance::TopTwo(ref p)
                => (6u8, p).encode(f),

            &Instance::SequentiallyRationalizableChoice(ref p, ref q)
                => (7u8, p, q).encode(f),

            &Instance::HybridDomination(ref p)
                => (8u8, p).encode(f),
        }
    }
}

impl Decode for Instance {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Instance> {
        match Decode::decode(f)? {
            0u8 => Ok(Instance::PreorderMaximization(Decode::decode(f)?)),
            1u8 => Ok(Instance::Unattractiveness{
                p: Decode::decode(f)?,
                mask: AltSet::from_blocks(Decode::decode(f)?),
            }),
            2u8 => Ok(Instance::UndominatedChoice(Decode::decode(f)?)),
            3u8 => Ok(Instance::PartiallyDominantChoice{
                p: Decode::decode(f)?,
                fc: Decode::decode(f)?,
            }),
            4u8 => Ok(Instance::StatusQuoUndominatedChoice(Decode::decode(f)?)),
            5u8 => Ok(Instance::Overload{
                p: Decode::decode(f)?,
                limit: Decode::decode(f)?,
            }),
            6u8 => Ok(Instance::TopTwo(Decode::decode(f)?)),
            7u8 => Ok(Instance::SequentiallyRationalizableChoice(
                Decode::decode(f)?,
                Decode::decode(f)?,
            )),
            8u8 => Ok(Instance::HybridDomination(Decode::decode(f)?)),
            _ => Err(codec::Error::BadEnumTag),
        }
    }
}

fn preorder_maximization(p : &Preorder, menu : AltSetView) -> AltSet {
    let mut result = AltSet::from(menu);
    for i in menu.iter() {
        result &= p.upset(i);
    }
    result
}

fn undominated_choice(p : &Preorder, menu : AltSetView) -> AltSet {
    // we can't use p.transpose().upset() because we don't have p.transpose()
    menu.iter().filter(
        // select elements i such that
        // there is no j that would dominate i
        |&i| !menu.iter().any(|j| p.lt(i, j))
    ).collect()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Penalty {
    // both bounds are inclusive
    pub lower_bound : u32,
    pub upper_bound : u32,
}

impl Penalty {
    pub fn exact(value : u32) -> Penalty {
        Penalty{lower_bound: value, upper_bound: value}
    }

    pub fn merge_min(&mut self, other : &Penalty) {
        if other.lower_bound < self.lower_bound {
            self.lower_bound = other.lower_bound;
        }

        if other.upper_bound < self.upper_bound {
            self.upper_bound = other.upper_bound;
        }
    }
}

impl Encode for Penalty {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (self.lower_bound, self.upper_bound).encode(f)
    }
}

impl Decode for Penalty {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Penalty> {
        Ok(Penalty {
            lower_bound: Decode::decode(f)?,
            upper_bound: Decode::decode(f)?,
        })
    }
}

impl Instance {
    pub fn determine_model(&self) -> Model {
        match self {
            &Instance::PreorderMaximization(ref p) =>
                Model::PreorderMaximization(PreorderParams::from_preorder(p)),

            &Instance::Unattractiveness{ref p, mask: ref _mask} =>
                Model::Unattractiveness(PreorderParams::from_preorder(p)),

            &Instance::UndominatedChoice(ref p) =>
                Model::UndominatedChoice{strict: p.is_strict()},

            &Instance::PartiallyDominantChoice{fc, ..} =>
                Model::PartiallyDominantChoice{fc},

            &Instance::StatusQuoUndominatedChoice(_) =>
                Model::StatusQuoUndominatedChoice,

            &Instance::Overload{ref p, ..} =>
                Model::Overload(PreorderParams::from_preorder(p)),

            &Instance::TopTwo(_) =>
                Model::TopTwo,

            &Instance::SequentiallyRationalizableChoice(_,_) =>
                Model::SequentiallyRationalizableChoice,

            &Instance::HybridDomination(ref p) =>
                Model::HybridDomination{strict: p.is_strict()},
        }
    }

    pub fn choice(&self, menu : AltSetView, default_opt : Option<Alt>) -> AltSet {
        assert!(menu.is_nonempty());

        match self {
            &Instance::PreorderMaximization(ref p) => {
                preorder_maximization(p, menu)
            }

            &Instance::Unattractiveness{ref p, ref mask} => {
                let mut result = preorder_maximization(p, menu);
                result &= mask;
                result
            }

            &Instance::UndominatedChoice(ref p) => {
                undominated_choice(p, menu)
            }

            &Instance::PartiallyDominantChoice{ref p, fc} => {
                // for efficiency, we *could* rely on p being strict here
                // (because it is for this model)
                // and check only Preorder::leq() rather than Preorder::lt()
                //
                // however, this requires an extra check that i != j
                // and feels quite error-prone so let's go
                // with the slightly more fool-proof and less efficient option
                //
                // we may revisit this in the future
                let result : AltSet = menu.iter().filter(|&i| {
                       menu.iter().all(|j| !p.lt(i, j))  // is not dominated itself
                    && menu.iter().any(|j|  p.lt(j, i))  // but dominates *something*
                }).collect();

                if result.view().is_empty() && fc {
                    // if there's no such element but we must choose
                    // we choose everything
                    AltSet::from(menu)
                } else {
                    result
                }
            }

            &Instance::HybridDomination(ref p) => {
                // maximally dominant choice = preorder maximisation on incomplete preorders
                let choice = preorder_maximization(p, menu);
                if choice.view().is_nonempty() {
                    // if MDC did yield an answer, that's what we return
                    choice
                } else {
                    // otherwise try UC
                    undominated_choice(p, menu)
                }
            }

            &Instance::StatusQuoUndominatedChoice(ref p) => {
                let default = default_opt.expect(
                    "No default value provided for Status-quo-biased Undominated Choice"
                );

                if menu.iter().all(|i| !p.lt(default, i)) {
                    // the default is not dominated
                    // so that's our answer
                    AltSet::from_iter(&[default])
                } else {
                    p.upset(default).iter().filter(
                        |&i| p.lt(default, i)  // dominates the default
                            && menu.iter().all(|j| !p.lt(i, j))  // undominated
                    ).collect()
                }
            }

            &Instance::Overload{ref p, limit} => {
                if menu.size() > limit {
                    AltSet::empty()
                } else {
                    preorder_maximization(p, menu)
                }
            }

            &Instance::TopTwo(ref p) => {
                debug_assert!(p.is_strict());
                debug_assert!(p.is_total());

                let n = menu.size();
                if n <= 2 {
                    return AltSet::from(menu);
                }

                // top 2 elements
                p.as_linear_order().into_iter().filter(|&i| menu.contains(i)).take(2).collect()
            }

            &Instance::SequentiallyRationalizableChoice(ref p, ref q) => {
                debug_assert!(p.is_strict());
                debug_assert!(q.is_strict());

                let shortlist = undominated_choice(p, menu);
                let answer = undominated_choice(q, shortlist.view());

                // requirement of the model
                // fairly cheap to check so we do it
                assert!(answer.size() == 1);

                answer
            }
        }
    }

    pub fn penalty(&self, crs : &[ChoiceRow]) -> Penalty {
        let upper_bound = crs.iter().map(|cr| {
            let standard_penalty =
                if cr.choice == self.choice(cr.menu.view(), cr.default) { 0 } else { 1 };

            if cr.menu.view().is_singleton() {
                if let Instance::PartiallyDominantChoice{p:_,fc:_} = self {
                    // PDC should not be penalised for deferring at singletons
                    0
                } else {
                    standard_penalty
                }
            } else {
                standard_penalty
            }
        }).sum();

        let lower_bound = match self {
            &Instance::SequentiallyRationalizableChoice(_,_)
                => cmp::min(1, upper_bound),
            _
                => upper_bound,
        };

        Penalty{lower_bound, upper_bound}
    }
}

#[derive(Debug)]
pub enum InstanceError {
    TooManyAlternatives {
        model: Model,
        alt_count: u32,
    },
    NeedPrecomputedPreorders,
}

impl Encode for InstanceError {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        match self {
            &InstanceError::TooManyAlternatives{ref model, alt_count}
                => (0u8, model, alt_count).encode(f),

            &InstanceError::NeedPrecomputedPreorders
                => 1u8.encode(f),
        }
    }
}

impl PreorderError {
    fn annotate(self, model : Model) -> InstanceError {
        match self {
            PreorderError::TooManyAlternatives(alt_count)
                => InstanceError::TooManyAlternatives {
                    model,
                    alt_count,
                },

            PreorderError::NeedPrecomputedPreorders
                => InstanceError::NeedPrecomputedPreorders,
        }
    }
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        match self {
            &InstanceError::TooManyAlternatives{model, alt_count} =>
                // TODO: encode this as <model:AAQmdeF0==>
                // and then search&replace such strings in the GUI
                // in the error handler
                write!(f, "Model \"{:?}\" does not support {} alternatives.", model, alt_count),

            &InstanceError::NeedPrecomputedPreorders =>
                PreorderError::NeedPrecomputedPreorders.fmt(f),
        }
    }
}

#[allow(clippy::collapsible_if)]
fn traverse_preorders<F>(
    precomputed : &Precomputed,
    preorder_params : PreorderParams,
    alt_count : u32,
    f : &mut F
) -> Result<(), PreorderError>
    where F : FnMut(Preorder)
{
    #[inline]
    fn satisfies<F : Fn(&Preorder) -> bool>(p : &Preorder, property : F, requirement : Option<bool>) -> bool {
        match requirement {
            None => true,
            Some(required_value) => property(p) == required_value,
        }
    }

    if let Some(true) = preorder_params.total {
        if let Some(true) = preorder_params.strict {
            // subset of linear orders
            if alt_count > 10 {
                return Err(PreorderError::TooManyAlternatives(alt_count));
            }

            for p in linear_preorders::all(alt_count) {
                f(p);
            }
        } else {
            // subset of weak orders
            if alt_count > 7 {
                return Err(PreorderError::TooManyAlternatives(alt_count));
            }

            for p in &precomputed.get(alt_count)?.weak_orders {
                if satisfies(&p, Preorder::is_strict, preorder_params.strict) {
                    f(p.clone());
                }
            }
        }
    } else {
        if let Some(true) = preorder_params.strict {
            // subset of partial orders
            if alt_count > 7 {
                return Err(PreorderError::TooManyAlternatives(alt_count));
            }

            for p in &precomputed.get(alt_count)?.partial_orders {
                if satisfies(&p, Preorder::is_total, preorder_params.total) {
                    f(p.clone());
                }
            }
        } else {
            for p in &precomputed.get(alt_count)?.preorders {
                if satisfies(&p, Preorder::is_strict, preorder_params.strict)
                    && satisfies(&p, Preorder::is_total, preorder_params.total)
                {
                    f(p.clone());
                }
            }
        }
    }

    Ok(())
}

fn traverse_unattractive<F>(
    precomputed : &Precomputed,
    preorder_params : PreorderParams,
    alt_count : u32,
    f : &mut F
) -> Result<(), PreorderError>
    where F : FnMut(Preorder, AltSet)
{
    if alt_count > 7 {
        return Err(PreorderError::TooManyAlternatives(alt_count));
    }

    // we don't include 0b11111...111 because unattractive=Some(true)
    // it's sufficient to use u32 masks because alt_count is limited by other aspects of the implementation
    for mask_u32 in 0u32 .. (1 << alt_count)-1 {
        traverse_preorders(precomputed, preorder_params, mask_u32.count_ones(),
            &mut |p| f(
                p.stuff(alt_count, mask_u32),
                AltSet::from_block(mask_u32),
            )
        )?;
    }

    Ok(())
}

pub fn traverse_all<F>(
    precomputed : &Precomputed,
    model : Model,
    alt_count : u32,
    choices : &[ChoiceRow],
    f : &mut F,
) -> Result<(), InstanceError>
    where F : FnMut(Instance)
{
    let ann = |e : PreorderError| e.annotate(model);

    match model {
        Model::PreorderMaximization(preorder_params)
            => traverse_preorders(precomputed, preorder_params, alt_count,
                &mut |p| f(Instance::PreorderMaximization(p))
            ).map_err(&ann)?,

        Model::Unattractiveness(preorder_params)
            => traverse_unattractive(precomputed, preorder_params, alt_count,
                &mut |p, mask| f(Instance::Unattractiveness{p, mask})
            ).map_err(&ann)?,

        Model::UndominatedChoice{strict}
            => traverse_preorders(
                precomputed,
                PreorderParams{strict: Some(strict), total: Some(false)},
                alt_count,
                &mut |p| f(Instance::UndominatedChoice(p))
            ).map_err(&ann)?,

        Model::PartiallyDominantChoice{fc}
            => traverse_preorders(
                precomputed,
                PreorderParams{strict: Some(true), total: Some(false)},
                alt_count,
                &mut |p| f(Instance::PartiallyDominantChoice{p, fc})
            ).map_err(&ann)?,

        Model::HybridDomination{strict}
            => traverse_preorders(
                precomputed,
                PreorderParams{strict: Some(strict), total: Some(false)},
                alt_count,
                &mut |p| f(Instance::HybridDomination(p))
            ).map_err(&ann)?,

        Model::StatusQuoUndominatedChoice
            => traverse_preorders(
                precomputed,
                PreorderParams{strict: Some(true), total: Some(false)},
                alt_count,
                &mut |p| f(Instance::StatusQuoUndominatedChoice(p))
            ).map_err(&ann)?,

        Model::Overload(pp)
            => traverse_preorders(precomputed, pp, alt_count, &mut |p| {
                // we have to be overloaded in at least one case
                // limit attains the maximum value of (alt_count-1)
                // so for the full set of size alt_count, this model will defer
                for limit in 0..alt_count {
                    f(Instance::Overload{p: p.clone(), limit});
                }
            }).map_err(&ann)?,

        Model::TopTwo
            => traverse_preorders(precomputed, PreorderParams{strict: Some(true), total: Some(true)}, alt_count,
                &mut |p| f(Instance::TopTwo(p))
            ).map_err(&ann)?,

        Model::SequentiallyRationalizableChoice => {
            traverse_preorders(precomputed,
                PreorderParams{strict: Some(true), total: Some(false)},
                alt_count, &mut |p|
            {
                /* SRC is supported only experimentally. If there is a perfectly rationalising
                 * instance, we will find it. Otherwise, we will probably give a sub-optimal
                 * solution. Either way, we produce at least one instance, however bad, to avoid
                 * the nasty corner case of choosing the best instance from an empty list later.
                 *
                 * How this works:
                 *
                 * SRC has two rationales, P and Q. P yields a shortlist from any given menu,
                 * and Q makes the final choice from the shortlist.
                 * We therefore know that any perfect instance will have a P such that
                 * P(X) is a subseteq of C(X), and then finally Q(P(X)) = C(X).
                 *
                 * We therefore prune the candidates for P using the above criterion
                 * to make the algorithm run faster.
                 *
                 * Finally, in case there is no perfect instance, we just invent one,
                 * for the sake of producing /something/.
                 */
                let shortlist = {
                    // we iterate explicitly to be able to return early
                    let mut shortlist = Vec::with_capacity(choices.len());
                    for cr in choices {
                        let choice = undominated_choice(&p, cr.menu.view());
                        if cr.choice.view().is_subseteq_of(choice.view()) {
                            shortlist.push((&cr.choice, choice));
                        } else {
                            // preorder "p" is incompatible, bail out early
                            return;
                        }
                    }
                    shortlist
                };

                traverse_preorders(precomputed,
                    PreorderParams{strict: Some(true), total: Some(false)},
                    alt_count, &mut |q|
                {
                    for &(cr_choice, ref shortlist) in &shortlist {
                        if undominated_choice(&q, shortlist.view()) != *cr_choice {
                            // gives wrong answer in at least one case, bail out early
                            return;
                        }
                    }

                    // all answers are compatible, list this instance
                    f(Instance::SequentiallyRationalizableChoice(p.clone(), q));
                }).unwrap();
            }).map_err(&ann)?
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use precomputed::Precomputed;
    use super::{AltSet,PreorderParams,Instance};
    use std::collections::HashSet;
    use preorder::Preorder;
    use fast_preorder::FastPreorder;
    use alt::Alt;
    use std::iter::FromIterator;
    /* disabled for now
    use rpc_common::ChoiceRow;
    use approximate_estimation::sequentially_rationalizable_choice;
    use super::*;

    #[test]
    fn seqrc() {
        let choices = choices![
                [0,1,2,3] -> [1],
                [0,1,2] -> [1],
                [0,1,3] -> [1],
                [0,2,3] -> [0],
                [1,2,3] -> [2],
                [0,1] -> [1],
                [0,2] -> [0],
                [0,3] -> [0],
                [1,2] -> [2],
                [1,3] -> [1],
                [2,3] -> [2]
        ];

        /*
        let choices = choices![
                [0,1,2,3] -> [1],
                [0,1,2] -> [1],
                [0,1,3] -> [1],
                [0,2,3] -> [0],
                [1,2,3] -> [3],
                [0,1] -> [1],
                [0,2] -> [0],
                [0,3] -> [3],
                [1,2] -> [1],
                [1,3] -> [3],
                [2,3] -> [3]
        ];
        */

        let (p, q) = sequentially_rationalizable_choice(4, &choices).unwrap();
        println!("P = {:?}", p.simple_digraph());
        println!("Q = {:?}", q.simple_digraph());

        for menu in AltSet::powerset(4) {
            let shortlist = undominated_choice(&p, menu.view());
            let answer = undominated_choice(&q, shortlist.view());
            assert_eq!(answer.size(), 1, "{} -> {}", menu, answer)
        }
    }
    */

    #[test]
    fn partially_dominant() {
        let inst = Instance::PartiallyDominantChoice{
            p: Preorder::from_fast_preorder(5, FastPreorder(0x1F_0F_07_02_01)),
            fc: true,
        };
        assert_eq!(inst.choice(alts![4,2].view(), None), alts![2]);
        assert_eq!(inst.choice(alts![0,1,2,3,4].view(), None), alts![0,1]);
    }

    #[test]
    fn unattractive() {
        fn case(precomp : &Precomputed, alt_count : u32) {
            let mut generated = Vec::new();
            let mut unique = HashSet::new();

            super::traverse_all(
                precomp,
                super::Model::Unattractiveness(PreorderParams{strict:None, total:None}),
                alt_count,
                &[],
                &mut |inst|
            {
                // make sure that the preorders are well-formed
                match &inst {
                    &super::Instance::Unattractiveness{ref p, mask:_} => {
                        assert!(p.is_reflexive());
                        assert!(p.is_transitive());
                    }

                    _ => (),
                }

                generated.push(inst.clone());
                unique.insert(inst);
            }).unwrap();

            assert_eq!(generated.len(), unique.len(), "{:?} != {:?}", generated, unique); // no repetitions
        }

        let mut precomp = Precomputed::new(None);
        precomp.precompute(6).unwrap();
        for size in 0..6 {
            case(&precomp, size);
        }
    }

    #[test]
    fn balance() {
        let mut precomp = Precomputed::new(None);
        precomp.precompute(4).unwrap();
        
        {
            let mut m = 0;
            let mut n = 0;

            super::traverse_all(
                &precomp,
                super::Model::PreorderMaximization(PreorderParams{strict:None,total:None}),
                4,
                &[],
                &mut |inst|
            {
                if inst.choice(alts![2].view(), None).view().contains(Alt(3)) {
                    m += 1;
                }

                if inst.choice(alts![3].view(), None).view().contains(Alt(2)) {
                    n += 1;
                }
            }).unwrap();

            assert_eq!(m, n);
        }
    }
}
