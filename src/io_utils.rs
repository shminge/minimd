use std::fs::{read_to_string, OpenOptions};
use std::io::Write;


/// Reads a file and returns the content
pub fn read_file(path: &str) -> String {
    read_to_string(path).expect("Something went wrong reading the file")
}

/// Writes a string to a file, overwriting previous content
pub fn write_file(path: &str, content: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path).expect("Something went wrong writing to file");

    file.write_all(content.as_bytes()).expect("Something went wrong writing to file");
}


#[cfg(test)]
mod tests {
    use std::fs::remove_file;
    use super::*;

    #[test]
    fn test_chain() {
        let s = "Hello, world!\n With newlines!";
        write_file("chain.txt", s);
        assert_eq!(read_file("chain.txt"), s);
        remove_file("chain.txt").expect("Unable to remove file");
    }
}
