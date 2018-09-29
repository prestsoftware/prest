use std::collections::HashMap;
use std::collections::hash_map::Entry;

type Matrix = u64;

#[inline]
fn ix(m : Matrix, i : usize, j : usize) -> bool {
    (m >> (8*i + j)) & 0x1 != 0
}

fn next(n : usize, matrix : Matrix, ij : (usize, usize)) -> Option<(usize, usize)> {
    let (mut i, mut j) = ij;

    loop {
        j += 1;

        if j == n {
            // EOL
            j = 0;
            i += 1;
        }

        if i == n {
            // EOF
            return None;
        }

        // we found an empty space!
        if !ix(matrix, i, j) {
            return Some((i, j))
        }
    }
}

fn choose(n : usize, history : &mut HashMap<Matrix, usize>, matrix : Matrix, ij : (usize, usize)) {
    let cur_ij = ij.0 * n + ij.1;
    if let Some(&prev_ij) = history.get(&matrix) {
        if cur_ij >= prev_ij {
            return; // been there, done that
        }
    }

    // either choose 0
    if let Some(ij_next) = next(n, matrix, ij) {
        choose(n, history, matrix, ij_next);
    }

    // or choose 1
    let mut new_matrix = matrix.clone();

    new_matrix |= 1 << (8*ij.0 + ij.1);
    let mut prop = vec![(0,0); n*n];
    let mut prop_i = 1;
    prop[0] = ij;

    while prop_i > 0 {
        prop_i -= 1;
        let ij = prop[prop_i];

        for k in 0..n {
            if ix(new_matrix, k, ij.0) && !ix(new_matrix, k, ij.1) {
                new_matrix |= 1 << (8*k + ij.1);

                prop[prop_i] = (k, ij.1);
                prop_i += 1;
            }

            if ix(new_matrix, ij.1, k) && !ix(new_matrix, ij.0, k) {
                new_matrix |= 1 << (8*ij.0 + k);

                prop[prop_i] = (ij.0, k);
                prop_i += 1;
            }
        }
    }

    if let Some(ij_next) = next(n, new_matrix, ij) {
        choose(n, history, new_matrix, ij_next);
    } else {
        // leaf!
        history.insert(new_matrix, cur_ij);
    }

    match history.entry(matrix) {
        Entry::Vacant(e) => {
            e.insert(cur_ij);
        }

        Entry::Occupied(mut e) => {
            if cur_ij < *e.get() {
                e.insert(cur_ij);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FastPreorder(pub u64);

// we gotta keep 'em in the memory, anyway...
pub fn all(n : u32) -> Vec<FastPreorder> {
    if n == 0 {
        return vec![FastPreorder(0)];
    }

    if n == 1 {
        return vec![FastPreorder(1)];
    }

    let mut matrix = 0;
    let mut history = HashMap::new();

    // set the diagonal
    for i in 0..n {
        matrix |= 1 << (8*i + i);
    }

    // 0,0 is on the diagonal so we start with 0,1
    choose(n as usize, &mut history, matrix, (0, 1));

    history.keys().cloned().map(FastPreorder).collect()
}
