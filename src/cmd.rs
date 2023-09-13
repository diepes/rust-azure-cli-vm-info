extern crate regex;
use regex::Regex;

extern crate lazy_static;
use lazy_static::lazy_static;

use colored::Colorize;
use std::process::Command;

const DEBUG: usize = 0;

pub fn run(cmd: &str) -> json::JsonValue {
    // Use regex to split spaces and keep 'quoted sub' str together.
    if DEBUG > 0 {
        println!("DEBUG cmd={cmd}", cmd = cmd.on_blue());
    }
    let cmds: Vec<&str> = split_and_strip(cmd);
    // build command and add args
    let mut command = Command::new(cmds[0]);
    for (i, arg) in cmds.iter().enumerate() {
        if i > 0 {
            command.arg(arg);
        }
    }
    let out_result = command.output();
    //    let status = command.status();
    // let status = status.expect("Error getting shell exit status");
    // unwrap result
    let output = match out_result {
        Ok(out) => out,
        Err(e) => {
            eprintln!("ERR {}", e);
            panic!()
        }
    };

    if output.status.success() {
        if DEBUG > 0 {
            eprintln!("Success cmd: {cmd}");
            eprintln!("Success output: {output:?}");
            eprintln!("Success output: {:?}", output.status.code());
        }
    } else {
        let stderr = String::from_utf8(output.stderr).expect("Error converting utf8");
        eprintln!(
            "code={code:?}, status={status}\n┎######\nstderr=\n{stderr}\n┖######",
            code = output.status.code(),
            status = output.status,
            stderr = stderr.red()
        );
        panic!()
    }

    let stdout = String::from_utf8(output.stdout).expect("Error converting utf8");
    let json_return = json::parse(&stdout).expect("Parse JsonValue failed");
    if DEBUG > 1 {
        eprintln!("json_return: '{:?}'", json_return.as_str());
    }
    json_return
}

fn split_and_strip(input: &str) -> Vec<&str> {
    RE.find_iter(input)
        .map(|m| m.as_str().trim().trim_matches('\''))
        .collect()
}
lazy_static! {
    static ref RE: Regex = Regex::new(r#"'([^']*)'\s*|([^'\s]*)\s*"#).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_split_and_strip_complex() {
        let input = "Hello 'World War'  'fail' Rust";
        let expected = vec!["Hello", "World War", "fail", "Rust"];
        assert_eq!(split_and_strip(input), expected);
    }
    #[test]
    fn test_split_and_strip_nospaces() {
        let input2 = "NoSpacesHere";
        let expected2 = vec!["NoSpacesHere"];
        assert_eq!(split_and_strip(input2), expected2);
    }
    #[test]
    fn test_split_and_strip_empty_quotes() {
        let input3 = "Empty '' Single Quotes";
        let expected3 = vec!["Empty", "", "Single", "Quotes"];
        assert_eq!(split_and_strip(input3), expected3);
    }
}
