# Simple rust shell

A learning project implementing a basic interactive shell with support for pipes, redirections, and some built-ins. It's not meant to replace bash or zsh, but it's a fun way to dive into systems programming in Rust. Just wanted to learn how a shell works.

## Features

- Interactive prompt with history and basic tab completion
- Execute external commands
- Built-in commands: `echo`, `exit`, `type`, `pwd`, `cd`, `history`
- Piping (`|`)
- Redirections: `>`, `>>`, `2>`, `2>>` (stdout and stderr)
- History saved to a file (controlled by `HISTFILE` env var)
- Handles quoted arguments and basic escapes in `echo`

## Quick Start

```bash
git clone https://github.com/yourusername/rush.git
cd rush
cargo run
```

Or for a release build:

```bash
cargo build --release
./target/release/rush
```

## Examples

```bash
$ echo "Hello, Rust shell!"
Hello, Rust shell!

$ pwd
/home/user/projects/rush

$ cd ..
$ pwd
/home/user/projects

$ ls -l | grep Cargo
-rw-r--r-- 1 user user 1234 Jan  1 12:00 Cargo.toml

$ echo "First line" > test.txt
$ echo "Second line" >> test.txt
$ cat test.txt
First line
Second line

$ some-bad-command 2> errors.log
$ cat errors.log
some-bad-command: command not found

$ history
1  echo "Hello, Rust shell!"
2  pwd
...

$ exit
```

## Project Structure

- `src/main.rs`: The main REPL loop, history handling, and orchestration
- `src/parser.rs`: Parses input into commands, handles pipes, redirections, and quoting
- `src/executor.rs`: Runs built-ins and spawns external processes with proper piping/redirection

## Dependencies

Keeps it minimal:

- `rustyline` for readline-like input and completion
- `which` to find executables in `$PATH`
- `rustyline-derive` for some convenience macros

## Why Rust?

~~I believe Rust is the most important development in system programming languages since C. What is novel is not any individual feature ("Rust is not a particularly original language"), but the fact that so many amazing features have come together in one mainstream language.~~
I just wanted to learn

## Environment Variables

- `HISTFILE`: Path to save/load command history (defaults to something in your home dir if not set)

Feel free to open issues or PRs if you spot bugs or want to add features!