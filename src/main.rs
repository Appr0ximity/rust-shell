#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs::File, process::Command};
use which::which;

struct ParsedResult{
    args: Vec<String>,
    output_file: Vec<String>,
    redirect: bool
}

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
        let mut output = String::new();
        let parsed_result = parse_command(command);
        let words = parsed_result.args;
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
                                    'n' => output.push_str("\n"),
                                    't' => output.push_str("\t"),
                                    '\\' => output.push_str("\\"),
                                    _ => {
                                        output.push('\\');
                                        output.push(escaped);
                                    }
                                }
                            }else{
                                output.push(c);
                            }
                        }else{
                            output.push(c);
                        }
                    }
                    if i < words.len() - 1{
                        output.push(' ');
                    }
                }
                if !parsed_result.redirect{
                    println!("{}", output);
                }
            },
            "type" => {
                if words.len() < 2{
                    output = format!("Usage: type <command>");
                    if !parsed_result.redirect{
                        println!("{}", output);
                    }
                    continue;
                }
                let cmd = &words[1];
                if shell_built_in.contains(&cmd.as_str()) {
                    output = format!("{} is a shell builtin", cmd)
                } else if let Ok(path) = which(cmd) {
                    output = format!("{} is {}", cmd, path.display())
                } else {
                    output = format!("{}: not found", cmd)
                }
                if !parsed_result.redirect{
                    println!("{}", output);
                }
            },
            "pwd" =>{
                match env::current_dir() {
                    Ok(path) => output = format!("{}", path.display()),
                    Err(e) => output = format!("Error while displaying the path: {}",e),
                }
                if !parsed_result.redirect{
                    println!("{}", output);
                }
            },
            "cd" => {
                if words.len() != 2 {
                    output = format!("Usage: cd <directory>");
                    if !parsed_result.redirect{
                        println!("{}", output);
                    }
                    continue;
                }
                if words[1] == "~" {
                    let home = env::var("HOME").expect("HOME not set");
                    if env::set_current_dir(&home).is_ok() {
                        continue;
                    }else {
                        output = format!("Error while changing to HOME");
                        if !parsed_result.redirect{
                            println!("{}", output);
                        }
                        continue;
                    }
                }
                if env::set_current_dir(&words[1]).is_ok() {
                    continue;
                } else {
                    output = format!("cd: {}: No such file or directory", words[1])
                }
                if !parsed_result.redirect{
                    println!("{}", output);
                }
            }
            _ => {
                let cmd = &words[0];
                if let Ok(_path) = which(cmd) {
                    let result = Command::new(cmd)
                        .args(&words[1..])
                        .output();
                    match result {
                        Ok(result_out) => {
                            output = format!("{}", String::from_utf8_lossy(&result_out.stdout));
                            if !result_out.stderr.is_empty() {
                                eprint!("{}", String::from_utf8_lossy(&result_out.stderr));
                            }
                            if !parsed_result.redirect{
                                println!("{}", output);
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    output = format!("{}: not found", cmd);
                }
            }
        }
        if parsed_result.redirect == true{
            for file_name in parsed_result.output_file{
                if let Ok(mut file) = File::create(file_name){
                    let write_to = file.write_all(output.as_bytes());
                    match write_to{
                        Ok(_result) => {

                        }
                        Err(e) => eprint!("Error while writing to file: {}", e)
                    }
                }
            }
        }
    }
}

fn parse_command(input: &str)->ParsedResult{
    let mut args = Vec::new();
    let mut arg = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut output_file:Vec<String>  = Vec::new();
    let mut is_file_name = false;
    let mut redirect = false;
    while let Some(&c) = chars.peek(){
        chars.next();
        match c {
            ' ' | '\t' | '\n' if !in_single && !in_double =>{
                if !arg.is_empty() && !is_file_name{
                    args.push(arg);
                    arg = String::new();
                }else if !arg.is_empty() && is_file_name {
                    output_file.push(arg.clone());
                    is_file_name = false;
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
            },
            '>' if !in_single && !in_double => {
                if !arg.is_empty(){
                    args.push(arg);
                    arg = String::new();
                }
                redirect = true;
                is_file_name = true;
            },
            '1' if !in_single && !in_double=>{
                if chars.peek() == Some(&'>'){
                    chars.next();
                    redirect = true;
                    is_file_name = true;
                    if !arg.is_empty(){
                        args.push(arg);
                        arg = String::new();
                    }
                }else{
                    arg.push(c);
                }
            }
            _ => arg.push(c),
        }
    }
    if !arg.is_empty() {
        if is_file_name{
            output_file.push(arg);
        }else{
            args.push(arg);
        }
    }
    ParsedResult { args, output_file , redirect}
}