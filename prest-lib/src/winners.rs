pub struct Winners<S, T> {
    best_score : Option<S>,
    winners : Vec<T>,
}

impl<S : Ord, T> Winners<S, T> {
    pub fn new() -> Winners<S, T> {
        Winners {
            best_score : None,
            winners : Vec::new(),
        }
    }

    pub fn add(&mut self, score : S, candidate : T) {
        match self.best_score {
            None => {
                self.best_score = Some(score);
                self.winners = vec![candidate];
            },

            Some(ref mut best_score) => {
                if score > *best_score {
                    *best_score = score;
                    self.winners = vec![candidate];
                } else if score == *best_score {
                    self.winners.push(candidate);
                }
            },
        }
    }

    pub fn into_result(self) -> Option<(S, Vec<T>)> {
        Some((self.best_score?, self.winners))
    }
}

pub fn run_iter<S, T, I>(it : I) -> Option<(S, Vec<T>)>
    where S : Ord, I : IntoIterator<Item = (S, T)>
{
    let mut winners = Winners::new();
    for (score, candidate) in it {
        winners.add(score, candidate);
    }
    winners.into_result()
}

pub fn run_iter_with_score<S, T, I, F>(it : I, get_score : F) -> Option<(S, Vec<T>)>
    where S : Ord, I : IntoIterator<Item=T>, F : Fn(&T) -> S
{
    let mut winners = Winners::new();
    for candidate in it {
        winners.add(get_score(&candidate), candidate);
    }
    winners.into_result()
}
