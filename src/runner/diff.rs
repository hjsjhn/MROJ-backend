use file_diff;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::{Command, Stdio};

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