extern crate csv;
extern crate prest;

use prest::alt::Alt;
use prest::alt_set::{AltSet,AltSetView};
use std::collections::HashMap;

#[derive(Clone)]
struct ExpRow {
    position : u32,
    menu : AltSet,
    choice : AltSet,
}

struct OutRow {
    exp_row : ExpRow,
    hm_total : u32,
    n_instances : u32,
    n_tweaks : u32,
}

struct Subject<ChoiceRow> {
    name : String,
    choices : Vec<ChoiceRow>,
}

fn parse_alts(alternatives : &mut Vec<String>, s : &str) -> AltSet {
    if s.trim() == "" {
        return AltSet::empty();
    }

    s.trim().split(',').map(|alt_s| {
        let alt_trimmed = alt_s.trim();
        match alternatives.iter().position(|s| s == alt_trimmed) {
            None => {
                let i = alternatives.len();
                alternatives.push(String::from(alt_trimmed));
                Alt(i as u32)
            }

            Some(i) => Alt(i as u32)
        }
    }).collect()
}

fn fmt_alts(alternatives : &[String], s : AltSetView) -> String {
    s.into_iter().map(
        |Alt(i)| alternatives[i as usize].as_str()
    ).collect::<Vec<&str>>().join(",")
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

    (alternatives, subjects.into_iter().map(
        |(_k, mut v)| {
            v.choices.sort_by(|crx, cry| Ord::cmp(&crx.position, &cry.position));
            v
        }
    ).collect())
}

fn write_stdout(alternatives : &[String], subjects : &[Subject<OutRow>]) {
    let mut csv_writer = csv::Writer::from_writer(std::io::stdout());
    csv_writer.write_record(&[
        "position", "subject", "menu", "choice",
        "n_instances", "n_tweaks", "hm_pa", "hm_avg", "hm_total",
        "position_active", "menu_size", "is_active_choice",
        "largest_menu_seen_excl", "largest_menu_seen_incl",
        "alternatives_seen_excl", "alternatives_seen_incl",
        "deferrals_seen_excl", "deferrals_seen_incl",
    ]).unwrap();

    for subject in subjects {
        let mut i_total = 1u32;
        let mut i_active = 1u32;
        let mut largest_menu_seen = 0u32;
        let mut alternatives_seen = AltSet::empty();
        let mut deferrals_seen = 0u32;

        for cr in &subject.choices {
            assert_eq!(cr.exp_row.position, i_total);

            let largest_menu_seen_incl = largest_menu_seen.max(cr.exp_row.menu.size());
            let alternatives_seen_incl = {
                let mut xs = alternatives_seen.clone();
                xs |= cr.exp_row.menu.view();
                xs
            };
            let deferrals_seen_incl = deferrals_seen + (cr.exp_row.choice.view().is_empty() as u32);

            csv_writer.write_record(&[
                cr.exp_row.position.to_string().as_str(),
                subject.name.as_str(),
                fmt_alts(alternatives, cr.exp_row.menu.view()).as_str(),
                fmt_alts(alternatives, cr.exp_row.choice.view()).as_str(),

                cr.n_instances.to_string().as_str(),
                cr.n_tweaks.to_string().as_str(),
                if cr.n_tweaks > 0 { "1" } else { "0" },
                (cr.n_tweaks as f32 / cr.n_instances as f32).to_string().as_str(),
                cr.hm_total.to_string().as_str(),

                i_active.to_string().as_str(),
                cr.exp_row.menu.size().to_string().as_str(),
                (cr.exp_row.choice.view().is_nonempty() as u32).to_string().as_str(),
                largest_menu_seen.to_string().as_str(),
                largest_menu_seen_incl.to_string().as_str(),
                alternatives_seen.size().to_string().as_str(),
                alternatives_seen_incl.size().to_string().as_str(),
                deferrals_seen.to_string().as_str(),
                deferrals_seen_incl.to_string().as_str(),
            ]).unwrap();

            i_total += 1;
            i_active += cr.exp_row.choice.view().is_nonempty() as u32;
            largest_menu_seen = largest_menu_seen_incl;
            alternatives_seen = alternatives_seen_incl;
            deferrals_seen = deferrals_seen_incl;
        }
    }
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

                cr.choice.view().is_nonempty()
                    && (cr.choice != model_choice)
            }).collect();
            let score = diff.iter().map(|d| *d as u32).sum();
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
                    hm_total: best_score.unwrap(),
                }
            ).collect(),
        }
    }).collect()
}

fn main() {
    let (alternatives, subjects) = load_stdin();
    assert_eq!(alternatives.len(), 5);
    let subjects_solved = process(&alternatives, &subjects);
    assert_eq!(subjects_solved.len(), subjects.len());
    write_stdout(&alternatives, &subjects_solved);
}
