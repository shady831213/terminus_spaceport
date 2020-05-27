extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate serde_json;
extern crate serde;
use regex::Regex;
use std::fs;
use std::env;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::sync::Mutex;
use serde::Deserialize;



fn get_files(dir:&str)->Vec<String> {
    fs::read_dir(dir).unwrap()
        .filter(|p| { p.as_ref().unwrap().path().is_file() })
        .map(|f| { String::from(f.unwrap().path().to_str().unwrap()) })
        .collect::<Vec<_>>()
}

lazy_static! {
    static ref WORKSPACES: Mutex<BTreeMap<String, &'static Path>> = Mutex::new(BTreeMap::new());
}

fn get_cargo_workspace(manifest_dir: &str) -> &Path {
    let mut workspaces = WORKSPACES.lock().unwrap();
    if let Some(rv) = workspaces.get(manifest_dir) {
        rv
    } else {
        #[derive(Deserialize)]
        struct Manifest {
            workspace_root: String,
        }
        let output = std::process::Command::new(env!("CARGO"))
            .arg("metadata")
            .arg("--format-version=1")
            .current_dir(manifest_dir)
            .output()
            .unwrap();
        let manifest: Manifest = serde_json::from_slice(&output.stdout).unwrap();
        let path = Box::leak(Box::new(PathBuf::from(manifest.workspace_root)));
        workspaces.insert(manifest_dir.to_string(), path.as_path());
        workspaces.get(manifest_dir).unwrap()
    }
}


fn main() {
    //build static lib
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("No &CARGO_MANIFEST_DIR!");
    let workspace_dir = get_cargo_workspace(&manifest_dir);
    let csrc_dir = Path::new(&manifest_dir).join("csrc").to_str().expect("csrc not exists!").to_string();
    let csrc =  get_files(&csrc_dir);
    let cfile_pat = Regex::new(r".*\.c$").unwrap();
    let cfiles = csrc.iter()
        .filter(|c| { cfile_pat.is_match(c) })
        .collect::<Vec<_>>();

    let hfile_pat = Regex::new(r".*\.h$").unwrap();
    let hfiles = csrc.iter()
        .filter(|h| { hfile_pat.is_match(h) })
        .collect::<Vec<_>>();

    let vsrc_dir = Path::new(&manifest_dir).join("vsrc").to_str().expect("vsrc not exists!").to_string();
    let vsrc =  get_files(&vsrc_dir);
    let vhfile_pat = Regex::new(r".*\.vh$").unwrap();
    let vhfiles = vsrc.iter()
        .filter(|vh| { vhfile_pat.is_match(vh) })
        .collect::<Vec<_>>();

    println!("cargo:rerun-if-changed={}", &csrc_dir);
    println!("cargo:rerun-if-changed={}", &vsrc_dir);
    for file in [&cfiles[..], &hfiles[..], &vhfiles[..]].concat() {
        println!("cargo:rerun-if-changed={}", file);
    }

    cc::Build::new()
        .files(&cfiles)
        .include(&csrc_dir)
        .shared_flag(true)
        .static_flag(true)
        .compile("ts.c");

    //build dyn lib
    let profile = env::var("PROFILE").expect("Can not get $PROFILE");
    let target_dir = env::var("CARGO_TARGET_DIR")
        .unwrap_or(env::var("CARGO_BUILD_TARGET_DIR")
            .unwrap_or(String::from("target")));
    let final_dir = workspace_dir.join(&target_dir)
        .join(&profile);

    cc::Build::new()
        .files(&cfiles)
        .include("csrc")
        .shared_flag(true)
        .pic(true)
        .cargo_metadata(false)
        .out_dir(&final_dir)
        .compile("ts.c.so");
    println!("rename {} to {}", &final_dir.join("libts.c.so.a").to_str().unwrap(), final_dir.join("libts.c.so").to_str().unwrap());
    fs::rename(&final_dir.join("libts.c.so.a"), final_dir.join("libts.c.so")).expect("Can not rename libts.c.so.a!");

    //copy header file
    for file in [&hfiles[..], &vhfiles[..]].concat() {
        let basename = Path::new(file).file_name().unwrap();
        println!("copy {} to {}", file, final_dir.join(basename).to_str().unwrap());
        fs::copy(file, &final_dir.join(basename)).expect(&format!("Can not copy {}!",file));
    }
    println!("cargo:rerun-if-changed=build.rs");
}