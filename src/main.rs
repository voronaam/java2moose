extern crate classreader;
extern crate zip;
extern crate docopt;
extern crate serde;

use docopt::Docopt;
use serde::Deserialize;
use classreader::*;
use std::fs::File;

mod codecity;
mod freeminer;
mod melo;

const USAGE: &'static str = "
Java code to FreeMiner map analyzier.

Usage:
  javaminer [options] <source>...
  javaminer (-h | --help)

Options:
  -h --help      Show this screen.
  --map=<path>   Path to FreeMiner SQLite map.
  --melo=<melo>  Path to output MELO file.

Source can be one or more class or jar files.
";

#[derive(Debug, Deserialize)]
struct Args {
	flag_map: Option<String>,
    flag_melo: Option<String>,
	arg_source: Vec<String>
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    let mut classes: Vec<Class> = Vec::new();
    for f in args.arg_source {
        if f.ends_with("class") {
            process_class_file(&f, &mut classes);
        } else if f.ends_with("jar") {
            process_jar_file(&f, &mut classes);
        } else {
            println!("Ignoring unknown file type {}", f);
        }
    }
    if !classes.is_empty() && args.flag_map.is_some() {
        println!("======================================================");
        let root_pkg = codecity::build_packages(&classes);
        freeminer::write_to_freeminer(&args.flag_map.unwrap(), &root_pkg);
    }
    if !classes.is_empty() && args.flag_melo.is_some() {
        println!("=================== MELO =============================");
        let melo_file = &args.flag_melo.unwrap();
        println!("Generating melo output in {}", melo_file);
        let music = codecity::build_music(&classes);
        melo::write_to_file(&melo_file, &music);
    }
    println!("Done!");
}

fn process_class_file(file_name: &String, classes: &mut Vec<Class>) {
    println!("Loading class file {}", file_name);
    println!("======================================================");
    let class = ClassReader::new_from_path(&file_name).unwrap();
    codecity::process_class(&class);
    classes.push(class);
}

fn process_jar_file(file_name: &String, classes: &mut Vec<Class>) {
    let file = File::open(file_name).expect("couldn't find a file!");
    let mut zip = zip::ZipArchive::new(file).expect("could not read JAR");
    for i in 0..zip.len() {
        let mut class_file = zip.by_index(i).unwrap();
        if class_file.name().ends_with("class") {
            let class = ClassReader::new_from_reader(&mut class_file).unwrap();
            codecity::process_class(&class);
            classes.push(class);
        }
    }
}
