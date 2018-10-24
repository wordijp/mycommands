use std::process::{Child, Command, Stdio};
use std::io::{stderr, stdout, BufRead, BufReader};
use std::env;
use std::process;
use std::thread;

extern crate regex;
use regex::Regex;

// スレッド間通信
//#[macro_use] // for select!
extern crate crossbeam_channel as channel;

// シグナル
extern crate ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const SUCCESS: i32 = 0;
const SIGINT: i32 = 2;
const SIGPIPE: i32 = 13;

fn main() {
    let mut _args = env::args();
    // get bat & args
    let bin = _args.nth(0).unwrap();
    let re = Regex::new(r"\.[eE][xX][eE]$").unwrap();
    let bat: String = (re.replace(&bin, "") + ".bat").to_string();
    let args: Vec<_> = _args.collect();

    let mut cmd: Child = Command::new("cmd")
        .arg("/c")
        .arg(bat)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mtx = Arc::new(Mutex::new(()));
    let stopped = Arc::new(AtomicBool::new(false));
    
    let (quit_out_s, quit_out_r) = channel::bounded(0);
    let quit_out_s = Arc::new(quit_out_s);

    // stdout
    {
        let stopped = stopped.clone();
        let mtx = mtx.clone();
        let quit_out_s = quit_out_s.clone();
        let out = BufReader::new(cmd.stdout.take().unwrap());
        thread::spawn(move || {
            do_io(out, stdout(), quit_out_s.as_ref(), mtx.as_ref(), stopped.as_ref());
        });
    }

    // stderr
    {
        let stopped = stopped.clone();
        let mtx = mtx.clone();
        let err = BufReader::new(cmd.stderr.take().unwrap());
        let (_quit_err_s, _) = channel::bounded(0);
        thread::spawn(move || {
            do_io(err, stderr(), &_quit_err_s, mtx.as_ref(), stopped.as_ref());
        });
    }

    // Ctrl-C
    {
        let quit_out_s = quit_out_s.clone();
        let stopped = stopped.clone();
        ctrlc::set_handler(move || {
            stopped.store(true, Ordering::SeqCst);
            quit_out_s.send(SIGINT);
        }).expect("Error setting Ctrl-C handler");
    }

    // wait cmd
    let cmd = Arc::new(Mutex::new(cmd));
    let (cmd_wait_s, cmd_wait_r) = channel::bounded(0);
    {
        let cmd = cmd.clone();
        thread::spawn(move || {
            let status = quit_out_r.recv().unwrap();
            //_quit_err_r.recv().unwrap();
            if status == SUCCESS {
                // 正常終了
                cmd_wait_s.send(SUCCESS);
            } else {
                cmd.lock().unwrap().kill().expect("Error cmd kill");
                cmd_wait_s.send(status);
            }
        });
    }

    let status_code = cmd_wait_r.recv().unwrap();
    process::exit(status_code);
}

fn do_io<R, W>(
    reader: BufReader<R>,
    mut writer: W,
    quit_s: &channel::Sender<i32>,
    mtx: &Mutex<()>,
    stopped: &AtomicBool,
)
    where R: std::io::Read,
          W: std::io::Write
{
    let mut it = reader.lines().into_iter();
    loop {
        match it.next() {
            None => {
                quit_s.send(SUCCESS);
                return;
            }
            Some(msg) => {
                if stopped.load(Ordering::SeqCst) {
                    return;
                }
                {
                    let mut _lock = mtx.lock().unwrap();
                    writeln!(writer, "{}", msg.unwrap());
                }
                let ret = writer.flush();
                if !ret.is_ok() {
                    stopped.store(true, Ordering::SeqCst);
                    quit_s.send(SIGPIPE);
                    return;
                }
            }
        }
    }
}
