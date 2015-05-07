/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.
This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::process::{Stdio, Command};
use std::fs::File;
use std::io::Write;

fn parsefile(cmd: &mut Vec<&str>, delimiter: &str) -> Option<String> {
    let idx = cmd.iter().position(|x| *x == delimiter);

    match idx {
        Some(x) => {
            let filename = cmd.remove(x + 1);
            cmd.remove(x);
            return Some(filename.to_string());
        },
        None => return None,
    }
}

fn main() {

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    const CMD_NOT_FOUND: i32 = 127;
    let mut exit_status: i32 = 0;

    const IN_REDIRECT_SYMBOL: &'static str = "<";
    const OUT_REDIRECT_SYMBOL: &'static str = ">";

    loop {
        print!("{} - $ ", exit_status);
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        if input == "exit\n" {
            break;
        }

        let mut args: Vec<&str> = input.split(char::is_whitespace).filter(|&x| x != "").collect();

        let in_filename = parsefile(&mut args, IN_REDIRECT_SYMBOL);
        let out_filename = parsefile(&mut args, OUT_REDIRECT_SYMBOL);

        let mut cmd = Command::new(args.remove(0));

        cmd.args(&args);

        if out_filename.is_some() {
            cmd.stdout(Stdio::piped());
        }

        if in_filename.is_some() {
            cmd.stdin(Stdio::piped());
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(_) => {
                exit_status = CMD_NOT_FOUND;
                println!("rush: command not found");
                continue;
            }
        };

        // I would like to use std::io::Tee here which I'm guessing does a
        // standard dup2() however it is unstable. I'll look at the nix
        // crate as an option. Having the parent process buffer data
        // between the files and processes is not ideal.

        if in_filename.is_some() {
            let mut f = File::open(in_filename.unwrap()).unwrap();
            let x: &mut std::process::ChildStdin = child.stdin.as_mut().unwrap();
            std::io::copy(&mut f, x).unwrap();
        }

        if out_filename.is_some() {
            let mut f = File::create(out_filename.unwrap()).unwrap();
            let x: &mut std::process::ChildStdout = child.stdout.as_mut().unwrap();
            std::io::copy(x, &mut f).unwrap();
        }

        match child.wait() {
            Ok(s) => { exit_status = s.code().unwrap_or(0); },
            Err(_) => { println!("rush: process killed by signal"); },
        };
    }
}
