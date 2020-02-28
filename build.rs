extern crate regex;

use regex::Regex;
use std::fs;
use std::ops::Deref;
use std::env;
use std::path::{PathBuf, Path};

fn get_files(dir:&str)->Vec<String> {
    fs::read_dir(dir).unwrap()
        .filter(|p| { p.as_ref().unwrap().path().is_file() })
        .map(|f| { String::from(f.unwrap().path().to_str().unwrap()) })
        .collect::<Vec<_>>()
}

fn main() {
    //build static lib
    let csrc =  get_files("csrc");
    let cfile_pat = Regex::new(r".*\.c$").unwrap();
    let cfiles = csrc.iter()
        .filter(|c| { cfile_pat.is_match(c) })
        .collect::<Vec<_>>();

    let hfile_pat = Regex::new(r".*\.h$").unwrap();
    let hfiles = csrc.iter()
        .filter(|h| { hfile_pat.is_match(h) })
        .collect::<Vec<_>>();

    let vsrc =  get_files("vsrc");
    let vhfile_pat = Regex::new(r".*\.vh$").unwrap();
    let vhfiles = vsrc.iter()
        .filter(|vh| { vhfile_pat.is_match(vh) })
        .collect::<Vec<_>>();

    println!("cargo:rerun-if-changed=csrc");
    println!("cargo:rerun-if-changed=vsrc");
    for file in [&cfiles[..], &hfiles[..], &vhfiles[..]].concat() {
        println!("cargo:rerun-if-changed={}", file);
    }

    cc::Build::new()
        .files(&cfiles)
        .include("csrc")
        .shared_flag(true)
        .static_flag(true)
        .compile("dm.c");

    //build dyn lib
    let profile = env::var("PROFILE").expect("Can not get $PROFILE");
    let target_dir = env::var("CARGO_TARGET_DIR")
        .unwrap_or(env::var("CARGO_BUILD_TARGET_DIR")
            .unwrap_or(String::from("target")));
    let final_dir = Path::new(&target_dir)
        .join(Path::new(&profile));

    cc::Build::new()
        .files(&cfiles)
        .include("csrc")
        .shared_flag(true)
        .pic(true)
        .cargo_metadata(false)
        .out_dir(&final_dir)
        .compile("dm.c.so");
    fs::rename(&final_dir.join("libdm.c.so.a"), final_dir.join("libdm.c.so")).expect("Can not rename libdm.c.so.a!");

    //copy header file
    for file in [&hfiles[..], &vhfiles[..]].concat() {
        let basename = Path::new(file).file_name().unwrap();
        fs::copy(file, &final_dir.join(basename)).expect(&format!("Can not copy {}!",file));
    }
    println!("cargo:rerun-if-changed=build.rs");
}