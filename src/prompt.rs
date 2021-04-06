use std::{collections::VecDeque, path::{Path, PathBuf}, process::{Child, Command as SysCommand, Stdio}, str::FromStr};

use crate::{cmd::{Cmd, CmdEvent}, subprocess::SubProcess};

pub struct Command {
    args: Vec<String>
}

struct CommandParser<'a> {
    chars: std::str::Chars<'a>,
    current: char
}

impl<'a> CommandParser<'a> {
    fn peek(&mut self) -> char {
        return self.current
    }

    fn skip(&mut self) {
        if self.chars.as_str() == "" {
            // This character will indicate end of file since it's unused anyway
            self.current = '\0';
            return;
        }
        self.current = self.chars.next().unwrap();
    }

    fn next(&mut self) -> char {
        let previous = self.current;

        self.skip();

        previous
    }

    pub fn new(src: &'a str) -> Self {
        let chars = src.chars();
        Self { current: ' ', chars }
    }

    fn parse(&mut self) -> Command {
        let mut cmd = Command { args: vec![] };
        while self.peek() != '\0' {
            if let '\'' | '"' = self.peek() { 
                cmd.args.push(self.parse_str());
            }
            else {
                let arg = self.parse_arg();       
                if arg != "" {
                    cmd.args.push(arg)
                }
            }
        }
        cmd
    }

    fn parse_arg(&mut self) -> String {
        let mut arg = "".to_string();
        while self.peek() != ' ' && self.peek() != '\0' {
            let p = self.parse_char();
            if p != '\0' {
                arg.push(p);
            }
        }
        self.skip();
        arg
    }

    fn parse_str(&mut self) -> String {
        let begin = self.next();
        
        let mut res = "".to_string();
        let terminators = [begin, '\0'];
        while !terminators.contains(&self.peek()) {
            let p = self.parse_char();
            if p != '\0' {
                res.push(p);
            }
        }
        self.skip();
        res
    }

    fn parse_char(&mut self) -> char {
        if self.peek() == '\\' {
            self.skip();
            match self.next() {
                'n' => return '\n',
                e => return e
            }
        }
        self.next()
    }
}

impl Command {
    pub fn parse(src: &str) {
        let param = "".to_string();
        let mut is_string = false;
        let mut is_escape = false;
    }
}
pub fn show_prompt(cmd: &mut Cmd) {
    if let Ok(dir) = std::env::current_dir() {
        cmd.write_stdout(dir.as_os_str().to_str().unwrap_or(""));
    }
    cmd.write_stdout(">");
}

// returns if child process has been created
pub fn interpret_command(cmd: &mut Cmd, parsed: Command, stdin: String) -> bool {
    let mut args_iter = parsed.args.iter();
    match args_iter.next().unwrap_or(&"".to_string()).as_str() { 
        "argv" => {
            cmd.write_stdout(&args_iter.fold(String::new(), |a, b| a + b + "&"));
            cmd.put_stdout('\n');
        },
        "cls" | "clear" => {
            cmd.clear();
        }
        "exit" => {
            cmd.exit();
        },
        "cd.." => {
             if let Ok(mut path) = std::env::current_dir() {
                 path.pop();
                 std::env::set_current_dir(path);
             }
        }
        "cd" => {
            if let Ok(mut path) = std::env::current_dir() {
                let arg = args_iter.fold(String::new(), |a, b| a + b + " ");
                let addition = match PathBuf::from_str(arg.trim_end()) {
                    Ok(e) => e,
                    _ => return false
                };
                path.push(addition);
                std::env::set_current_dir(path);
            }
        }
        "" => {
        }
        _ => {
            cmd.attach_child(SubProcess::from_cmd(&stdin));
            return true;
        }

    }
    return false;
}

pub fn end_of_command(cmd: &mut Cmd) {
    cmd.write_stdout("\n");
    show_prompt(cmd);
    let s = cmd.get_stdin() as *const str;
    unsafe { 
        cmd.write_stdout(&*s);
    }
}

pub fn interpret(cmd: &mut Cmd) {
    cmd.put_stdout('\n');
    let stdin = cmd.flush_stdin();
    let mut parsed = CommandParser::new(&stdin).parse();
    if !interpret_command(cmd, parsed, stdin) {
        show_prompt(cmd);
    }
}
