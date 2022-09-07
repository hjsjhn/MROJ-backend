use file_diff;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

use crate::config::RESULTS;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

// Return 1 if two files are different
pub fn diff_strict(file1: &str, file2: &str) -> usize {
    if file_diff::diff(file1, file2) { return 0; }
    else { return 1; }
}

pub fn diff_standard(file1: &str, file2: &str) -> usize {
   let status = match cfg![target="windows"] {
        true => Command::new("fc")
                     .args(["/W", file1, file2])
                     .stdout(Stdio::null())
                     .stderr(Stdio::null())
                     .status(),
        false => Command::new("diff")
                     .args(["-w", file1, file2])
                     .stdout(Stdio::null())
                     .stderr(Stdio::null())
                     .status(),
   };
   match status {
        Ok(s) => { 
            if s.code().unwrap() == 0 { return 0; }
            else { return 1; }
        },
        _ => { return 1; },
   };
}

pub async fn diff_spj(spj_info: &Vec<String>) -> Result<(usize, String), ()> {
    let mut output = Command::new(&spj_info[0])
                            .args(&spj_info[1..])
                            .stdin(Stdio::null())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::null())
                            .output()
                            .unwrap()
                            .stdout;

    let stdout = String::from_utf8(output).unwrap();
    let outputs: Vec<&str> = stdout.trim().split('\n').collect();
    println!("{:?}", outputs);
    if outputs.len() != 2 { return Err(()); }
    if !RESULTS.contains(&outputs[0]) { return Err(()); }
    match outputs[0].eq("Accepted") {
        true => Ok((0, outputs[1].to_string())),
        false => Ok((1, outputs[1].to_string())),
    }
}