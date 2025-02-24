use std::result;
use std::fmt;
use std::io::{Read,Write};
use std::collections::HashMap;

use crate::alt::{Alt};
use crate::alt_set::{AltSet};
use crate::codec::{self,Packed,Encode,Decode};
use crate::common::{Subject};

#[derive(Debug)]
pub struct Request {
    subject : Packed<Subject>
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            subject: Decode::decode(f)?
        })
    }
}

enum Issue {
    RepeatedMenu(AltSet),
    ChoiceNotInMenu(AltSet, Alt),
}

impl Encode for Issue {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        use self::Issue::*;
        match self {
            RepeatedMenu(menu) => {
                (0u8, menu).encode(f)
            }
            ChoiceNotInMenu(menu, alt) => {
                (1u8, menu, alt).encode(f)
            }
        }
    }
}

pub struct Response {
    subject_name : String,
    issues : Vec<Issue>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.subject_name, &self.issues).encode(f)
    }
}

pub enum IntegrityError {
}

impl Encode for IntegrityError {
    fn encode<W : Write>(&self, _f : &mut W) -> codec::Result<()> {
        match *self { }
    }
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, _f : &mut fmt::Formatter) -> fmt::Result {
        match *self { }
    }
}

pub type Result<T> = result::Result<T, IntegrityError>;

fn find_issues(subject : &Subject) -> Vec<Issue> {
    let mut issues = Vec::new();

    // repeated menus
    {
        let mut menu_counts = HashMap::new();
        for row in &subject.choices {
            *menu_counts.entry(&row.menu).or_insert(0) += 1;
        }

        for (menu, repetitions) in menu_counts.into_iter() {
            if repetitions > 1 {
                issues.push(Issue::RepeatedMenu(menu.clone()));
            }
        }
    }

    for row in &subject.choices {
        // choice not in menu
        for choice in row.choice.view() {
            if !row.menu.view().contains(choice) {
                issues.push(Issue::ChoiceNotInMenu(
                    row.menu.clone(),
                    choice,
                ));
            }
        }

        // default choice not in menu
        if let Some(choice) = row.default {
            if !row.menu.view().contains(choice) {
                issues.push(Issue::ChoiceNotInMenu(
                    row.menu.clone(),
                    choice,
                ));
            }
        }
    }

    issues
}

pub fn run(req : Request) -> Result<Response> {
    let Packed(subject) = req.subject;
    let issues = find_issues(&subject);
    Ok(Response{
        subject_name: subject.name,
        issues
    })
}
