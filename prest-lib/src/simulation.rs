use std::result;
use std::fmt;
use std::io::{Read,Write};
use std::iter::FromIterator;
use rand::Rng;
use rand::seq::IndexedRandom;

use crate::model;
use crate::common::{ChoiceRow,Subject};
use crate::codec::{Encode,Decode,Packed,self};
use crate::alt_set::{AltSet,AltSetView};
use crate::alt::Alt;

pub type Menu = AltSet;

#[derive(Debug)]
pub enum MenuGenerator {
    Exhaustive,
    SampleWithReplacement(u32),
    Copycat(Packed<Subject>),
    Binary,
}

impl Decode for MenuGenerator {
    fn decode<R : Read>(f : &mut R) -> codec::Result<MenuGenerator> {
        use self::MenuGenerator::*;

        Ok(match Decode::decode(f)? {
            0u8 => Exhaustive,
            1u8 => SampleWithReplacement(Decode::decode(f)?),
            2u8 => Copycat(Decode::decode(f)?),
            3u8 => Binary,
            _ => Err(codec::Error::BadEnumTag)?,
        })
    }
}

impl MenuGenerator {
    pub fn generate<R : Rng>(&self, rng : &mut R, alt_count : u32) -> Vec<(AltSet, Option<Alt>)> {
        use self::MenuGenerator::*;
        match *self {
            Exhaustive => {
                // does not include the empty set
                AltSet::powerset(alt_count).map(|m| (m, None)).collect()
            }

            SampleWithReplacement(menu_count) => {
                // does not generate the empty set
                (0..menu_count).map(
                    |_| (AltSet::rand_nonempty(rng, alt_count), None)
                ).collect()
            }

            Copycat(Packed(ref subject)) => {
                subject.choices.iter().map(
                    |cr| (cr.menu.clone(), cr.default)
                ).collect()
            },

            Binary => {
                Alt::distinct_pairs(alt_count).map(
                    |(i, j)| (AltSet::from_iter(&[i, j]), None)
                ).collect()
            }
        }
    }
}

#[derive(Debug)]
pub struct GenMenus {
    pub generator : MenuGenerator,
    pub defaults : bool,
}

impl GenMenus {
    fn generate<R : Rng>(&self, rng : &mut R, alt_count : u32) -> Vec<(AltSet, Option<Alt>)> {
        let menus = self.generator.generate(rng, alt_count);

        if self.defaults {
            menus.into_iter().map(
                |(m, _)| {
                    let alts = Vec::from_iter(m.view().into_iter());
                    let default = alts.choose(rng).expect("empty menu").clone();
                    (m, Some(default))
                }
            ).collect()
        } else {
            menus
        }
    }
}

impl Decode for GenMenus {
    fn decode<R : Read>(f : &mut R) -> codec::Result<GenMenus> {
        Ok(GenMenus {
            generator: Decode::decode(f)?,
            defaults: Decode::decode(f)?,
        })
    }
}

#[derive(Debug)]
pub enum GenChoices {
    Instance(model::Instance),
    Uniform {
        forced_choice: bool,
        multiple_choice: bool,
    },
}

impl Decode for GenChoices {
    fn decode<R : Read>(f : &mut R) -> codec::Result<GenChoices> {
        Ok(match Decode::decode(f)? {
            0u8 => GenChoices::Instance({
                let bytes : Vec<u8> = Decode::decode(f)?;
                codec::decode_from_memory(&bytes)?
            }),
            1u8 => GenChoices::Uniform {
                forced_choice: Decode::decode(f)?,
                multiple_choice: Decode::decode(f)?,
            },
            _ => Err(codec::Error::BadEnumTag)?,
        })
    }
}

impl GenChoices {
    fn generate<R : Rng>(&self, rng : &mut R, alt_count : u32, menu : AltSetView, default : Option<Alt>) -> AltSet {
        assert!(menu.is_nonempty());
        use self::GenChoices::*;

        match *self {
            Uniform{forced_choice, multiple_choice} => {
                // deferral is an extra alternative
                let defer : Alt = Alt(alt_count);

                let feasible = {
                    let mut feasible = Vec::from_iter(menu.iter());
                    if !forced_choice {
                        feasible.push(defer);
                    }
                    feasible
                };

                let choice = feasible.choose(rng).unwrap().clone(); // contains at least deferral

                if choice == defer {
                    if let Some(alt) = default {
                        AltSet::singleton(alt)
                    } else {
                        AltSet::empty()
                    }
                } else if multiple_choice {
                    // nonempty subset of menu
                    AltSet::rand_nonempty(rng, menu.size()).view().iter().map(
                        |Alt(i)| feasible[i as usize]
                    ).collect()
                } else {
                    AltSet::singleton(choice)
                }
            }

            Instance(ref inst) => {
                inst.choice(menu, None)
            }
        }
    }
}

#[derive(Debug)]
pub struct Request {
    name : String,
    alternatives : Vec<String>,
    gen_menus : GenMenus,
    gen_choices : GenChoices,
    preserve_deferrals : bool,
}

impl Decode for Request {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Request> {
        Ok(Request {
            name: Decode::decode(f)?,
            alternatives: Decode::decode(f)?,
            gen_menus: Decode::decode(f)?,
            gen_choices: Decode::decode(f)?,
            preserve_deferrals: Decode::decode(f)?,
        })
    }
}

pub struct Response {
    subject : Packed<Subject>,
    observation_count : u32,
}

impl Encode for Response {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.subject, self.observation_count).encode(f)
    }
}

pub enum Error {
}

impl Encode for Error {
    fn encode<W : Write>(&self, _f : &mut W) -> codec::Result<()> {
        match *self { }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _f : &mut fmt::Formatter) -> fmt::Result {
        match *self { }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn run<R : Rng>(rng : &mut R, request : Request) -> Result<Response> {
    let alt_count = request.alternatives.len() as u32;

    let choices : Vec<ChoiceRow> = match request.gen_menus.generator {
        MenuGenerator::Copycat(Packed(ref subj)) => subj.choices.iter().map(
            |cr| ChoiceRow {
                menu: cr.menu.clone(),
                default: cr.default.clone(),
                choice: if request.preserve_deferrals
                    && cr.choice.view().is_empty() {
                        AltSet::empty()
                    } else {
                        request.gen_choices.generate(
                            rng, alt_count, cr.menu.view(), cr.default
                        )
                    }
            }
        ).collect(),

        _ => request.gen_menus.generate(rng, alt_count).into_iter().map(
            // we use this order of ChoiceRow fields
            // because we first need to generate the choice
            // and only then pass the ownership of the menu
            |(menu, default)| ChoiceRow {
                choice: request.gen_choices.generate(rng, alt_count, menu.view(), default),
                menu,
                default,
            }
        ).collect(),
    };

    let name = match request.gen_menus.generator {
        MenuGenerator::Copycat(subject_packed)
            => format!("{}{}", subject_packed.unpack().name, request.name),
        _ => request.name,
    };

    Ok(Response {
        observation_count: choices.len() as u32,
        subject: Packed(Subject {
            name,
            alternatives: request.alternatives,
            choices,
        })
    })
}
