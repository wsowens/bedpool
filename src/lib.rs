use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::num::{ParseFloatError, ParseIntError};

pub struct BedFile {
    pub lineno: usize,
    pub last: Option<String>,
    pub filename: String,
    file: io::BufReader<File>,
    bufsize: usize, // hint for how big the buffer should be
    at_eof: bool,
}

pub enum BedError {
    IO(io::Error),
    File(String, io::Error),
    Parse(String, usize, String),
}

impl fmt::Display for BedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BedError::IO(e) => write!(f, "An IO error occurred:\n  {}", e),
            BedError::File(filename, e) => {
                write!(f, "An IO error occurred with file '{}':\n  {}", filename, e)
            }
            BedError::Parse(filename, lineno, msg) => write!(
                f,
                "A parse error occurred on line {} of file '{}':\n  {}",
                lineno, filename, msg
            ),
        }
    }
}

trait ToBedErr {
    fn bed_error(self, bf: &BedFile) -> BedError;
}

impl ToBedErr for io::Error {
    fn bed_error(self, bf: &BedFile) -> BedError {
        BedError::File(bf.filename.clone(), self)
    }
}

impl ToBedErr for ParseIntError {
    fn bed_error(self, bf: &BedFile) -> BedError {
        BedError::Parse(
            bf.filename.clone(),
            bf.lineno,
            format!("expected integer, but {}", self),
        )
    }
}

impl ToBedErr for ParseFloatError {
    fn bed_error(self, bf: &BedFile) -> BedError {
        BedError::Parse(
            bf.filename.clone(),
            bf.lineno,
            format!("expected float, but {}", self),
        )
    }
}

trait ToBedResult<T> {
    fn bed_result(self: Self, bf: &BedFile) -> Result<T, BedError>;
}

impl<T, E: ToBedErr> ToBedResult<T> for Result<T, E> {
    fn bed_result(self: Self, bf: &BedFile) -> Result<T, BedError> {
        self.map_err(|e| e.bed_error(bf))
    }
}

impl BedFile {
    pub fn new(fname: &str) -> Result<Self, BedError> {
        let filename = fname.to_string();
        let file = match File::open(fname) {
            Err(io_error) => {
                return Err(BedError::File(filename, io_error));
            }
            Ok(f) => io::BufReader::new(f),
        };
        Ok(BedFile {
            lineno: 0,
            last: None,
            filename,
            file,
            bufsize: 32,
            at_eof: false,
        })
    }

    pub fn next(&mut self) -> Result<Option<BedRecord>, BedError> {
        if self.at_eof {
            return Ok(None);
        }
        let mut buffer = String::with_capacity(self.bufsize);
        self.bufsize = self.file.read_line(&mut buffer).bed_result(self)?;
        if self.bufsize == 0 {
            self.at_eof = true;
            return Ok(None);
        }
        self.lineno += 1;
        self.last = Some(buffer);

        // annotate the BedRecord
        if let Some(ref line) = self.last {
            let parts: Vec<&str> = line.split_ascii_whitespace().take(6).collect();
            if parts.len() < 6 {
                return Err(BedError::Parse(
                    self.filename.clone(),
                    self.lineno,
                    format!("expected at least 6 columns, got {}", parts.len()),
                ));
            }
            let chrom = parts[0];
            let start = parts[1].parse().bed_result(self)?;
            let end = parts[2].parse().bed_result(self)?;
            let ratio = parts[3].parse().bed_result(self)?;
            let meth = parts[4].parse().bed_result(self)?;
            let cov = parts[5].parse().bed_result(self)?;
            Ok(Some(BedRecord {
                coords: BedCoords { chrom, start, end },
                ratio,
                meth,
                cov,
            }))
        } else {
            unreachable!()
        }
    }
}

pub fn sync2(mut file1: BedFile, mut file2: BedFile) -> Result<(), BedError> {
    // assume the files are unitialized
    let mut maybe_rec1 = file1.next()?;
    let mut maybe_rec2 = file2.next()?;
    loop {
        match (maybe_rec1.as_ref(), maybe_rec2.as_ref()) {
            (Some(rec1), Some(rec2)) => match rec1.coords.cmp(&rec2.coords) {
                Ordering::Equal => {
                    let meth = rec1.meth + rec2.meth;
                    let cov = rec1.cov + rec2.cov;
                    let ratio = meth / cov;
                    println!(
                        "{}",
                        BedRecord {
                            ratio,
                            meth,
                            cov,
                            ..maybe_rec1.unwrap()
                        }
                    );
                    maybe_rec1 = file1.next()?;
                    maybe_rec2 = file2.next()?;
                }
                Ordering::Less => {
                    println!("{}", rec1);
                    maybe_rec1 = file1.next()?;
                }
                Ordering::Greater => {
                    println!("{}", rec2);
                    maybe_rec2 = file2.next()?;
                }
            },
            (Some(rec), None) | (None, Some(rec)) => {
                println!("{}", rec);
                maybe_rec1 = file1.next()?;
                maybe_rec2 = file2.next()?;
            }
            (None, None) => {
                break;
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct BedRecord<'a> {
    coords: BedCoords<'a>,
    ratio: f32,
    meth: f32,
    cov: f32,
}

impl<'a> fmt::Display for BedRecord<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}\t{}\t{}",
            self.coords.chrom, self.coords.start, self.coords.end, self.ratio, self.meth, self.cov,
        )
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct BedCoords<'a> {
    chrom: &'a str,
    start: u64,
    end: u64,
}
