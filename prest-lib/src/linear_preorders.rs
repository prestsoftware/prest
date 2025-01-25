use crate::preorder::Preorder;

// https://en.wikipedia.org/wiki/Steinhaus-Johnson-Trotter_algorithm
pub struct LinearPreorders {
    vals : Vec<u32>,
    dirs : Vec<i8>,
    first : bool,
}

pub fn all(alt_count : u32) -> LinearPreorders {
    let vals : Vec<_> = (0..alt_count).collect();
    let dirs = vals.iter().map(|&i| if i == 0 { 0 } else { -1 }).collect();
    LinearPreorders {
        vals,
        dirs,
        first: true,
    }
}

impl Iterator for LinearPreorders {
    type Item = Preorder;

    fn next(&mut self) -> Option<Preorder> {
        // first permutation is already available
        if self.first {
            self.first = false;
            return Some(Preorder::from_values(&self.vals));
        }

        // get the greatest element with non-zero direction
        let mut best = None;
        for (i, (&val, &dir)) in self.vals.iter().zip(self.dirs.iter()).enumerate() {
            if dir != 0 {
                match best {
                    None => { best = Some((i, val, dir)) },
                    Some((_j, j_val, _j_dir)) => {
                        if val > j_val {
                            best = Some((i, val, dir));
                        }
                    }
                }
            }
        }

        if let Some((i, _i_val, i_dir)) = best {
            // println!("{}: {}/{}", i, i_value, i_dir);

            // swap
            let j = (i as isize + i_dir as isize) as usize;
            self.vals.swap(i, j);
            self.dirs.swap(i, j);

            // adjust directions
            let k = j as isize + i_dir as isize;
            if k < 0 || k >= self.vals.len() as isize || self.vals[k as usize] > self.vals[j as usize] {
                self.dirs[j] = 0;
            }

            // start greater elements
            for l in 0..j {
                if self.vals[l] > self.vals[j] {
                    self.dirs[l] = 1;
                }
            }

            for l in j+1..self.vals.len() {
                if self.vals[l] > self.vals[j] {
                    self.dirs[l] = -1;
                }
            }

            Some(Preorder::from_values(&self.vals))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn counts() {
        assert_eq!(super::all(0).count(),        1);
        assert_eq!(super::all(1).count(),        1);
        assert_eq!(super::all(2).count(),        2);
        assert_eq!(super::all(3).count(),        6);
        assert_eq!(super::all(4).count(),       24);
        assert_eq!(super::all(5).count(),      120);
        assert_eq!(super::all(6).count(),      720);
        /* let's not overload debug builds
        assert_eq!(super::all(7).count(),     5040);
        assert_eq!(super::all(8).count(),    40320);
        assert_eq!(super::all(9).count(),   362880);
        assert_eq!(super::all(10).count(), 3628800);
        assert_eq!(super::all(11).count(), 39916800); // takes a few seconds even with --release
        */
    }
}
