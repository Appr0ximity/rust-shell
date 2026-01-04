use std::{env, fs};
use std::process::Child;
use std::result::Result::Ok;
use std::io::{self, Write};
use std::{ fs::{File, OpenOptions}, process::{Command}};
use rustyline::{CompletionType, Config, Editor, Helper, completion::{Completer, Pair}, highlight::Highlighter, hint::Hinter};

use crate::executor::run_command;
use crate::parser::{parse_command};
mod parser;
mod executor;

pub struct MyHelper{
    commands: Vec<String>
}
pub enum CommandResult{
    Output (String, String),
    Exit,
    NoOp
}

impl Completer for MyHelper {
    type Candidate = Pair;
    
    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut completions = Vec::new();
        for cmd in &self.commands {
            if cmd.starts_with(line) {
                let completion = cmd.clone();
                completions.push(Pair {
                    display: completion.clone(),
                    replacement: format!("{} ", completion),
                });
            }
        }
        Ok((0, completions))
    }
}

impl Hinter for MyHelper {}

impl Highlighter for MyHelper {}

impl Helper for MyHelper {}

struct ParsedResult{
    commands: Vec<Vec<String>>,
    output_file: Vec<String>,
    error_file: Vec<String>,
    redirect_as_output: bool,
    redirect_as_error: bool,
    append_as_output: bool,
    append_as_error: bool
}

fn main() {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let built_ins: Vec<String> = vec!["echo", "exit", "type", "pwd", "cd", "history"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let mut all_commands: Vec<String> = built_ins.clone();
    all_commands.extend(get_all_commands());
    let helper = MyHelper { commands:all_commands.clone() };
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(helper));
    'shell: loop{
        io::stdout().flush().unwrap();
        let full_command = rl.readline("$ ");
        match full_command {
            Ok(full_command) => {
                let _ = rl.add_history_entry(full_command.as_str());
                let history_vec = rl.history().iter().map(|s| s.to_string()).collect();
                let parsed_result = parse_command(&full_command);
                if parsed_result.commands.len() > 1{
                    use std::process::Stdio;
                    let mut last_child: Option<Child> = None;
                    let mut previous_output: Option<String> = None;

                    for (i, cmd_parts) in parsed_result.commands.iter().enumerate(){
                        let is_builtin = built_ins.contains(&cmd_parts[0]);

                        if !is_builtin{
                            let mut cmd = Command::new(&cmd_parts[0]);
                            cmd.args(&cmd_parts[1..]);

                            if i > 0{
                                if let Some(mut prev_child) = last_child.take(){
                                    let prev_stdout = prev_child.stdout.take().unwrap();
                                    cmd.stdin(prev_stdout);
                                }else if previous_output.is_some(){
                                    cmd.stdin(Stdio::piped());
                                }
                            }
                            
                            let needs_builtin_input = i > 0 && previous_output.is_some();

                            if i < parsed_result.commands.len() - 1 {
                                cmd.stdout(Stdio::piped());
                            }

                            match cmd.spawn(){
                                Ok(mut child) =>{
                                    if needs_builtin_input{
                                        if let Some (prev_out) = previous_output.take() {
                                            if let Some (mut stdin) = child.stdin.take(){
                                                let _ = stdin.write_all(prev_out.as_bytes());
                                                drop(stdin);
                                            }
                                        }
                                    }
                                    last_child = Some(child);
                                },
                                Err(e) =>{
                                    eprint!("Error while trying to spawn command: {}", e);
                                }
                            }
                        }else{
                            match run_command(cmd_parts, &parsed_result, &built_ins, &history_vec){
                                CommandResult::Output(output, _error_output)=>{
                                    previous_output = Some(output);
                                    last_child = None;
                                },
                                CommandResult::NoOp => {
                                    previous_output = None;
                                    last_child = None;
                                },
                                CommandResult::Exit => {
                                    break 'shell;
                                }
                            }
                        }
                    }

                    if let Some(last) = last_child{
                        match last.wait_with_output(){
                            Ok(cmd_output) => {
                                print!("{}", String::from_utf8_lossy(&cmd_output.stdout));
                                if !cmd_output.stderr.is_empty() {
                                    eprint!("{}", String::from_utf8_lossy(&cmd_output.stderr));
                                }
                            },
                            Err(e) =>{
                                eprintln!("{}", e);
                            }
                        }
                    } else if let Some(output) = previous_output{
                        print!("{}", output);
                    }
                    continue;
                }
                match run_command(&parsed_result.commands[0], &parsed_result, &built_ins, &history_vec){
                    CommandResult::Output(output, error_output) =>{
                        if !parsed_result.redirect_as_output && !parsed_result.append_as_output && !output.is_empty() {
                            print!("{}", output);
                        }
                        if !parsed_result.redirect_as_error && !parsed_result.append_as_error && !error_output.is_empty() {
                            eprint!("{}", error_output);
                        }
                        if parsed_result.redirect_as_output {
                            for file_name in &parsed_result.output_file{
                                match File::create(file_name){
                                    Ok(mut file) => {
                                        if let Err(e) = file.write_all(output.as_bytes()){
                                            eprintln!("Error while writing to file: {}",e);
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Error while creating the file: {}", e);
                                    }
                                }
                            }
                        }
                        if parsed_result.redirect_as_error {
                            for file_name in &parsed_result.error_file{
                                match File::create(file_name){
                                    Ok(mut file) => {
                                        if let Err(e) = file.write_all(error_output.as_bytes()){
                                            eprint!("Error while writing to file: {}", e);
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Error while creating the file: {}", e);
                                    }
                                }
                            }
                        }
                        if parsed_result.append_as_output {
                            for file_name in &parsed_result.output_file{
                                match OpenOptions::new().create(true).append(true).open(file_name){
                                    Ok(mut file) =>{
                                        if let Err(e ) = file.write_all(output.as_bytes()){
                                            eprintln!("Error while appending to file: {}", e);
                                        }
                                    }
                                    Err (e) => {
                                        eprintln!("Error while opening the file: {}", e);
                                    }
                                }
                            }
                        }
                        if parsed_result.append_as_error {
                            for file_name in &parsed_result.error_file{
                                match OpenOptions::new().create(true).append(true).open(file_name){
                                    Ok(mut file) =>{
                                        if let Err(e ) = file.write_all(error_output.as_bytes()){
                                            eprintln!("Error while appending to file: {}", e);
                                        }
                                    }
                                    Err (e) => {
                                        eprintln!("Error while opening the file: {}", e);
                                    }
                                }
                            }
                        }
                    },
                    CommandResult::NoOp => continue,
                    CommandResult::Exit => break,
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            },
        }
    }
}

fn get_all_commands()-> Vec<String>{
    let mut all_commands = Vec::new();
    if let Ok(path_vars) = env::var("PATH"){
        let separator = if cfg!(windows){";"}else{":"};

        for dir in path_vars.split(separator){
            if let Ok(entries) = fs::read_dir(dir){
                for entry in entries.filter_map(Result::ok){
                    let path = entry.path();

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = fs::metadata(&path){
                            let permissions = metadata.permissions();
                            if path.is_file() && permissions.mode() & 0o111 != 0{
                                if let Some(name) = path.file_name(){
                                    all_commands.push(name.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    #[cfg(windows)]
                    {
                        if path.is_file(){
                            if let Some(name) = path.file_name(){
                                all_commands.push(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    all_commands.sort();
    all_commands.dedup();
    all_commands
}