use std::result;

use std::fmt;
use std::io::{Read,Write};
use alt::{Alt};
use alt_set::{AltSet};
use codec::{self,Packed,Encode,Decode};
use rpc_common::{Subject};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Request {
    GeneralIntegrity{ subjects: Vec<Packed<Subject>> },
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        use self::Request::*;
        Ok(match Decode::decode(f)? {
            0u8 => GeneralIntegrity{ subjects: Decode::decode(f)? },
            _ => panic!("wrong request tag"),
        })
    }
}

enum IssueDescription {
    RepeatedMenu(AltSet),
    ChoiceNotInMenu(AltSet, Alt),
}

impl Encode for IssueDescription {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        use self::IssueDescription::*;
        match self {
            RepeatedMenu(ref menu) => {
                (0u8, menu).encode(f)
            }
            ChoiceNotInMenu(ref menu, ref alt) => {
                (1u8, menu, alt).encode(f)
            }
        }
    }
}

pub struct Issue {
    subject_name : String,
    description : IssueDescription,
}

impl Encode for Issue {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.subject_name, &self.description).encode(f)
    }
}

pub struct Response {
    issues : Vec<Issue>,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.issues.encode(f)
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

fn find_issues(subject : &Subject) -> Vec<IssueDescription> {
    let mut issues = Vec::new();

    // repeated menus
    {
        let mut menu_counts = HashMap::new();
        for row in &subject.choices {
            *menu_counts.entry(&row.menu).or_insert(0) += 1;
        }

        for (menu, repetitions) in menu_counts.into_iter() {
            if repetitions > 1 {
                issues.push(IssueDescription::RepeatedMenu(menu.clone()));
            }
        }
    }

    for row in &subject.choices {
        // choice not in menu
        for choice in row.choice.view() {
            if !row.menu.view().contains(choice) {
                issues.push(IssueDescription::ChoiceNotInMenu(
                    row.menu.clone(),
                    choice,
                ));
            }
        }

        // default choice not in menu
        if let Some(choice) = row.default {
            if !row.menu.view().contains(choice) {
                issues.push(IssueDescription::ChoiceNotInMenu(
                    row.menu.clone(),
                    choice,
                ));
            }
        }
    }

    issues
}

pub fn run(req : Request) -> Result<Response> {
    use self::Request::*;
    match req {
        GeneralIntegrity{subjects} => {
            let mut issues = Vec::new();

            for Packed(subject) in subjects {
                let subject_issues = find_issues(&subject);
                issues.extend(subject_issues.into_iter().map(
                    |idesc| Issue {
                        subject_name: subject.name.clone(),
                        description: idesc,
                    }
                ));
            }

            Ok(Response{issues})
        }
    }
}
