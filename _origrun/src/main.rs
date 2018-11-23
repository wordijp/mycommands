fuse std::process::{Child, Command, Stdio};
use std::io::{stderr, stdout};
use std::env;
use std::process;
use std::thread;

extern crate regex;
use regex::Regex;

extern crate encoding_rs;
use encoding_rs::UTF_8;
use encoding_rs::SHIFT_JIS;
use encoding_rs::EUC_JP;

mod rawbufreader;

mod decodingreader;
use decodingreader::DecodingReader;

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
        .arg("call")
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

        let mut out = DecodingReader::new(&[UTF_8, SHIFT_JIS, EUC_JP], cmd.stdout.take().unwrap());
        thread::spawn(move || {
            do_io(&mut out, stdout(), quit_out_s.as_ref(), mtx.as_ref(), stopped.as_ref());
        });
    }

    // stderr
    {
        let stopped = stopped.clone();
        let mtx = mtx.clone();
        let (_quit_err_s, _) = channel::bounded(0);

        let mut err = DecodingReader::new(&[UTF_8, SHIFT_JIS, EUC_JP], cmd.stderr.take().unwrap());
        thread::spawn(move || {
            do_io(&mut err, stderr(), &_quit_err_s, mtx.as_ref(), stopped.as_ref());
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
    reader: &mut DecodingReader<R>,
    mut writer: W,
    quit_s: &channel::Sender<i32>,
    mtx: &Mutex<()>,
    stopped: &AtomicBool,
)
    where R: std::io::Read,
          W: std::io::Write
{
    // UTF-8以外をpanicを出さずに取得する方法
    // TODO: 必要ならば対応
    //let s = unsafe { std::str::from_utf8_unchecked_mut(&mut buf[..nread]) };

    // 行単位の出力
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        if stopped.load(Ordering::SeqCst) {
            return;
        }
        {
            let mut _lock = mtx.lock().unwrap();
            write!(writer, "{}", line);
        }
        let ret = writer.flush();
        if !ret.is_ok() {
            stopped.store(true, Ordering::SeqCst);
            quit_s.send(SIGPIPE);
            return;
        }

        line.clear(); // clear to reuse the buffer
    }

    quit_s.send(SUCCESS);
}
