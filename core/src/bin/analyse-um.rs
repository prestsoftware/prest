extern crate csv;
extern crate prest;

use prest::alt::Alt;
use prest::alt_set::AltSet;
use std::collections::HashMap;

#[derive(Clone)]
struct ExpRow {
    position : u32,
    menu : AltSet,
    choice : AltSet,
}

struct OutRow {
    exp_row : ExpRow,
    n_instances : u32,
    n_tweaks : u32,
}

struct Subject<ChoiceRow> {
    name : String,
    choices : Vec<ChoiceRow>,
}

fn parse_alts(alternatives : &mut Vec<String>, s : &str) -> AltSet {
    s.split(',').map(|alt_s| {
        match alternatives.iter().position(|s| s == alt_s) {
            None => {
                let i = alternatives.len();
                alternatives.push(String::from(alt_s));
                Alt(i as u32)
            }

            Some(i) => Alt(i as u32)
        }
    }).collect()
}

fn load_stdin() -> (Vec<String>, Vec<Subject<ExpRow>>) {
    let mut alternatives : Vec<String> = Vec::new();
    let mut subjects : HashMap<String, Subject<ExpRow>> = HashMap::new();

    let mut csv_reader = csv::Reader::from_reader(std::io::stdin());
    assert_eq!(csv_reader.headers().unwrap(), vec!["position", "subject", "menu", "choice"]);

    for result in csv_reader.records() {
        let record = result.unwrap();
        let fields : Vec<&str> = record.iter().collect();
        match fields.as_slice() {
            [s_pos, s_subject, s_menu, s_choice] => {
                let subject = subjects.entry(String::from(*s_subject)).or_insert(
                    Subject{ name: String::from(*s_subject), choices: Vec::new() }
                );

                subject.choices.push(
                    ExpRow {
                        position: s_pos.parse().unwrap(),
                        menu: parse_alts(&mut alternatives, *s_menu),
                        choice: parse_alts(&mut alternatives, *s_choice),
                    }
                );
            }

            _ => {
                panic!("could not parse CSV row");
            }
        }
    }

    (alternatives, subjects.into_iter().map(|(_k,v)| v).collect())
}

struct Instance {
    diff : Vec<bool>,
}

fn process(alternatives : &[String], subjects : &[Subject<ExpRow>]) -> Vec<Subject<OutRow>> {
    subjects.iter().map(|subject| {
        let mut best_score : Option<u32> = None;
        let mut best_instances : Vec<Instance> = Vec::new();

        for p in prest::linear_preorders::all(alternatives.len() as u32) {
            let diff : Vec<bool> = subject.choices.iter().map(|cr| {
                let model_choice = prest::model::preorder_maximization(
                    &p, cr.menu.view()
                );

                cr.choice != model_choice
            }).collect();
            let score = diff.iter().map(|d| d).len() as u32;
            let instance = Instance{diff};

            match best_score {
                None => {
                    best_score = Some(score);
                    best_instances = vec![instance];
                }

                Some(best_score_so_far) => {
                    if score > best_score_so_far {
                        // do nothing
                    } else if score == best_score_so_far {
                        best_instances.push(instance);
                    } else {
                        assert!(score < best_score_so_far);
                        best_score = Some(score);
                        best_instances = vec![instance];
                    }
                }
            }
        }

        let mut tweaks = vec![0; subject.choices.len()];
        for instance in &best_instances {
            for (n_tweaks, is_tweak) in tweaks.iter_mut().zip(&instance.diff) {
                *n_tweaks += *is_tweak as u32
            }
        }

        assert_eq!(subject.choices.len(), tweaks.len());

        Subject {
            name: subject.name.clone(),
            choices: subject.choices.iter().zip(tweaks.into_iter()).map(
                |(exp_row, n_tweaks)| OutRow {
                    exp_row: (*exp_row).clone(),
                    n_instances: best_instances.len() as u32,
                    n_tweaks,
                }
            ).collect(),
        }
    }).collect()
}

fn main() {
    let (alternatives, subjects) = load_stdin();
    assert_eq!(alternatives.len(), 5);

    let _ = process(&alternatives, &subjects);

}
