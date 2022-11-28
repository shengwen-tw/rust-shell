use std::collections::HashMap;
use std::io::{self, Write};

enum TermKeys {
    NullCh = 0,      /* null character */
    CtrlA = 1,       /* ctrl + a */
    CtrlB = 2,       /* ctrl + b */
    CtrlC = 3,       /* ctrl + c */
    CtrlD = 4,       /* ctrl + d */
    CtrlE = 5,       /* ctrl + e */
    CtrlF = 6,       /* ctrl + f */
    CtrlG = 7,       /* ctrl + g */
    CtrlH = 8,       /* ctrl + h */
    Tab = 9,         /* tab */
    CtrlJ = 10,      /* ctrl + j */
    CtrlK = 11,      /* ctrl + k */
    CtrlL = 12,      /* ctrl + l */
    Enter = 13,      /* enter */
    CtrlN = 14,      /* ctrl + n */
    CtrlO = 15,      /* ctrl + o */
    CtrlP = 16,      /* ctrl + p */
    CtrlQ = 17,      /* ctrl + r */
    CtrlR = 18,      /* ctrl + r */
    CtrlS = 19,      /* ctrl + s */
    CtrlT = 20,      /* ctrl + t */
    CtrlU = 21,      /* ctrl + u */
    CtrlW = 23,      /* ctrl + w */
    CtrlX = 24,      /* ctrl + x */
    CtrlY = 25,      /* ctrl + y */
    CtrlZ = 26,      /* ctrl + z */
    EscSeq1 = 27,    /* first byte of the vt100/xterm escape sequence */
    Space = 32,      /* space */
    Delete = 51,     /* delete, third byte of the xterm escape sequence */
    UpArrow = 65,    /* up arrow, third byte of the xterm escape sequence */
    DownArrow = 66,  /* down arrow, third byte of the xterm escape sequence */
    RightArrow = 67, /* right arrow, third byte of the xterm escape sequence */
    LeftArrow = 68,  /* left arrow, third byte of the xterm escape sequence */
    EndXterm = 70,   /* end, third byte of the xterm escape sequence */
    EndVt100 = 52,   /* end, third byte of the vt100 escape sequence */
    HomeXterm = 72,  /* home, third byte of the escape sequence */
    HomeVt100 = 49,  /* home, third byte of the vt100 escape sequence */
    EscSeq2 = 91,    /* second byte of the escape sequence */
    EscSeq4 = 126,   /* fourth byte of the vt100 escape sequence */
    Backspace = 127, /* backspace */
}

struct Shell<'a> {
    cmds: HashMap<&'a str, Box<dyn Fn(Vec<&str>, usize)>>,
}

impl<'a> Shell<'a> {
    fn new() -> Shell<'a> {
        Shell {
            cmds: HashMap::new(),
        }
    }

    fn getc() -> char {
        ' '
    }

    fn puts(s: &str) {
        io::stdout().write(s.as_bytes());
    }

    fn cls() {
        Shell::puts("\x1b[H\x1b[2J");
    }

    fn add_command(&mut self, cmd_name: &'a str, cmd_func: fn(Vec<&str>, usize)) {
        self.cmds.insert(cmd_name, Box::new(cmd_func));
    }

    fn parse(&mut self, cmd: &str) {
        /* split string into vector of arguments */
        let argc: Vec<&str> = cmd.split_whitespace().collect();
        let argv = argc.len();

        /* get first element of the argc vector */
        let argc_0 = match argc.first() {
            Some(cmd_name) => cmd_name, //get string of the argc[0]
            None => return,             //empty string
        };

        /* match command */
        match self.cmds.get(argc_0) {
            Some(cmd_func) => cmd_func(argc, argv),
            None => println!("unknown command."),
        };
    }
}

fn shell_cmd_help(argc: Vec<&str>, argv: usize) {
    print!("argc: ");
    for arg in argc {
        print!("{} ", arg);
    }

    println!("\n\rargv: {}", argv);
}

fn shell_cmd_clear(argc: Vec<&str>, argv: usize) {
    Shell::cls();

    print!("argc: ");
    for arg in argc {
        print!("{} ", arg);
    }

    println!("\n\rargv: {}", argv);
}

fn main() {
    let mut shell = Shell::new();
    shell.add_command("help", shell_cmd_help);
    shell.add_command("clear", shell_cmd_clear);

    let stdin = io::stdin();
    let mut new_cmd = String::new();
    loop {
        stdin.read_line(&mut new_cmd);
        shell.parse(new_cmd.as_str());
        new_cmd.clear();
    }
}
