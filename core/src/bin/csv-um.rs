extern crate csv;
extern crate prest;

use prest::alt_set::AltSet;

#[derive(Clone)]
struct ChoiceRow {
    menu : AltSet,
    choice : AltSet,
}

fn get_hm_score(subject : &prest::csv::Subject<(), ChoiceRow>) -> u32 {
    prest::linear_preorders::all(subject.alternatives.len() as u32).map(
        |p| subject.rows.iter().map(
            |cr|
                if cr.choice.view().is_empty() {
                    0
                } else {
                    if cr.choice == prest::model::preorder_maximization(&p, cr.menu.view()) {
                        0
                    } else {
                        1
                    }
                }
        ).sum()
    ).min().expect("impossible: no linear preorders")
}

impl prest::csv::FromRow for ChoiceRow {
    const COLUMN_NAMES : &'static[&'static str] = &["menu", "choice"];
    type ParseError = prest::csv::Void;
    fn from_row(alternatives : &mut Vec<String>, row : &[&str]) -> Result<ChoiceRow, prest::csv::Void> {
        match row {
            [menu_s, choice_s] => Ok(ChoiceRow{
                menu: prest::csv::FromCell::from_cell(alternatives, menu_s)?,
                choice: prest::csv::FromCell::from_cell(alternatives, choice_s)?,
            }),

            _ => panic!("from_row received inconsistent number of columns"),  // this is impossible
        }
    }
}

fn main() {
    let mut csv_out = csv::Writer::from_writer(std::io::stdout());
    csv_out.write_record(&[
        "subject",
        "hm",
    ]).unwrap();
    for subject_r in prest::csv::read_subjects(std::io::stdin(), "subject").unwrap() {
        let subject : prest::csv::Subject<(), ChoiceRow> = subject_r.unwrap();
        assert_eq!(subject.alternatives.len(), 5);
        let hm_score = get_hm_score(&subject);
        csv_out.write_record(&[
            subject.name.as_str(),
            hm_score.to_string().as_str()
        ]).unwrap();
    }
}
