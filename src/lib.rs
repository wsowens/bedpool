use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};

pub struct BedFile {
    pub lineno: usize,
    pub last: Option<String>,
    pub filename: String,
    file: io::BufReader<File>,
    bufsize: usize, // hint for how big the buffer should be
    at_eof: bool,
}

impl BedFile {
    pub fn new(filename: &str) -> io::Result<Self> {
        Ok(BedFile {
            lineno: 0,
            last: None,
            filename: filename.to_string(),
            file: io::BufReader::new(File::open(filename)?),
            bufsize: 32,
            at_eof: false,
        })
    }

    pub fn next(&mut self) -> io::Result<Option<BedRecord>> {
        if self.at_eof {
            return Ok(None);
        }
        let mut buffer = String::with_capacity(self.bufsize);
        self.bufsize = self.file.read_line(&mut buffer)?;
        if self.bufsize == 0 {
            self.at_eof = true;
            return Ok(None);
        }
        self.lineno += 1;
        self.last = Some(buffer);

        // annotate the BedRecord
        if let Some(ref line) = self.last {
            let mut parts = line.split_ascii_whitespace();
            let chrom = parts.next().expect("expected at least 5 fields");
            let start = parts
                .next()
                .expect("expected at least 5 fields")
                .parse()
                .unwrap();
            let end = parts
                .next()
                .expect("expected at least 5 fields")
                .parse()
                .unwrap();
            let meth = parts
                .next()
                .expect("expected at least 5 fields")
                .parse()
                .unwrap();
            let cov = parts
                .next()
                .expect("expected at least 5 fields")
                .parse()
                .unwrap();
            Ok(Some(BedRecord {
                coords: BedCoords { chrom, start, end },
                meth,
                cov,
            }))
        } else {
            unreachable!()
        }
    }
}


pub fn sync2(mut file1: BedFile, mut file2: BedFile) -> io::Result<()> {
    // assume the files are unitialized
    let mut maybe_rec1 = file1.next()?;
    let mut maybe_rec2 = file2.next()?;
    loop {
        match (maybe_rec1.as_ref(), maybe_rec2.as_ref()) {
            (Some(rec1), Some(rec2)) => match rec1.coords.cmp(&rec2.coords) {
                Ordering::Equal => {
                    println!(
                        "{}",
                        BedRecord {
                            meth: rec1.meth + rec2.meth,
                            cov: rec1.cov + rec2.cov,
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
    meth: u32,
    cov: u32,
}

impl<'a> fmt::Display for BedRecord<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}\t{}",
            self.coords.chrom, self.coords.start, self.coords.end, self.meth, self.cov,
        )
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct BedCoords<'a> {
    chrom: &'a str,
    start: u64,
    end: u64,
}
