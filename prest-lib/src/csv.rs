use std::io::{Read,Write};
use std::marker::PhantomData;
use std::collections::HashSet;

use crate::alt::Alt;
use crate::alt_set::AltSet;

pub trait FromRow {
    type ParseError;
    const COLUMN_NAMES : &'static[&'static str];
    fn from_row(alternatives : &mut Vec<String>, row : &[&str]) -> Result<Self, Self::ParseError>
        where Self : Sized;
}

pub trait ToRow {
    const COLUMN_NAMES : &'static[&'static str];
    fn to_row(&self, alternatives : &[&str]) -> Vec<String>;
}

pub trait FromCell {
    type ParseError;
    fn from_cell(alternatives : &mut Vec<String>, cell : &str) -> Result<Self, Self::ParseError>
        where Self : Sized;
}

pub trait ToCell {
    fn to_cell(&self, alternatives : &[String]) -> String;
}

#[derive(Debug)]
pub enum Void {}

impl Void {
    pub fn into<T>(self) -> T {
        match self {}
    }
}

impl ToRow for () {
    const COLUMN_NAMES : &'static[&'static str] = &[];
    fn to_row(&self, _alternatives : &[&str]) -> Vec<String> {
        Vec::new()
    }
}

impl FromRow for () {
    type ParseError = Void;
    const COLUMN_NAMES : &'static[&'static str] = &[];
    fn from_row(_alternatives : &mut Vec<String>, _row : &[&str]) -> Result<(), Void> {
        Ok(())
    }
}

impl FromCell for AltSet {
    type ParseError = Void;
    fn from_cell(alternatives : &mut Vec<String>, cell : &str) -> Result<AltSet, Void> {
        let ctrim = cell.trim();

        if ctrim == "" {
            return Ok(AltSet::empty());
        }

        Ok(ctrim.split(',').map(|alt_s| {
            let alt_trimmed = alt_s.trim();
            match alternatives.iter().position(|s| s == alt_trimmed) {
                None => {
                    let i = alternatives.len();
                    alternatives.push(String::from(alt_trimmed));
                    Alt(i as u32)
                }

                Some(i) => Alt(i as u32)
            }
        }).collect())
    }
}

impl ToCell for AltSet {
    fn to_cell(&self, alternatives : &[String]) -> String {
        self.view().into_iter().map(
            |Alt(i)| alternatives[i as usize].as_str()
        ).collect::<Vec<&str>>().join(",")
    }
}

pub struct Subject<Sub, Row> {
    pub name : String,
    pub alternatives : Vec<String>,
    pub data : Sub,
    pub rows : Vec<Row>,
}

#[derive(Debug)]
pub enum Error<SubE, RowE> {
    IO(std::io::Error),
    Csv(csv::Error),
    ColumnOverlap(String),
    ParseSub(SubE),
    ParseRow(RowE),
    RowTooShort(usize),
    SubjectDiscontiguous(String),
    SubjectInconsistent(String),
    NoNameColumn(String),
}

impl<S,R> From<csv::Error> for Error<S,R> { fn from(e : csv::Error) -> Self { Error::Csv(e) } }
impl<S,R> From<std::io::Error> for Error<S,R> { fn from(e : std::io::Error) -> Self { Error::IO(e) } }

pub struct Columns<T> {
    indices : Vec<usize>,
    phantom : PhantomData<T>,
}

pub struct IterSubjects<R, Sub, Row> {
    csv : csv::StringRecordsIntoIter<R>,
    ix_name : usize,
    cols_sub : Columns<Sub>,
    cols_row : Columns<Row>,
    closed_subjects : HashSet<String>,
    current_subject : Option<Subject<Sub, Row>>,
    alternatives : Vec<String>,
}

