use std::{borrow::BorrowMut, collections::VecDeque, convert::TryInto, io::{Read, Write}, process::{Child, ChildStderr, ChildStdin, ChildStdout}};

use crate::subprocess::SubProcess;

pub enum CmdEvent {
    ChildExited,
    StdoutChanged
}

pub struct Cmd {
    child: Option<SubProcess>,
    pub events: VecDeque<CmdEvent>,
    stdout: String,
    stdin: String,
    is_running: bool,
}

impl Cmd {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            stdout: "".to_string(),
            stdin: "".to_string(),
            is_running: true,
            child: None,
        }
    }

    pub fn trigger_stdout_update(&mut self) {
        self.emit(CmdEvent::StdoutChanged);
    }

    fn emit(&mut self, event: CmdEvent) {
        self.events.push_back(event);
    }

    pub fn attach_child(&mut self, child: Option<SubProcess>) {
        self.child = child;
    }
    
    pub fn drain_events(&mut self) -> VecDeque<CmdEvent> {
        let mut replace = VecDeque::new();
        std::mem::swap(&mut replace, &mut self.events);
        replace
    }

    pub fn exit(&mut self) {
        self.is_running = false;
    }

    pub fn is_exited(&self) -> bool {
        return !self.is_running;
    }

    pub fn pop_stdin(&mut self) {
        if self.stdin.pop() != None {
            self.stdout.pop();
            self.emit(CmdEvent::StdoutChanged);
        }
    }

    pub fn put_stdin(&mut self, c: char) {
        self.stdin.push(c);
        self.put_stdout(c);
    }

    pub fn get_stdin(&self) -> &str {
        self.stdin.as_str()
    }

    pub fn get_stdout(&self) -> &str {
        self.stdout.as_str()
    }

    pub fn is_handling_subprocess(&self) -> bool {
        self.child.is_some()
    }

    pub fn destroy_child(&mut self) {
        if let Some(ref mut child) = &mut self.child {
            println!("Child process found, destroying");
            child.kill();
        }
    }

    pub fn clear(&mut self) {
        self.stdout = "".to_string();
    }

    pub fn flush_stdin(&mut self) -> String {
        if let Some(ref mut child) = &mut self.child {
            child.write_stdin(self.stdin.as_bytes());
        }
        let mut old = "".to_string();
        std::mem::swap(&mut self.stdin, &mut old);
        old
    }

    pub fn put_stdout(&mut self, c: char) {
        self.stdout.push(c);
        self.emit(CmdEvent::StdoutChanged);
    }

    pub fn write_stdout(&mut self, s: &str) {
        self.stdout += s;
        self.emit(CmdEvent::StdoutChanged);
    }

    pub fn update(&mut self) {
        let mut process_done = false;

        let this = (self as *mut Self);
        if let Some(ref mut child) = &mut self.child {
            unsafe {
                let s = crate::cp437::from_cp437_if_windows(&child.get_bytes_stdout());
                (*this).write_stdout(&s);
                let s = crate::cp437::from_cp437_if_windows(&child.get_bytes_stderr());
                (*this).write_stdout(&s);
            }
            if child.is_dead() {
                process_done = true;
            }
        }

        if process_done {
            self.events.push_back(CmdEvent::ChildExited);
            self.child = None;
        }

        let n = 50000;
        // To avoid clogging up stdout remove we only leave last 50000 characters
        let amount_to_drain = self.stdout.char_indices().rev().nth(n-1).map_or(0, |(idx, ch)| idx);

        if amount_to_drain > 10000 {
            self.stdout.drain(0..amount_to_drain);
        }
    }
}
