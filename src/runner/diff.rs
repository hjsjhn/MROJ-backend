use file_diff::{diff_files};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

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
    if let Ok(lines1) = read_lines(file1) {
        if let Ok(lines2) = read_lines(file2) {
            for (line1, line2) in lines1.zip(lines2) {
                if let Ok(out1) = line1 {
                    if let Ok(out2) = line2 {
                        if out1.trim_end() != out2.trim_end() {
                            return 1;
                        }
                    } else { return 1; }
                } else { return 1; }
            }   
        } else {
            return 1;
        }
    } else {
        return 1;
    }

    0
}