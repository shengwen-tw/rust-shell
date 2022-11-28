use std::collections::HashMap;

struct Shell<'a> {
    cmds: HashMap<&'a str, Box<dyn Fn(Vec<&str>, usize)>>,
}

impl<'a> Shell<'a> {
    fn new() -> Shell<'a> {
        Shell {
            cmds: HashMap::new(),
        }
    }

    fn add_command(&mut self, cmd_name: &'a str, cmd_func: fn(Vec<&str>, usize)) {
        self.cmds.insert(cmd_name, Box::new(cmd_func));
    }

    fn parse(&self, cmd: &str) {
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

fn shell_cmd_help(_argc: Vec<&str>, _argv: usize) {
    println!("help");
}

fn shell_cmd_clear(_argc: Vec<&str>, _argv: usize) {
    println!("clear");
}

fn main() {
    let mut shell = Shell::new();
    shell.add_command("help", shell_cmd_help);
    shell.add_command("clear", shell_cmd_clear);

    shell.parse("clear 1 2 3");
}
