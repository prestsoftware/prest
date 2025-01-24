use crate::alt::Alt;
use crate::alt_set::{AltSet,AltSetView};
use crate::preorder::Preorder;
use crate::common::ChoiceRow;
use std::collections::HashMap;
use std::iter::{FromIterator,repeat};

pub fn set_insert(x : Alt, xs : AltSetView) -> AltSet {
    let mut result = AltSet::from(xs);
    result |= &AltSet::singleton(x);
    result
}

// As given by Dutta and Horan (2015), p. 186
pub fn sequentially_rationalizable_choice(
    alt_count : u32,
    choices : &[ChoiceRow],
) -> Option<(Preorder, Preorder)> {
    let cmap : HashMap<_,_> = choices.iter().map(|cr| (cr.menu.view(), cr.choice.view())).collect();

    let p = {
        let mut p = Preorder::diagonal(alt_count);
        for (a, b) in Alt::all_pairs(alt_count) {
            if a == b {
                continue;
            }

            let just_b = AltSet::singleton(b);
            let just_ab = AltSet::from_iter(&[a,b]);

            p.set_leq(a, b,
                choices.iter().any(|cr|
                    (cr.choice == just_b)
                    && (cr.menu.size() < alt_count)
                    && !cmap.get(
                            // when we add "a" to the menu where "b" was chosen
                            &set_insert(a, cr.menu.view()).view()
                        ).unwrap_or(
                            // if not present in dataset, we just say that "b" was chosen again
                            &just_b.view()
                        ).is_strict_subset_of(just_ab.view())
                        // ^^ but the result is neither "a", nor "b"
                )
            );

            #[cfg(test)]
            println!("[p] {} < {} = {}", a.index(), b.index(), p.leq(a, b));
        }

        p
    };

    let q = {
        let mut q = Preorder::diagonal(alt_count);
        for (a, b) in Alt::all_pairs(alt_count) {
            if a == b {
                continue;
            }

            let just_a = AltSet::singleton(a);
            let just_b = AltSet::singleton(b);

            #[allow(non_snake_case)]
            q.set_leq(a, b,
                choices.iter().flat_map(
                    |crA| repeat(crA).zip(choices)
                ).any(|(crA, crB)|
                    just_b.view().is_strict_subset_of(crA.menu.view())
                    && crA.menu.view().is_strict_subset_of(crB.menu.view())
                    && crA.choice == just_a
                    && crB.choice == just_b
                )
            );

            #[cfg(test)]
            println!("[q] {} < {} = {}", a.index(), b.index(), q.leq(a, b));
        }

        q
    };

    if cfg!(test) {
        assert!(p.is_transitive());
        assert!(p.is_reflexive());
        assert!(p.is_strict());

        assert!(q.is_transitive());
        assert!(q.is_reflexive());
        assert!(q.is_strict());
    }

    if p.is_transitive() && p.is_reflexive() && p.is_strict()
        && q.is_transitive() && q.is_reflexive() && q.is_strict()
    {
        Some((p, q))
    } else {
        None
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn subsets() {
        let just_a = alts![2];
        let just_b = alts![1];
        let crA = ChoiceRow{ menu: alts![1,2], default: None, choice: alts![2] };
        let crB = ChoiceRow{ menu: alts![0,1,2], default: None, choice: alts![1] };

        assert!(just_b.view().is_strict_subset_of(crA.menu.view()));
        assert!(crA.menu.view().is_strict_subset_of(crB.menu.view()));
        assert!(crA.choice == just_a);
        assert!(crB.choice == just_b);
    }

    #[test]
    fn yorgos() {
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
        let (p, q) = sequentially_rationalizable_choice(4, &choices).unwrap();

        assert!(p.leq(Alt(0), Alt(2)));

        let just_a = alts![2];
        let just_b = alts![1];
        #[allow(non_snake_case)]
        let qab =
            choices.iter().flat_map(
                |crA| repeat(crA).zip(choices.iter())
            ).any(|(crA, crB)|
                just_b.view().is_strict_subset_of(crA.menu.view())
                && crA.menu.view().is_strict_subset_of(crB.menu.view())
                && crA.choice == just_a
                && crB.choice == just_b
            );
        assert!(qab);

        // A = {1,2}   -> a = {2}
        // B = {0,1,2} -> b = {1}
        assert!(q.leq(Alt(2), Alt(1)));
    }

    #[test]
    fn linear_3() {
        let (p, q) = sequentially_rationalizable_choice(3, &choices![
            [0,1,2] -> [2],
            [0,1] -> [1],
            [0,2] -> [2],
            [1,2] -> [2],
            [2] -> [2],
            [1] -> [1],
            [0] -> [0]
        ]).unwrap();

        assert_eq!(p, Preorder::from_values(&[0,1,2]));
        assert_eq!(q, Preorder::diagonal(3));
    }

    #[test]
    fn linear_4() {
        let (p, q) = sequentially_rationalizable_choice(4, &choices![
            [0,1,2,3] -> [3],
            [0,1,2] -> [2],
            [0,1,3] -> [3],
            [0,2,3] -> [3],
            [1,2,3] -> [3],
            [0,1] -> [1],
            [0,2] -> [2],
            [0,3] -> [3],
            [1,2] -> [2],
            [1,3] -> [3],
            [2,3] -> [3],
            [0] -> [0],
            [1] -> [1],
            [2] -> [2],
            [3] -> [3]
        ]).unwrap();

        assert_eq!(p, Preorder::from_values(&[0,1,2,3]));
        assert_eq!(q, Preorder::diagonal(4));
    }
}
*/
