use std::io::Read;
use std::marker::PhantomData;
use std::collections::HashSet;

pub trait FromCsv {
    type ParseError;
    fn column_names() -> Vec<&'static str>;
    fn from_row(row : &[&str]) -> Result<Self, Self::ParseError>
        where Self : Sized;
}

pub struct Subject<Sub, Row> {
    name : String,
    data : Sub,
    rows : Vec<Row>,
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
}

impl<R : Read, Sub : FromCsv+Eq, Row : FromCsv>
    Iterator for IterSubjects<R, Sub, Row>
{
    type Item = Result<Subject<Sub, Row>, Error<Sub::ParseError, Row::ParseError>>;

    fn next(&mut self) -> Option<Result<Subject<Sub, Row>, Error<Sub::ParseError, Row::ParseError>>> {
        loop {
            let csv_row = match self.csv.next() {
                None => return match self.current_subject.take() {
                    Some(subj) => Some(Ok(subj)),  // last subject
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

                match Sub::from_row(&row) {
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

                match Row::from_row(&row) {
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
                        rows: Vec::new(),
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
                            rows: Vec::new(),
                        };

                        // swap it for the old one
                        std::mem::swap(subj, &mut result);

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
    where R : Read, Sub : FromCsv, Row : FromCsv
{
    let mut csv_reader = csv::Reader::from_reader(rdr);
    let headers = csv_reader.headers()?;

    // split the columns between subject-specific and row-specific structs
    let cols_sub = Sub::column_names();
    let cols_row = Row::column_names();
    let mut ixs_sub = Vec::new();
    let mut ixs_row = Vec::new();

    for (ix, hdr) in headers.iter().enumerate() {
        match (cols_sub.contains(&hdr), cols_row.contains(&hdr)) {
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
    })
}