impl<R : Read, Sub : FromRow+Eq, Row : FromRow>
    Iterator for IterSubjects<R, Sub, Row>
{
    type Item = Result<Subject<Sub, Row>, Error<Sub::ParseError, Row::ParseError>>;

    fn next(&mut self) -> Option<Result<Subject<Sub, Row>, Error<Sub::ParseError, Row::ParseError>>> {
        loop {
            let csv_row = match self.csv.next() {
                None => return match self.current_subject.take() {
                    Some(mut subj) => {
                        subj.alternatives = self.alternatives.clone();
                        Some(Ok(subj))
                    },  // last subject
                    None => None, // EOF
                },
                Some(r) => match r {
                    Ok(row) => row,
                    Err(e) => return Some(Err(Error::Csv(e))),
                },
            };

            let subject_name = match csv_row.get(self.ix_name) {
                None => return Some(Err(Error::RowTooShort(self.ix_name))),
                Some(name) => name,
            };

            let sub_data = {
                let mut row : Vec<&str> = Vec::new();
                for i in &self.cols_sub.indices {
                    match csv_row.get(*i) {
                        None => return Some(Err(Error::RowTooShort(*i))),
                        Some(cell) => row.push(cell),
                    }
                }

                match Sub::from_row(&mut self.alternatives, &row) {
                    Err(e) => return Some(Err(Error::ParseSub(e))),
                    Ok(data) => data,
                }
            };

            let row = {
                let mut row : Vec<&str> = Vec::new();
                for i in &self.cols_row.indices {
                    match csv_row.get(*i) {
                        None => return Some(Err(Error::RowTooShort(*i))),
                        Some(cell) => row.push(cell),
                    }
                }

                match Row::from_row(&mut self.alternatives, &row) {
                    Err(e) => return Some(Err(Error::ParseRow(e))),
                    Ok(row) => row,
                }
            };

            match self.current_subject {
                None => {
                    // we're just starting out, don't check anything
                    self.current_subject = Some(Subject {
                        name: String::from(subject_name),
                        data: sub_data,
                        rows: vec![row],
                        alternatives: Vec::new(), // we'll set this at the end
                    });
                }

                Some(ref mut subj) => {
                    if subject_name == subj.name {
                        // if it's the same subject,
                        // check that it's got the same subject-specific data
                        if sub_data != subj.data {
                            return Some(Err(Error::SubjectInconsistent(subj.name.clone())));
                        }

                        // and add the current row
                        subj.rows.push(row);
                    } else {
                        // if it's a new subject,
                        // check that we have not seen it yet
                        if self.closed_subjects.contains(subject_name) {
                            return Some(Err(Error::SubjectDiscontiguous(String::from(subject_name))));
                        }

                        // and then mark the current one as old and closed
                        self.closed_subjects.insert(subj.name.clone());

                        // create a new record for the new subject
                        let mut result = Subject {
                            name: String::from(subject_name),
                            data: sub_data,
                            rows: vec![row],
                            alternatives: Vec::new(),
                        };

                        // swap it for the old one
                        std::mem::swap(subj, &mut result);

                        // copy the right list of alternatives
                        //
                        // in theory, this could contain alternatives
                        // not found in the original subject
                        // if they are present in the first row of the new subject
                        // but that should be okay
                        result.alternatives = self.alternatives.clone();

                        // and return the old one
                        return Some(Ok(result));
                    }
                }
            }
        }
    }
}

pub fn read_subjects<R, Sub, Row>(rdr : R, subj_name_column : &str)
    -> Result<IterSubjects<R, Sub, Row>, Error<Sub::ParseError, Row::ParseError>>
    where R : Read, Sub : FromRow, Row : FromRow
{
    let mut csv_reader = csv::Reader::from_reader(rdr);
    let headers = csv_reader.headers()?;

    // split the columns between subject-specific and row-specific structs
    let mut ixs_sub = Vec::new();
    let mut ixs_row = Vec::new();

    for (ix, hdr) in headers.iter().enumerate() {
        match (Sub::COLUMN_NAMES.contains(&hdr), Row::COLUMN_NAMES.contains(&hdr)) {
            (true, false) => ixs_sub.push(ix),
            (false, true) => ixs_row.push(ix),
            (true, true) => return Err(Error::ColumnOverlap(
                String::from(hdr)
            )),
            (false, false) => (),  // unknown column, silently ignore
        }
    }

    let ix_name = match headers.iter().position(|h| h == subj_name_column) {
        None => return Err(Error::NoNameColumn(String::from(subj_name_column))),
        Some(ix) => ix,
    };

    Ok(IterSubjects {
        csv: csv_reader.into_records(),
        ix_name,
        cols_sub: Columns {
            indices: ixs_sub,
            phantom: PhantomData,
        },
        cols_row: Columns {
            indices: ixs_row,
            phantom: PhantomData,
        },
        closed_subjects: HashSet::new(),
        current_subject: None,
        alternatives: Vec::new(),
    })
}

pub struct Writer<W : Write, Sub, Row> {
    csv : csv::Writer<W>,
    phantom_sub : PhantomData<Sub>,
    phantom_row : PhantomData<Row>,
}

impl<W : Write, Sub : ToRow, Row : ToRow> Writer<W, Sub, Row> {
    pub fn new(w : W) -> Result<Self, csv::Error> {
        let mut csv = csv::Writer::from_writer(w);
        csv.write_record({
            let mut header : Vec<&'static str> = vec!["subject"];
            header.extend_from_slice(Sub::COLUMN_NAMES);
            header.extend_from_slice(Row::COLUMN_NAMES);
            header
        })?;

        Ok(Writer{
            csv,
            phantom_sub : PhantomData,
            phantom_row : PhantomData,
        })
    }

    pub fn write(&mut self, subject : Subject<Sub, Row>) -> Result<(), csv::Error> {
        let alternatives : Vec<&str> = subject.alternatives.iter().map(String::as_str).collect();
        for row in subject.rows {
            self.csv.write_record({
                let mut csv_row : Vec<String> = vec![subject.name.clone()];
                csv_row.append(&mut subject.data.to_row(&alternatives));
                csv_row.append(&mut row.to_row(&alternatives));
                csv_row
            })?;
        }

        Ok(())
    }
}

pub fn write_subjects<W, Sub, Row, I>(wtr : W, subjects : I)
    -> Result<(), csv::Error>
    where W : Write, I : Iterator<Item=Subject<Sub, Row>>, Sub : ToRow, Row : ToRow
{
    let mut csv = Writer::new(wtr)?;
    for subject in subjects {
        csv.write(subject)?;
    }
    Ok(())
}
