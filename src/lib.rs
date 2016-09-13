extern crate regex;

use std::process::{Command, ChildStdout, Stdio};
use std::io;
use std::io::{BufReader};
use std::io::prelude::*;
use regex::Regex;

pub struct Scan {
    pid: Option<u32>,
    queue: Vec<Option<String>>,
    out: BufReader<ChildStdout>,
}

impl Scan {
    pub fn stop(&mut self) {
        if let Some(pid) = self.pid.take() {
            // Brute force kill our child.
            Command::new("kill").arg("-TERM").arg(&format!("{}", pid)).output().unwrap();
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Discovery {
    name: String,
    mac: Mac,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Mac([u8; 6]);


impl Iterator for Scan {
    type Item = Discovery;

    fn next(&mut self) -> Option<Discovery> {
        loop {
            let s = match self.queue.pop() {
                Some(Some(s)) => s,
                _ => {
                    let mut s = String::new();
                    match self.out.read_line(&mut s) {
                        Ok(len) => {
                            if len == 0 {
                                return None;
                            }
                            s
                        }
                        Err(_) => return None,
                    }
                }
            };

            let re = Regex::new(r#"(?m)^([A-F0-9]{2}):([A-F0-9]{2}):([A-F0-9]{2}):([A-F0-9]{2}):([A-F0-9]{2}):([A-F0-9]{2})\s*(.*?)[\r\n]*$"#).unwrap();
            if let Some(cap) = re.captures(&s) {
                return Some(Discovery {
                    name: cap.at(7).unwrap().to_string(),
                    mac: Mac([
                        u8::from_str_radix(cap.at(1).unwrap(), 16).unwrap(),
                        u8::from_str_radix(cap.at(2).unwrap(), 16).unwrap(),
                        u8::from_str_radix(cap.at(3).unwrap(), 16).unwrap(),
                        u8::from_str_radix(cap.at(4).unwrap(), 16).unwrap(),
                        u8::from_str_radix(cap.at(5).unwrap(), 16).unwrap(),
                        u8::from_str_radix(cap.at(6).unwrap(), 16).unwrap(),
                    ]),
                });
            }
        }
    }
}

pub fn scan() -> io::Result<Scan> {
    // hcitool requires a TTY, so fake it for now.
    // This was the easiest option -.- pty/tty are better crates going forward.
    let mut hcitool = Command::new("python")
        .arg("-c")
        .arg(r#"import pty; pty.spawn(["hcitool", "lescan"])"#)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute `hcitool`");

    let pid = hcitool.id();
    let mut buf = BufReader::new(hcitool.stdout.take().unwrap());

    // Queue two lines to find i/o error line.
    let mut queue = vec![];
    let mut s = String::new();
    match buf.read_line(&mut s) {
        Ok(_) => {
            if s.find("Input/output error").is_some() {
                hcitool.wait().unwrap();
                Command::new("hciconfig").arg("hdi0").arg("down").output().unwrap();
                Command::new("hciconfig").arg("hdi0").arg("up").output().unwrap();
                return scan();
            } else {
                queue.push(Some(s));
            }
        }
        Err(_) => {
            queue.push(None);
        }
    };

    Ok(Scan {
        pid: Some(pid),
        queue: queue,
        out: buf,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
