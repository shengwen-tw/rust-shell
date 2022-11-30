use ncurses::*;
use std::char;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::str;

const CMD_LEN_MAX: usize = 50;
const HISTORY_MAX_NUM: usize = 50;

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
    history: LinkedList<String>,
    history_num: isize,
    history_disp_curr: isize,
    read_history: bool,
    typing_preserve: String,
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
            history: LinkedList::new(),
            history_num: 0,
            history_disp_curr: 0,
            read_history: false,
            typing_preserve: String::new(),
        }
    }

    fn start(&self) {
        let win = ncurses::initscr();
        ncurses::raw();
        ncurses::nonl();
        ncurses::noecho();
        ncurses::scrollok(win, true);
    }

    fn add_command(&mut self, cmd_name: &'a str, cmd_func: fn(Vec<&str>, usize)) {
        self.cmds.insert(cmd_name, Box::new(cmd_func));
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

    fn new_line(&self) {
        /* shift the cursor to the line end before switching the new line,
         * otherwise the the user input might be cut */
        let mut cur_x = 0;
        let mut cur_y = 0;
        ncurses::getyx(stdscr(), &mut cur_y, &mut cur_x);
        ncurses::mv(cur_y, (self.prompt_len + self.char_cnt) as i32);
        Shell::puts("\n\r");
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

    fn reset_line_tracking(&mut self) {
        self.cursor_pos = 0;
        self.char_cnt = 0;
    }

    fn reset_history_tracking(&mut self) {
        /* reorder the command in hsitory according to the current display number */
        for _i in 0..self.history_disp_curr {
            let cmd = self.history.pop_back().unwrap();
            self.history.push_front(cmd);
        }

        self.history_disp_curr = 0;
        self.read_history = false;
    }

    fn push_new_history(&mut self, cmd: &String) {
        if self.history_num < (HISTORY_MAX_NUM as isize) {
            self.history.push_front(cmd.clone());
            self.history_num += 1;
            return;
        } else {
            self.history.push_front(cmd.clone());
            self.history.pop_back();
        }
    }

    fn preserve_current_typing(&mut self) {
        let mut cmd = String::new();
        self.get_command_string(&mut cmd);
        self.typing_preserve = cmd;
    }

    #[allow(dead_code)]
    fn print_history(&mut self) {
        Shell::puts("\n\rhistory:");

        for _i in 0..self.history_num {
            /* pop and and print out the last history command */
            let curr_cmd = self.history.pop_front().unwrap();
            Shell::puts(format!("\n\r{}", curr_cmd.as_str()).as_ref());

            //push the command back into the history list
            self.history.push_back(curr_cmd);
        }
    }

    fn get_history_arrow_up(&mut self) {
        /* pop the command from the front of the history list */
        let cmd = self.history.pop_front().unwrap();

        /* display the command by overwriting the buffer */
        for (i, c) in cmd.chars().enumerate() {
            self.buf[i] = c as u32;
        }
        self.char_cnt = cmd.len();

        /* push the command into the back of the history list */
        self.history.push_back(cmd);
    }

    fn get_history_arrow_down(&mut self) {
        /* pop the command from the back of the history list */
        let cmd = self.history.pop_back().unwrap();

        /* display the command by overwriting the buffer */
        for (i, c) in cmd.chars().enumerate() {
            self.buf[i] = c as u32;
        }
        self.char_cnt = cmd.len();

        /* push the command into the front of the history list */
        self.history.push_front(cmd);
    }

    fn restore_user_typing(&mut self) {
        /* restore the user typing by overwriting the buffer */
        for (i, c) in self.typing_preserve.chars().enumerate() {
            self.buf[i] = c as u32;
        }
        self.char_cnt = self.typing_preserve.len();
    }

    fn listen(&mut self) -> String {
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
                    /* reset the history tracking so the command is placed chronologically */
                    self.reset_history_tracking();

                    /* generate the command string for function return */
                    let mut cmd = String::new();
                    self.get_command_string(&mut cmd);

                    /* push command to the history if it is not empty */
                    if self.char_cnt > 0 {
                        self.push_new_history(&cmd);
                    }

                    /* move to next line */
                    self.new_line();
                    self.reset_line_tracking();

                    return cmd;
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
                            /* ignore the event if no command is stored in the history */
                            if self.history_num == 0 {
                                continue;
                            }

                            /* set up the flag to indicate the user triggered the history reading */
                            if self.read_history == false {
                                self.preserve_current_typing(); //save current input words
                                self.history_disp_curr = 0; //counter set zero (i.e., read from the latest record)
                                self.read_history = true; //history reading is on
                            }

                            if self.history_disp_curr < self.history_num {
                                /* display an old command from the history */
                                self.get_history_arrow_up();
                                self.history_disp_curr += 1;
                            } else {
                                /* restore user's typing if the whole list has been traversed */
                                self.restore_user_typing();
                                self.history_disp_curr = 0;
                                self.read_history = false;
                            }

                            /* relocate the cursor position and refresh the line */
                            self.cursor_pos = self.char_cnt;
                            self.refresh_line();
                        } else if seq1 == TermKeys::DownArrow as i32 {
                            /* ignore the event before the up arrow is first pressed */
                            if self.read_history == false {
                                continue;
                            }

                            if self.history_disp_curr >= 1 {
                                /* display an old command from the history */
                                self.get_history_arrow_down();
                                self.history_disp_curr -= 1;
                            } else {
                                /* restore user's typing if the whole list has been traversed */
                                self.restore_user_typing();
                                self.history_disp_curr = 0;
                                self.read_history = false;
                            }

                            /* relocate the cursor position and refresh the line */
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

fn shell_cmd_help(_argc: Vec<&str>, _argv: usize) {
    Shell::puts("help\n\rclear\n\recho\n\r");
}

fn shell_cmd_clear(_argc: Vec<&str>, _argv: usize) {
    Shell::cls();
}

fn shell_cmd_echo(argc: Vec<&str>, argv: usize) {
    for i in 1..argv {
        if i != (argv - 1) {
            Shell::puts(format!("{} ", argc[i]).as_ref());
        } else {
            Shell::puts(format!("{}\n\r", argc[i]).as_ref());
        }
    }
}

fn main() {
    let mut shell = Shell::new("shell > ");
    shell.add_command("help", shell_cmd_help);
    shell.add_command("clear", shell_cmd_clear);
    shell.add_command("echo", shell_cmd_echo);

    shell.start();
    Shell::puts("type 'help' for help\n\r");

    loop {
        let cmd = shell.listen();
        shell.parse(cmd.as_str());
    }
}
