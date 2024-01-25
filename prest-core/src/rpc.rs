use std;
use std::io::{Read,Write,Stdout,Stdin,BufReader,BufWriter};
use std::result::Result;
use std::fmt::Display;

use prest::codec::{self,Encode,Decode};
use prest::common::{Log,LogLevel};
use prest::{estimation,simulation,consistency,experiment_stats,budgetary,integrity,instviz};

#[derive(Debug)]
pub enum ActionRequest {
    InstViz(instviz::Request),
    IntegrityCheck(integrity::Request),
    BudgetaryConsistency(budgetary::consistency::Request),
    Summary(experiment_stats::Request),
    SetRngSeed(Vec<u8>),
    Simulation(simulation::Request),
    ConsistencyDeterministic(consistency::deterministic::Request),
    ConsistencyStochastic(consistency::stochastic::Request),
    TupleIntransMenus(consistency::deterministic::Request),
    TupleIntransAlts(consistency::deterministic::Request),
    Estimation(estimation::Request),
    Echo(String),
    Crash(String),
    Fail(String),
    Quit,
}

impl Decode for ActionRequest {
    fn decode<R : Read>(f : &mut R) -> codec::Result<ActionRequest> {
        use self::ActionRequest::*;

        let tag : String = Decode::decode(f)?;
        match tag.as_str() {
            "instviz" => Ok(InstViz(Decode::decode(f)?)),
            "budgetary-consistency" => Ok(BudgetaryConsistency(Decode::decode(f)?)),
            "summary" => Ok(Summary(Decode::decode(f)?)),
            "set-rng-seed" => Ok(SetRngSeed(Decode::decode(f)?)),
            "simulation" => Ok(Simulation(Decode::decode(f)?)),
            "consistency-deterministic" => Ok(ConsistencyDeterministic(Decode::decode(f)?)),
            "consistency-stochastic" => Ok(ConsistencyStochastic(Decode::decode(f)?)),
            "tuple-intrans-menus" => Ok(TupleIntransMenus(Decode::decode(f)?)),
            "tuple-intrans-alts" => Ok(TupleIntransAlts(Decode::decode(f)?)),
            "estimation" => Ok(Estimation(Decode::decode(f)?)),
            "integrity-check" => Ok(IntegrityCheck(Decode::decode(f)?)),
            "echo" => Ok(Echo(Decode::decode(f)?)),
            "crash" => Ok(Crash(Decode::decode(f)?)),
            "fail" => Ok(Fail(Decode::decode(f)?)),
            "quit" => Ok(Quit),
            _ => Err(codec::Error::BadEnumTag),
        }
    }
}

pub struct Error {
    error_message : String,
    error : Vec<u8>,
}

impl Encode for Error {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        self.error_message.encode(f)?;
        self.error.encode(f)
    }
}

impl<E : Encode + Display> From<E> for Error {
    fn from(e : E) -> Error {
        Error {
            error_message: format!("{}", e),
            error: codec::encode_to_memory(&e).unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct LogMessage {
    pub level : LogLevel,
    pub message : String,
}

impl Encode for LogMessage {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        (self.level, &self.message).encode(f)
    }
}

pub enum Message<Ans> {
    Progress(u32),
    Answer(Ans),
    Error(Error),
    Log(LogMessage),
}

impl<Ans : Encode> Encode for Message<Ans> {
    fn encode<W : Write>(&self, f : &mut W) -> codec::Result<()> {
        use self::Message::*;
        match self {
            &Progress(position) => (0u8, position).encode(f),
            &Answer(ref answer) => (1u8, answer).encode(f),
            &Error(ref error)   => (2u8, error).encode(f),
            &Log(ref log)       => (3u8, log).encode(f),
        }
    }
}

pub struct IO {
    stdin : BufReader<Stdin>,
    stdout : BufWriter<Stdout>,
}

impl IO {
    pub fn from_stdio() -> Self {
        IO {
            stdin: BufReader::new(std::io::stdin()),
            stdout: BufWriter::new(std::io::stdout()),
        }
    }

    pub fn read<T : Decode>(&mut self) -> codec::Result<T> {
        Decode::decode(&mut self.stdin)
    }

    pub fn write_result<T : Encode, E : Encode+Display>(&mut self, r : Result<T, E>)
        -> codec::Result<()>
    {
        match r {
            Ok(x) => Message::Answer(x).encode(&mut self.stdout)?,
            Err(e) => Message::Error::<()>(Error::from(e)).encode(&mut self.stdout)?,
        }
        self.stdout.flush()?;
        Ok(())
    }
}

pub struct Logger<'a> {
    io : &'a mut IO
}

impl<'a> Logger<'a> {
    pub fn new(io : &'a mut IO) -> Logger<'a> {
        Logger{ io }
    }
}

impl<'a> Log for Logger<'a> {
    fn log(&mut self, level : LogLevel, message : String) {
        Message::Log::<()>(LogMessage{ level, message }).encode(&mut self.io.stdout).unwrap();
        self.io.stdout.flush().unwrap();
    }

    fn progress(&mut self, position : u32) {
        Message::Progress::<()>(position).encode(&mut self.io.stdout).unwrap();
        self.io.stdout.flush().unwrap();
    }
}

pub struct DummyLogger;

impl Log for DummyLogger {
    fn log(&mut self, _level : LogLevel, _message : String) {}
    fn progress(&mut self, _position : u32) {}
}
