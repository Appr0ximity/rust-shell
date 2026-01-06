use crate::ParsedResult;

/// Parses a shell command line into commands, arguments, and redirections.
/// Handles pipes, output/error redirection, and quoted strings.
///
/// # Arguments
/// * `input` - The command line string to parse
///
/// # Returns
/// * `ParsedResult` - The parsed structure for execution
pub fn parse_command(input: &str)->ParsedResult{
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