use std::result::Result::Ok;
use std::io::{self, Write};
use std::{ env, fs::{self, File, OpenOptions}, process::{Command}};
use rustyline::{CompletionType, Config, Editor, Helper, completion::{Completer, Pair}, highlight::Highlighter, hint::Hinter};
use which::which;

struct MyHelper{
    commands: Vec<String>
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
    let built_ins: Vec<String> = vec!["echo", "exit", "type", "pwd", "cd"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let mut all_commands: Vec<String> = built_ins.clone();
    all_commands.extend(get_all_commands());
    let helper = MyHelper { commands:all_commands.clone() };
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(helper));
    loop{
        io::stdout().flush().unwrap();
        let full_command = rl.readline("$ ");
        match full_command {
            Ok(full_command) => {
                let _ = rl.add_history_entry(full_command.as_str());
                let parsed_result = parse_command(&full_command);
                if parsed_result.commands.len() > 1{
                    let all_external_commands = parsed_result.commands
                        .iter()
                        .all(|cmd| !built_ins.contains(&cmd[0]));

                    if all_external_commands{
                        use std::process::Stdio;
                        let mut children: Vec<std::process::Child> = Vec::new();

                        for (i, cmd_parts) in parsed_result.commands.iter().enumerate(){
                            let mut cmd = Command::new(&cmd_parts[0]);
                            cmd.args(&cmd_parts[1..]);

                            if i > 0{
                                if let Some(prev_child) = children.last_mut(){
                                    let prev_stdout = prev_child.stdout.take().unwrap();
                                    cmd.stdin(prev_stdout);
                                }
                            }

                            if i < parsed_result.commands.len() - 1 {
                                cmd.stdout(Stdio::piped());
                            }

                            match cmd.spawn(){
                                Ok(child) =>{
                                    children.push(child);
                                },
                                Err(e) =>{
                                    eprint!("Error while trying to spawn command: {}", e);
                                }
                            }
                        }

                        if let Some(last_child) = children.pop(){
                            match last_child.wait_with_output(){
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
                        }
                        continue;
                    }
                }
                run_command(&parsed_result.commands[0], &parsed_result, &built_ins);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            },
        }
    }
}

fn parse_command(input: &str)->ParsedResult{
    let mut commands = Vec::new();
    let mut args = Vec::new();
    let mut arg = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut output_file:Vec<String>  = Vec::new();
    let mut error_file:Vec<String>  = Vec::new();
    let mut is_output_file_name = false;
    let mut is_error_file_name = false;
    let mut redirect_as_output = false;
    let mut redirect_as_error = false;
    let mut append_as_output = false;
    let mut append_as_error = false;
    while let Some(&c) = chars.peek(){
        chars.next();
        match c {
            ' ' | '\t' | '\n' if !in_single && !in_double =>{
                if !arg.is_empty() && !is_output_file_name && !is_error_file_name{
                    args.push(arg);
                    arg = String::new();
                }else if !arg.is_empty() && is_output_file_name {
                    output_file.push(arg.clone());
                    is_output_file_name = false;
                    arg = String::new();
                }else if !arg.is_empty() && is_error_file_name{
                    error_file.push(arg.clone());
                    is_error_file_name = false;
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
            '>' if !in_single && !in_double && !is_error_file_name && !is_output_file_name => {
                if !arg.is_empty(){
                    args.push(arg);
                    arg = String::new();
                }
                redirect_as_output = true;
                is_output_file_name = true;
                if chars.peek() == Some(&'>'){
                    append_as_output = true;
                    redirect_as_output = false;
                    chars.next();
                }
            },
            '|' if !in_single && !in_double =>{
                if !arg.is_empty(){
                    args.push(arg);
                    arg = String::new();
                }
                if !args.is_empty(){
                    commands.push(args);
                    args = Vec::new();
                }
            },
            '1' if !in_single && !in_double=>{
                if chars.peek() == Some(&'>'){
                    chars.next();
                    redirect_as_output = true;
                    is_output_file_name = true;
                    if !arg.is_empty(){
                        args.push(arg);
                        arg = String::new();
                    }
                    if chars.peek() == Some(&'>'){
                        append_as_output = true;
                        redirect_as_output = false;
                        chars.next();
                    }
                }else{
                    arg.push(c);
                }
            },
            '2' if !in_single && !in_double=>{
                if chars.peek() == Some(&'>'){
                    chars.next();
                    redirect_as_error = true;
                    is_error_file_name = true;
                    if !arg.is_empty(){
                        args.push(arg);
                        arg = String::new();
                    }
                    if chars.peek() == Some(&'>'){
                        append_as_error = true;
                        redirect_as_error = false;
                        chars.next();
                    }
                }else{
                    arg.push(c);
                }
            }
            _ => arg.push(c),
        }
    }
    if !arg.is_empty() {
        if is_output_file_name{
            output_file.push(arg);
        }else if is_error_file_name{
            error_file.push(arg);
        }else{
            args.push(arg);
        }
    }
    if !args.is_empty(){
        commands.push(args.clone());
    }
    ParsedResult { commands, output_file, error_file, redirect_as_output, redirect_as_error, append_as_error, append_as_output }
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

fn run_command(command: &Vec<String>, parsed_result: &ParsedResult, built_ins: &Vec<String>){
    let mut output = String::new();
    let mut error_output = String::new();
    if command.is_empty() {return;}
    match command[0].as_str(){
        "exit" => return,
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
            if !parsed_result.redirect_as_output && !parsed_result.append_as_output{
                print!("{}", output);
            }
        },
        "type" => {
            if command.len() < 2{
                eprintln!("Usage: type <command>");
                return;
            }
            let cmd = &command[1];
            if built_ins.iter().any(|s| s == cmd) {
                output = format!("{} is a shell builtin", cmd)
            } else if let Ok(path) = which(cmd) {
                output = format!("{} is {}", cmd, path.display())
            } else {
                output = format!("{}: not found", cmd)
            }
            if !parsed_result.redirect_as_output && !parsed_result.append_as_output{
                println!("{}", output);
            }
        },
        "pwd" =>{
            match env::current_dir() {
                Ok(path) => output = format!("{}", path.display()),
                Err(e) => {
                    eprintln!("Error while displaying the path: {}",e);
                    return;
                },
            }
            if !parsed_result.redirect_as_output && !parsed_result.append_as_output{
                println!("{}", output);
            }
        },
        "cd" => {
            if command.len() != 2 {
                eprintln!("Usage: cd <directory>");
                return;
            }
            if command[1] == "~" {
                let home = env::var("HOME").expect("HOME not set");
                if env::set_current_dir(&home).is_ok() {
                    return;
                }else {
                    eprintln!("Error while changing to HOME");
                    return;
                }
            }
            if env::set_current_dir(&command[1]).is_err() {
                eprintln!("cd: {}: No such file or directory", command[1]);
            }
        }
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
                            print!("{}", output);
                            if !output.ends_with('\n'){
                                println!()
                            }
                        }
                        if !parsed_result.redirect_as_error && !parsed_result.append_as_error{
                            eprint!("{}", error_output);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else {
                output = format!("{}: not found", cmd);
                if !parsed_result.redirect_as_output && !parsed_result.append_as_output{
                    println!("{}", output);
                }
            }
        }
    }
    if parsed_result.redirect_as_output {
        for file_name in &parsed_result.output_file{
            match File::create(file_name){
                Ok(mut file) => {
                    if let Err(e) = file.write_all(output.as_bytes()){
                        eprintln!("Error while writing to file: {}",e);
                    }
                },
                Err(e) => eprintln!("Error while creating the file: {}", e)
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
                Err(e) => eprintln!("Error while creating the file: {}", e),
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
                Err (e) => eprintln!("Error while opening the file: {}", e)
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
                Err (e) => eprintln!("Error while opening the file: {}", e)
            }
        }
    }
}