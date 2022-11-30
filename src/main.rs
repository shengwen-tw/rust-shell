use ncurses::*;
use std::char;
use std::collections::HashMap;
use std::process;
use std::str;

const CMD_LEN_MAX: usize = 50;
const HISTORY_MAX_SIZE: usize = 5;

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
    cursor_pos: usize,
    char_cnt: usize,
    prompt_msg: &'a str,
    prompt_len: usize,
    buf: [u32; CMD_LEN_MAX],
    history_num: isize,
    history_disp_curr: isize,
    read_history: bool,
}

impl<'a> Shell<'a> {
    fn new(prompt_msg: &'a str) -> Shell<'a> {
        Shell {
            cmds: HashMap::new(),
            cursor_pos: 0,
            char_cnt: 0,
            prompt_msg,
            prompt_len: prompt_msg.len(),
            buf: [0; CMD_LEN_MAX],
            history_num: 0,
            history_disp_curr: 0,
            read_history: false,
        }
    }

    fn start(&self) {
        ncurses::initscr();
        ncurses::raw();
        ncurses::nonl();
        ncurses::noecho();
    }

    fn getc() -> i32 {
        ncurses::getch()
    }

    fn puts(s: &str) {
        ncurses::addstr(s);
    }

    fn cls() {
        ncurses::clear();
    }

    fn ctrl_c_handler(&self) {
        ncurses::endwin();
        std::process::exit(0);
    }

    fn insert_char(&mut self, c: i32) {
        for i in (self.cursor_pos + 1..self.char_cnt + 1).rev() {
            self.buf[i] = self.buf[i - 1];
        }

        self.char_cnt += 1;
        self.buf[self.char_cnt] = 0;

        self.buf[self.cursor_pos] = c as u32;
        self.cursor_pos += 1;
    }

    fn remove_char(&mut self, remove_pos: usize, cursor_fixed: bool) {
        for i in (remove_pos - 1)..(self.char_cnt) {
            self.buf[i] = self.buf[i + 1];
        }

        self.buf[self.char_cnt] = 0;
        self.char_cnt -= 1;

        /* cursor shift left by on only if the remove event is triggered by the backspace */
        if cursor_fixed == false {
            self.cursor_pos -= 1;
        }

        if self.cursor_pos > self.char_cnt {
            self.cursor_pos = self.char_cnt;
        }
    }

    fn get_command_string(&self, cmd_ref: &mut String) {
        for i in 0..self.char_cnt {
            let c = char::from_u32(self.buf[i] as u32).unwrap();
            cmd_ref.push(c);
        }
    }

    fn refresh_line(&self) {
        /* clear the current line */
        let mut cur_x = 0;
        let mut cur_y = 0;
        ncurses::getyx(stdscr(), &mut cur_y, &mut cur_x);
        ncurses::mv(cur_y, 0);
        ncurses::clrtoeol();

        /* print prompt message */
        Shell::puts(self.prompt_msg);

        /* print user input */
        let mut cmd = String::new();
        self.get_command_string(&mut cmd);
        Shell::puts(cmd.as_str());

        /* shift cursor position */
        ncurses::mv(cur_y, (self.prompt_len + self.cursor_pos) as i32);
    }

    fn cursor_shift_one_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.refresh_line();
        }
    }

    fn cursor_shift_one_right(&mut self) {
        if self.cursor_pos < self.char_cnt {
            self.cursor_pos += 1;
            self.refresh_line();
        }
    }

    fn reset_data(&mut self) {
        self.cursor_pos = 0;
        self.char_cnt = 0;
        self.read_history = false;
    }

    fn push_new_history(&mut self) {
        //shell_history_t *curr_history;

        if self.history_num < (HISTORY_MAX_SIZE as isize) {
            //curr_history = &self.history[HISTORY_MAX_SIZE - self.history_num - 1];
            //strcpy(self.cmd, cmd);
            self.history_num += 1;
            //self.history_top = curr_history;
            return;
        }

        /* if history list is full, drop the oldest one */
        //shell_history_t *history_end = shell->history_top;
        for i in 0..(HISTORY_MAX_SIZE - 1) {
            //if(history_end.cmd[0] == 0) {
            //    break;
            //}
            //history_end = history_end.next;
        }
        //strcpy(history_end.cmd, self.buf);
        //self.history_top = history_end;
    }

    fn listen(&mut self) -> Option<String> {
        Shell::puts(self.prompt_msg);

        loop {
            let c = Shell::getc();
            //Shell::puts(format!("read {}", c).as_ref());

            match c {
                c if c == TermKeys::NullCh as i32 => continue,
                c if c == TermKeys::CtrlA as i32 => {
                    self.cursor_pos = 0;
                    self.refresh_line();
                    continue;
                }
                c if c == TermKeys::CtrlB as i32 => {
                    self.cursor_shift_one_left();
                    continue;
                }
                c if c == TermKeys::CtrlC as i32 => {
                    self.ctrl_c_handler();
                    return None;
                }
                c if c == TermKeys::CtrlD as i32 => continue,
                c if c == TermKeys::CtrlE as i32 => {
                    if self.char_cnt > 0 {
                        self.cursor_pos = self.char_cnt;
                        self.refresh_line();
                    }
                    continue;
                }
                c if c == TermKeys::CtrlF as i32 => {
                    self.cursor_shift_one_right();
                    continue;
                }
                c if c == TermKeys::CtrlG as i32 => continue,
                c if c == TermKeys::CtrlH as i32 => continue,
                c if c == TermKeys::Tab as i32 => continue,
                c if c == TermKeys::CtrlJ as i32 => continue,
                c if c == TermKeys::Enter as i32 => {
                    /* generate the command string for function return */
                    let mut cmd = String::new();
                    self.get_command_string(&mut cmd);

                    /* push command to the history if it is not empty */
                    if self.char_cnt > 0 {
                        self.push_new_history();
                    }

                    /* move to next line */
                    self.reset_data();
                    Shell::puts("\n\r");

                    return Some(cmd);
                }
                c if c == TermKeys::CtrlK as i32 => continue,
                c if c == TermKeys::CtrlL as i32 => continue,
                c if c == TermKeys::CtrlN as i32 => continue,
                c if c == TermKeys::CtrlO as i32 => continue,
                c if c == TermKeys::CtrlP as i32 => continue,
                c if c == TermKeys::CtrlQ as i32 => continue,
                c if c == TermKeys::CtrlR as i32 => continue,
                c if c == TermKeys::CtrlS as i32 => continue,
                c if c == TermKeys::CtrlT as i32 => continue,
                c if c == TermKeys::CtrlU as i32 => {
                    self.buf[0] = 0;
                    self.char_cnt = 0;
                    self.cursor_pos = 0;
                    self.refresh_line();
                    continue;
                }
                c if c == TermKeys::CtrlW as i32 => continue,
                c if c == TermKeys::CtrlX as i32 => continue,
                c if c == TermKeys::CtrlY as i32 => continue,
                c if c == TermKeys::CtrlZ as i32 => continue,
                c if c == TermKeys::EscSeq1 as i32 => {
                    let seq0 = Shell::getc();
                    let seq1 = Shell::getc();
                    if seq0 == TermKeys::EscSeq2 as i32 {
                        if seq1 == TermKeys::UpArrow as i32 {
                            if self.history_num == 0 {
                                continue;
                            }

                            if self.read_history == false {
                                //strcpy(self.typing_preserve, self.buf);
                                //self.history_disp = self.history_top;
                                self.history_disp_curr = 0;
                            } else {
                                //self.history_disp = self.history_disp.next;
                            }

                            /* restore user's typing if finished traveling through the whole list */
                            if self.history_disp_curr < self.history_num {
                                //strcpy(self.buf, self.history_disp.cmd);
                                self.history_disp_curr += 1;
                            } else {
                                //strcpy(self.buf, self.history.cmd);
                                //self.history_disp = self.history_top;
                                self.history_disp_curr = 0;
                                self.read_history = false;
                            }

                            //self.char_cnt = strlen(self.buf);
                            self.cursor_pos = self.char_cnt;
                            self.refresh_line();
                        } else if seq1 == TermKeys::DownArrow as i32 {
                            if self.read_history == false {
                                continue;
                            } else {
                                //self.history_disp = self.history_disp.last;
                            }

                            /* restore user's typing if finished traveling through the whole list */
                            if self.history_disp_curr > 1 {
                                //strcpy(self.buf, self.history_disp.cmd);
                                self.history_disp_curr -= 1;
                            } else {
                                //strcpy(self.buf, self.typing_preserve);
                                //self.history_disp = self.history_top;
                                self.history_disp_curr = 0;
                                self.read_history = false;
                            }

                            //self.char_cnt = strlen(self.buf);
                            self.cursor_pos = self.char_cnt;
                            self.refresh_line();
                        } else if seq1 == TermKeys::RightArrow as i32 {
                            self.cursor_shift_one_right();
                        } else if seq1 == TermKeys::LeftArrow as i32 {
                            self.cursor_shift_one_left();
                        } else if seq1 == TermKeys::HomeXterm as i32 {
                            self.cursor_pos = 0;
                            self.refresh_line();
                        } else if seq1 == TermKeys::HomeVt100 as i32 {
                            self.cursor_pos = 0;
                            self.refresh_line();
                            Shell::getc();
                        } else if seq1 == TermKeys::EndXterm as i32 {
                            if self.char_cnt > 0 {
                                self.cursor_pos = self.char_cnt;
                                self.refresh_line();
                            }
                        } else if seq1 == TermKeys::EndVt100 as i32 {
                            if self.char_cnt > 0 {
                                self.cursor_pos = self.char_cnt;
                                self.refresh_line();
                            }
                        } else if seq1 == TermKeys::Delete as i32 {
                            let seq = Shell::getc();
                            if seq == TermKeys::EscSeq4 as i32
                                && self.char_cnt != 0
                                && self.cursor_pos != self.char_cnt
                            {
                                self.remove_char(self.cursor_pos + 1, true);
                                self.refresh_line();
                            }
                        }
                    }
                    continue;
                }
                c if c == TermKeys::Backspace as i32 => {
                    if (self.char_cnt != 0) && (self.cursor_pos != 0) {
                        self.remove_char(self.cursor_pos, false);
                        self.refresh_line();
                    }
                    continue;
                }
                _ => {
                    if self.char_cnt != (CMD_LEN_MAX - 1) {
                        self.read_history = false;
                        self.insert_char(c);
                        self.refresh_line();
                    }
                    continue;
                }
            };
        }
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
            None => Shell::puts("unknown command.\n\r"),
        };
    }
}

fn shell_cmd_help(argc: Vec<&str>, argv: usize) {
    Shell::puts("argc: ");

    for arg in argc {
        Shell::puts(format!("{} ", arg).as_ref());
    }

    Shell::puts(format!("\n\rargv: {}\n\r", argv).as_ref());
}

fn shell_cmd_clear(argc: Vec<&str>, argv: usize) {
    Shell::cls();
}

fn main() {
    let mut shell = Shell::new("shell > ");
    shell.add_command("help", shell_cmd_help);
    shell.add_command("clear", shell_cmd_clear);

    shell.start();
    loop {
        let cmd = shell.listen().unwrap();
        shell.parse(cmd.as_str());
    }
}
