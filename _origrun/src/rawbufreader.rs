// バイト列BufReader

extern crate memchr;

use std::io::Result;
use std::io::ErrorKind;
use std::io::Read;
use std::io::BufRead;
use std::cmp;

const DEFAULT_BUF_SIZE: usize = 8 * 1024;

// バイト列のまま読み込む
pub struct RawBufReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl <R: Read> RawBufReader<R> {
    pub fn new(read: R) -> Self {
        RawBufReader::with_capacity(DEFAULT_BUF_SIZE, read)
    }

    pub fn with_capacity(cap: usize, read: R) -> Self {
        unsafe {
            let mut buf = Vec::with_capacity(cap);
            buf.set_len(cap);

            RawBufReader {
                inner: read,
                buf: buf.into_boxed_slice(),
                pos: 0,
                cap: 0,
            }
        }
    }

    // 一行読み込み
    // 改行 or EOFまでを読み込む
    pub fn read_line(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        append_to_raw(buf, |b| read_until(self, b'\n', b))
    }
}

impl<R: Read> Read for RawBufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }
}

// NOTE: BufReaderの方法を流用
impl<R: Read> BufRead for RawBufReader<R> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        // 使い切った
        if self.pos >= self.cap {
            debug_assert!(self.pos == self.cap);
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }
}

// ------------------------------
// original) std::io::BufRead ---

fn append_to_raw<F>(buf: &mut Vec<u8>, f: F) -> Result<usize>
    where F: FnOnce(&mut Vec<u8>) -> Result<usize>
{
    f(buf)
}

// delim or EOFまで読み込む
fn read_until<R: BufRead>(r: &mut R, delim: u8, buf: &mut Vec<u8>)
   -> Result<usize>
{
    let mut read = 0;
    loop {
        let (done, used) = {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e)
            };
            match memchr::memchr(delim, available) {
                Some(i) => {
                    buf.extend_from_slice(&available[..i + 1]);
                    (true, i + 1)
                }
                None => {
                    buf.extend_from_slice(available);
                    (false, available.len())
                }
            }
        };
        r.consume(used);
        read += used;
        if done || used == 0 {
            return Ok(read);
        }
    }
}
