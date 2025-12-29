#[allow(unused_imports)]
use std::io::{self, Write};
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
            _ => println!("{}: not found", command)
        }
    }
}