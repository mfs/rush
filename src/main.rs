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

use std::process::Command;
use std::io::Write;

fn main() {

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let mut exit_status: i32 = 0;

    loop {
        print!("{} - $ ", exit_status);
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        if input == "exit\n" {
            break;
        }

        let mut args: Vec<&str> = input.split(char::is_whitespace).filter(|&x| x != "").collect();

        let command = args.remove(0);

        let mut child = match Command::new(command).args(&args).spawn() {
            Ok(c) => c,
            Err(_) => {
                println!("rush: command not found");
                continue;
            }
        };

        match child.wait() {
            Ok(s) => { exit_status = s.code().unwrap_or(0); },
            Err(_) => { println!("rush: process killed by signal"); },
        };
    }
}
