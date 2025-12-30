#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, process::Command};
use which::which;

fn main() {
    let shell_built_in: Vec<&str> = vec!["echo", "exit", "type", "pwd", "cd"];
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();
        if command.is_empty(){
            continue;
        }
        let words = parse_command(command);
        if words.is_empty() {continue;}
        match words[0].as_str(){
            "exit" => break,
            "echo" => {
                for (i,word) in words.iter().enumerate().skip(1){
                    let mut chars = word.chars().peekable();
                    while let Some(&c) = chars.peek(){
                        chars.next();
                        if c == '\\'{
                            if chars.peek().is_some(){
                                let escaped = chars.next().unwrap();
                                match escaped{
                                    'n' => print!("\n"),
                                    't' => print!("\t"),
                                    '\\' => print!("\\"),
                                    _ => {
                                        print!("\\{}",escaped);
                                    }
                                }
                            }else{
                                print!("{}", c);
                            }
                        }else{
                            print!("{}", c);
                        }
                    }
                    if i < words.len() - 1{
                        print!(" ");
                    }
                }
                println!();
            },
            "type" => {
                if words.len() < 2{
                    println!("Usage: type <command>");
                    continue;
                }
                let cmd = &words[1];
                if shell_built_in.contains(&cmd.as_str()) {
                    println!("{} is a shell builtin", cmd);
                } else if let Ok(path) = which(cmd) {
                    println!("{} is {}", cmd, path.display());
                } else {
                    println!("{}: not found", cmd);
                }
            },
            "pwd" =>{
                match env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(e) => println!("Error while displaying the path: {}",e),
                }
            },
            "cd" => {
                if words.len() != 2 {
                    println!("Usage: cd <directory>");
                    continue;
                }
                if words[1] == "~" {
                    let home = env::var("HOME").expect("HOME not set");
                    if env::set_current_dir(&home).is_ok() {
                        continue;
                    }else {
                        println!("Error while changing to HOME");
                        continue;
                    }
                }
                if env::set_current_dir(&words[1]).is_ok() {
                    continue;
                } else {
                    println!("cd: {}: No such file or directory", words[1]);
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

fn parse_command(input: &str)->Vec<String>{
    let mut args = Vec::new();
    let mut arg = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(&c) = chars.peek(){
        chars.next();
        match c {
            ' ' | '\t' | '\n' if !in_single && !in_double =>{
                if !arg.is_empty(){
                    args.push(arg);
                    arg = String::new();
                }
            },
            '"' =>{
                if in_single{
                    arg.push(c);
                }else{
                    in_double = !in_double;
                }
            },
            '\''=>{
                if in_double{
                    arg.push(c);
                }else{
                    in_single = !in_single;
                }
            },
            '\\' => {
                if in_single{
                    arg.push(c);
                }else if in_double{
                    if let Some(&next) = chars.peek(){
                        if next == '"' || next == '\\' || next == '$' || next == '`' {
                            chars.next();
                            arg.push(next);
                        }else{
                            arg.push(c);
                        }
                    }
                }else{
                    if let Some(next) = chars.next(){
                        arg.push(next);
                    }
                }
            }
            _ => arg.push(c),
        }
    }
    if !arg.is_empty() {
        args.push(arg);
    }
    args
}