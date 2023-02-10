use std::hash::Hash;
use std::collections::HashSet;

pub fn greedy<T : Eq+Hash+Clone>(sets : &[HashSet<T>]) -> HashSet<usize> {
    let universe : HashSet<T> = sets.iter().fold(
        HashSet::new(), 
        |mut univ, set| { univ.extend(set.clone()); univ },
    );

    let mut covered : HashSet<T> = HashSet::new();
    let mut selected_indices = HashSet::new();

    while covered != universe {
        let mut best_score = 0;
        let mut best_index = None;

        for (i, set) in sets.iter().enumerate() {
            if selected_indices.contains(&i) {
                // already done
                continue;
            }

            let score = set.difference(&covered).count();
            if score > best_score {
                best_score = score;
                best_index = Some(i);
            }
        }

        let i = best_index.unwrap();
        covered.extend(sets[i].clone());
        selected_indices.insert(i);
    }

    selected_indices
}
