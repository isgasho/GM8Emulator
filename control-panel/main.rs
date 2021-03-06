#![allow(dead_code)]

mod font;
mod panel;

use shared::message::{Message, MessageStream};
use std::{env, path::Path, process};

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

const WINDOW_WIDTH: u32 = 300;
const WINDOW_HEIGHT: u32 = 750;

fn main() {
    process::exit(xmain());
}

fn xmain() -> i32 {
    let args: Vec<String> = env::args().collect();
    let process_name = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "prints this help message");
    opts.optopt("n", "project-name", "name of TAS project to create or load", "NAME");
    opts.optflag("v", "verbose", "enables verbose logging");

    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(fail) => {
            use getopts::Fail::*;
            match fail {
                ArgumentMissing(arg) => eprintln!("missing argument {}", arg),
                UnrecognizedOption(opt) => eprintln!("unrecognized option {}", opt),
                OptionMissing(opt) => eprintln!("missing option {}", opt),
                OptionDuplicated(opt) => eprintln!("duplicated option {}", opt),
                UnexpectedArgument(arg) => eprintln!("unexpected argument {}", arg),
            }
            return EXIT_FAILURE
        },
    };

    if args.len() < 2 || matches.opt_present("h") {
        print!(
            "{}",
            opts.usage(&format!("Usage: {} FILE -n PROJECT-NAME [-v]", match Path::new(&process_name).file_name() {
                Some(file) => file.to_str().unwrap_or(&process_name),
                None => &process_name,
            }))
        );
        return EXIT_SUCCESS
    }

    let verbose = matches.opt_present("v");
    let input = {
        if matches.free.len() == 1 {
            &matches.free[0]
        } else if matches.free.len() > 1 {
            eprintln!("unexpected second input {}", matches.free[1]);
            return EXIT_FAILURE
        } else {
            eprintln!("no input file");
            return EXIT_FAILURE
        }
    };
    let project_name = match matches.opt_str("n") {
        Some(p) => p,
        None => {
            eprintln!("missing required argument: -n project-name");
            return EXIT_FAILURE
        },
    };

    println!("input {}, project name {}, verbose {}", input, project_name, verbose);

    let bind_addr = format!("127.0.0.1:15560");
    println!("Waiting on TCP connection to {}", bind_addr);
    let listener = std::net::TcpListener::bind(bind_addr).unwrap();

    let mut emu = process::Command::new("gm8emulator.exe");
    let _emu_handle = if verbose { emu.arg(input).arg("v") } else { emu.arg(input) }
        .arg("-n")
        .arg(project_name.clone())
        .arg("-p")
        .arg("15560")
        .spawn()
        .expect("failed to execute process");

    let (stream, remote_addr) = listener.accept().unwrap();
    stream.set_nonblocking(true).unwrap();
    println!("Connection established with {}", &remote_addr);

    let mut panel = match panel::ControlPanel::new(stream, &project_name) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error starting control panel: {}", e);
            return EXIT_FAILURE
        },
    };

    let keys = panel.key_buttons.iter().map(|x| x.key).collect::<Vec<_>>();
    let buttons = Vec::new();
    println!("Sending 'Hello' with {} keys, {} mouse buttons", keys.len(), buttons.len());
    panel
        .stream
        .send_message(&Message::Hello {
            keys_requested: keys,
            mouse_buttons_requested: buttons,
            filename: "save.bin".into(),
        })
        .unwrap();
    match panel.await_update() {
        Ok(true) => (),
        Ok(false) => return EXIT_SUCCESS,
        Err(e) => {
            eprintln!("error during handshake: {}", e);
            return EXIT_FAILURE
        },
    };

    loop {
        match panel.update() {
            Ok(true) => (),
            Ok(false) => return EXIT_SUCCESS,
            Err(e) => {
                eprintln!("error during handshake: {}", e);
                return EXIT_FAILURE
            },
        };

        panel.draw();
        if panel.window.close_requested() {
            break
        }
    }

    EXIT_SUCCESS
}
