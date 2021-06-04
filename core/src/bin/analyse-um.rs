extern crate csv;
extern crate prest;

use prest::alt::Alt;
use prest::alt_set::AltSet;
use std::collections::HashMap;

struct ExpObservation {
    position : u32,
    menu : AltSet,
    choice : AltSet,
}

struct ExpSubject {
    name : String,
    choices : Vec<ExpObservation>,
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

fn load_stdin() -> (Vec<String>, Vec<ExpSubject>) {
    let mut alternatives : Vec<String> = Vec::new();
    let mut subjects : HashMap<String, ExpSubject> = HashMap::new();

    let mut csv_reader = csv::Reader::from_reader(std::io::stdin());
    assert_eq!(csv_reader.headers().unwrap(), vec!["position", "subject", "menu", "choice"]);

    for result in csv_reader.records() {
        let record = result.unwrap();
        let fields : Vec<&str> = record.iter().collect();
        match fields.as_slice() {
            [s_pos, s_subject, s_menu, s_choice] => {
                let subject = subjects.entry(String::from(*s_subject)).or_insert(
                    ExpSubject{ name: String::from(*s_subject), choices: Vec::new() }
                );

                subject.choices.push(
                    ExpObservation {
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

fn main() {
    let (alternatives, subjects) = load_stdin();
    assert_eq!(alternatives.len(), 5);

    for subject in subjects {
        for p in prest::linear_preorders::all(alternatives.len() as u32) {
            for cr in &subject.choices {
                let model_choice = prest::model::preorder_maximization(
                    &p, cr.menu.view()
                );


            }
        }
    }
}
