// extern crate regex;
// use regex::Regex;
// use std::fs;
// use std::ops::Deref;

//should in post build
fn main() {
    // let files = fs::read_dir("csrc").unwrap()
    //     .filter(|p|{p.as_ref().unwrap().path().is_file()})
    //     .map(|f|String::from({f.unwrap().path().to_str().unwrap()}))
    //     .collect::<Vec<_>>();
    //
    // let cfile_pat = Regex::new(r".*\.c$").unwrap();
    // let cfiles = files.iter()
    //     .filter(|c|{cfile_pat.is_match(c)})
    //     .collect::<Vec<_>>();
    //
    // let hfile_pat = Regex::new(r".*\.h$").unwrap();
    // let hfiles = files.iter()
    //     .filter(|h|{hfile_pat.is_match(h)})
    //     .collect::<Vec<_>>();
    //
    // for file in [&cfiles[..],&hfiles[..]].concat() {
    //     println!("cargo:rerun-if-changed={}",file);
    // }
    //
    // cc::Build::new()
    //     .files(cfiles)
    //     .include("csrc")
    //     .compile("libdm.c.a");
}