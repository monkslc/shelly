use signal_hook::{iterator::Signals, SIGINT};
use std::{
    env,
    io::{self, Write},
    path::Path,
    process::{self, Stdio},
    thread,
};

fn main() {
    handle_sigint();

    loop {
        match get_input() {
            Ok(input) => {
                let mut prev_command: Option<process::Child> = None;

                let mut commands = input.split('|').peekable();
                while let Some(command) = commands.next() {
                    let mut args = command.split_whitespace();
                    if let Some(command_name) = args.next() {
                        match command_name {
                            "exit" => goodbye(),
                            "cd" => match args.next() {
                                Some(path) => change_curr_dir(path),
                                None => change_curr_dir("/"),
                            },
                            _ => {
                                match create_child(
                                    command_name,
                                    args,
                                    prev_command,
                                    commands.peek().is_some(),
                                ) {
                                    Ok(child) => prev_command = Some(child),
                                    Err(err) => {
                                        println!("Err: {}", err);
                                        prev_command = None;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                // wait for the last command to finish output
                if let Some(mut final_command) = prev_command {
                    final_command.wait().unwrap();
                }
            }
            Err(err) => println!("Error trying to read input.\n{}", err),
        }
    }
}

fn get_input() -> Result<String, io::Error> {
    let mut buffer = String::new();
    print!("shelly> ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
}

fn handle_sigint() {
    // basically just ignore sigint
    let signals = Signals::new(&[SIGINT]);
    match signals {
        Ok(signals) => {
            thread::spawn(move || for _ in signals.forever() {});
        }
        Err(err) => {
            panic!("Error starting shell.\n{}", err);
        }
    }
}

fn goodbye() {
    let goodbye_message = String::from_utf8(
        (0..3)
            .map(|_| vec![0xF0, 0x9F, 0x98, 0x94])
            .into_iter()
            .flatten()
            .collect(),
    )
    .unwrap();
    println!("{}", goodbye_message);
    std::process::exit(0);
}

fn change_curr_dir(path: &str) {
    let new_dir = Path::new("").join(Path::new(path));
    if let Err(err) = env::set_current_dir(new_dir) {
        println!("Couldn't change dir\n{}", err);
    } else {
        if let Ok(current_dir) = env::current_dir() {
            println!("{:?}", current_dir);
        }
    }
}

fn create_child(
    command_name: &str,
    args: std::str::SplitWhitespace,
    prev_command: Option<process::Child>,
    is_last: bool,
) -> std::io::Result<process::Child> {
    let stdin = match prev_command {
        Some(prev) => Stdio::from(prev.stdout.unwrap()),
        None => Stdio::inherit(),
    };

    let stdout = match is_last {
        true => Stdio::piped(),
        _ => Stdio::inherit(),
    };

    process::Command::new(command_name)
        .args(args)
        .stdin(stdin)
        .stdout(stdout)
        .spawn()
}
