use crate::alt::Alt;
use crate::alt_set::AltSet;
use std::io::{Read,Write};
use crate::codec::{self,Decode,Encode};

#[derive(Clone, Debug)]
pub struct ChoiceRow {
    pub menu    : AltSet,
    pub default : Option<Alt>,
    pub choice  : AltSet,
}

impl Encode for ChoiceRow {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (self.menu.view(), self.default, self.choice.view()).encode(f)
    }
}

impl Decode for ChoiceRow {
    fn decode<R : Read>(f : &mut R) -> codec::Result<ChoiceRow> {
        Ok(ChoiceRow {
            menu: Decode::decode(f)?,
            default: Decode::decode(f)?,
            choice: Decode::decode(f)?,
        })
    }
}

#[macro_export]
macro_rules! choices {
    ($([$($x:expr_2021),*] -> [$($y:expr_2021),*]),*) => {vec![
        $(ChoiceRow{
            menu: alts![$($x),*],
            default: None,
            choice: alts![$($y),*],
        }),*
    ]}
}

#[derive(Debug, Clone)]
pub struct Subject {
    pub name : String,
    pub alternatives : Vec<String>,
    pub choices : Vec<ChoiceRow>,
}

impl Subject {
    pub fn drop_deferrals(&self, do_drop : bool) -> Self {
        if !do_drop {
            self.clone()
        } else {
            Subject {
                name: self.name.clone(),
                alternatives: self.alternatives.clone(),
                choices: self.choices.iter().filter(
                    |cr| cr.choice.view().is_nonempty()
                ).cloned().collect(),
            }
        }
    }
}

impl Encode for Subject {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (&self.name, &self.alternatives, &self.choices).encode(f)
    }
}

impl Decode for Subject {
    fn decode<R : Read>(f : &mut R) -> codec::Result<Subject> {
        Ok(Subject {
            name: Decode::decode(f)?,
            alternatives: Decode::decode(f)?,
            choices: Decode::decode(f)?,
        })
    }
}

pub trait Log {
    fn log(&mut self, level : LogLevel, message : String);
    fn progress(&mut self, position : u32);

    fn debug(&mut self, msg : String) {
        self.log(LogLevel::Debug, msg)
    }

    fn info(&mut self, msg : String) {
        self.log(LogLevel::Info, msg)
    }

    fn warn(&mut self, msg : String) {
        self.log(LogLevel::Warning, msg)
    }

    fn error(&mut self, msg : String) {
        self.log(LogLevel::Error, msg)
    }
}

#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Encode for LogLevel {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        use self::LogLevel::*;
        match *self {
            Debug   => 0u8.encode(f),
            Info    => 1u8.encode(f),
            Warning => 2u8.encode(f),
            Error   => 3u8.encode(f),
        }
    }
}
