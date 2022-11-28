use std::collections::HashMap;

struct Shell<'a> {
    cmds: HashMap<&'a str, Box<dyn Fn()>>,
}

impl<'a> Shell<'a> {
    fn new() -> Shell<'a> {
        Shell {
            cmds: HashMap::new(),
        }
    }

    fn add_command(&mut self, cmd_name: &'a str, cmd_func: fn()) {
        self.cmds.insert(cmd_name, Box::new(cmd_func));
    }

    fn parse(&self, cmd: &str) {
        match self.cmds.get(cmd) {
            Some(cmd_func) => cmd_func(),
            None => {
                println!("unknown command.");
            }
        }
    }
}

fn shell_cmd_help() {
    println!("help");
}

fn shell_cmd_clear() {
    println!("clear");
}

fn main() {
    let mut shell = Shell::new();
    shell.add_command("help", shell_cmd_help);
    shell.add_command("clear", shell_cmd_clear);

    shell.parse("clear");
}
