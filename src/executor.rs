use std::{env, process::Command};

use which::which;

use crate::{CommandResult, ParsedResult};

pub fn run_command(command: &Vec<String>, parsed_result: &ParsedResult, built_ins: &Vec<String>, history: &Vec<String>)-> CommandResult{
    let mut output = String::new();
    let mut error_output = String::new();
    if command.is_empty() {return CommandResult::NoOp;}
    match command[0].as_str(){
        "exit" => return CommandResult::Exit,
        "echo" => {
            for (i,word) in command.iter().enumerate().skip(1){
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
                if i < command.len() - 1{
                    output.push(' ');
                }
            }
            output.push('\n');
            return CommandResult::Output(output,error_output);
        },
        "type" => {
            if command.len() < 2{
                eprintln!("Usage: type <command>");
                return CommandResult::Output(output,error_output);
            }
            let cmd = &command[1];
            if built_ins.iter().any(|s| s == cmd) {
                output = format!("{} is a shell builtin\n", cmd)
            } else if let Ok(path) = which(cmd) {
                output = format!("{} is {}\n", cmd, path.display())
            } else {
                output = format!("{}: not found\n", cmd)
            }
            return CommandResult::Output(output,error_output);
        },
        "pwd" =>{
            match env::current_dir() {
                Ok(path) => output = format!("{}\n", path.display()),
                Err(e) => {
                    eprintln!("Error while displaying the path: {}",e);
                    return CommandResult::Output(output,error_output);
                },
            }
            return CommandResult::Output(output,error_output);
        },
        "cd" => {
            if command.len() != 2 {
                eprintln!("Usage: cd <directory>");
                return CommandResult::NoOp;
            }
            let target = if command[1] == "~" {
                match env::var("HOME"){
                    Ok(home) => home,
                    Err(e) => {
                        eprintln!("HOME not set: {}", e);
                        return CommandResult::NoOp;
                    }
                }
            }else{
                command[1].clone()    
            };
            if let Err(_) = env::set_current_dir(&target){
                eprintln!("cd: {}: No such file or directory", command[1]);
            }
            return CommandResult::NoOp;
        },
        "history" =>{
            for (i, entry) in history.iter().enumerate(){
                output.push_str(&format!("{} {}\n", i+1, entry));
            }
            return CommandResult::Output(output, error_output);
        },
        _ => {
            let cmd = &command[0];
            if let Ok(_path) = which(cmd) {
                let result = Command::new(cmd)
                    .args(&command[1..])
                    .output();
                match result {
                    Ok(result_out) => {
                        output = format!("{}", String::from_utf8_lossy(&result_out.stdout));
                        if !result_out.stderr.is_empty() {
                            error_output = format!("{}", String::from_utf8_lossy(&result_out.stderr));
                        }
                        if !parsed_result.redirect_as_output && !parsed_result.append_as_output && !output.is_empty(){
                            if !output.ends_with('\n'){
                                println!()
                            }
                        }
                        return CommandResult::Output(output,error_output);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return CommandResult::Output(output,error_output);
                    },
                }
            } else {
                output = format!("{}: not found\n", cmd);
                return CommandResult::Output(output,error_output);
            }
        }
    }
}