#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;
use which::which;

fn main() {
    loop{
        let shell_built_in: Vec<&str> = vec!["echo", "exit", "type"];
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();
        if command.is_empty(){
            continue;
        }
        let words: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
        match words[0].as_str(){
            "exit" => break,
            "echo" => {
                for (i, word) in words.iter().enumerate().skip(1){
                    print!("{}", word);
                    if i < words.len() - 1 { print!(" ")}
                }
                println!();
            },
            "type" => {
                let cmd = &words[1];
                if shell_built_in.contains(&cmd.as_str()) {
                    println!("{} is a shell builtin", cmd);
                } else if let Ok(path) = which(cmd) {
                    println!("{} is {}", cmd, path.display());
                } else {
                    println!("{}: not found", cmd);
                }
            }
            _ => {
                let cmd = &words[0];
                if let Ok(_path) = which(cmd) {
                    let output = Command::new(cmd)
                        .args(&words[1..])
                        .output();
                    match output {
                        Ok(output) => {
                            print!("{}", String::from_utf8_lossy(&output.stdout));
                            if !output.stderr.is_empty() {
                                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    println!("{}: not found", cmd);
                }
            }
        }
    }
}