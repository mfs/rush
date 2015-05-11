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

extern crate libc;

use std::io::Write;
use std::ffi::CString;

const CMD_NOT_FOUND: i32 = 127;
const IN_REDIRECT_SYMBOL: &'static str = "<";
const OUT_REDIRECT_SYMBOL: &'static str = ">";

extern {
    fn wait(stat_loc: *const libc::c_int) -> libc::pid_t;
}

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

fn redirect(in_filename: Option<String>, out_filename: Option<String>) -> bool {

    if in_filename.is_some() {
        let filename = in_filename.unwrap();
        unsafe {
            let fd = libc::open(filename.as_ptr() as *const i8, libc::O_RDONLY, 0o600);
            if fd < 0 {
                return false;
            }
            if libc::dup2(fd, libc::STDIN_FILENO) == -1 {
                libc::close(fd);
                return false;
            }
            libc::close(fd);
        }
    }

    if out_filename.is_some() {
        let filename = out_filename.unwrap();
        unsafe {
            let fd = libc::open(filename.as_ptr() as *const i8, libc::O_WRONLY | libc::O_CREAT, 0o600);
            if fd < 0 {
                return false;
            }
            if libc::dup2(fd, libc::STDOUT_FILENO) == -1 {
                libc::close(fd);
                return false;
            }
            libc::close(fd);
        }
    }

    true
}

fn executecmdline(args: &mut Vec<&str>) -> ! {

    let in_filename = parsefile(args, IN_REDIRECT_SYMBOL);
    let out_filename = parsefile(args, OUT_REDIRECT_SYMBOL);

    if !redirect(in_filename, out_filename) {
        println!("Redirection failed.");
        std::process::exit(1);
    }

    let arg_cstr: Vec<CString> = args.iter().map(|&x| CString::new(x).unwrap()).collect();

    let mut arg: Vec<*const i8> = arg_cstr.iter().map(|x| x.as_ptr()).collect();
    arg.push(std::ptr::null());

    unsafe { libc::execvp(arg[0], arg.as_mut_ptr()) };

    std::process::exit(CMD_NOT_FOUND);
}

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


        let pid = unsafe { libc::fork() };

        if pid == 0 {
            executecmdline(&mut args);
        } else if pid > 0 {
            let status: libc::c_int = 0;
            unsafe { wait(&status) };
            exit_status = (status & 0xff00) >> 8; // include/bits/waitstatus.h
        }
    }
}
