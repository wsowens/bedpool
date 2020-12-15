use bedpool::{sync2, BedFile};

extern crate clap;
use clap::{App, Arg};

fn show_error<T, E: std::fmt::Display>(foo: Result<T, E>) -> T {
    foo.unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    })
}

fn main() {
    let matches = App::new("bedpool")
        .version("0.1")
        .author("William Owens <wowens@ufl.edu>")
        .about("Pool 2 BED files from dmap2 together.")
        .arg(
            Arg::with_name("file1")
                .index(1)
                .help("First file to pool")
                .required(true),
        )
        .arg(
            Arg::with_name("file2")
                .index(2)
                .help("Second file to pool")
                .required(true),
        )
        .get_matches();

    let file1 = matches.value_of("file1").unwrap();
    let file1 = show_error(BedFile::new(file1));
    let file2 = matches.value_of("file2").unwrap();
    let file2 = show_error(BedFile::new(file2));

    show_error(sync2(file1, file2));
}
