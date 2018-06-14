use std::time::{SystemTime, UNIX_EPOCH};


mod cmd;

fn main() {
    let mut command = cmd::Cmd {
        command: "ls -al ~/".to_string(),
        stdout: "".to_string(),
        stderr: "".to_string() 
    };

    command.run();

    if command.stdout.len() > 0{
        print!("stdout:\n{}", command.stdout);
    }

    if command.stderr.len() > 0{
        println!("stderr:\n{}", command.stderr);
    }

    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("{:?}", since_the_epoch);
}