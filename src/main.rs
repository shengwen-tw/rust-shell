mod tiny_shell;
use tiny_shell::tiny_shell::Shell;

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
