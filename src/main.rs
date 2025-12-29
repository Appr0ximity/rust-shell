#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop{
        let keywords: Vec<&str> = vec!["exit", "echo", "type"];
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
                if keywords.contains(&words[1].as_str()){
                    println!("{} is a shell builtin", words[1]);
                }else{
                    println!("{}: not found", words[1])  
                }
            }
            _ => println!("{}: not found", command)
        }
    }
}
