use std::io::Read;
use std::io;

extern crate encoding_rs;
use encoding_rs::CoderResult;
use encoding_rs::DecoderResult;

use encoding_rs::UTF_8;
use encoding_rs::SHIFT_JIS;

use rawbufreader::RawBufReader;

pub struct DecodingReader<R> {
    raw_reader: RawBufReader<R>,
}

impl <R: Read> DecodingReader<R> {
    pub fn new(read: R) -> Self {
        DecodingReader {
            raw_reader: RawBufReader::new(read),
        }
    }

    // 一行分を読み取り、デコードして返す
    pub fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        // バイト列を一行読み込み
        let mut line = Vec::new();
        let nline = self.raw_reader.read_line(&mut line)?;
        if nline == 0 {
            return Ok(0)
        }

        // UTF-8とShift_JIS混在のデコード
        let mut decoded: &mut Vec<u8> = unsafe { buf.as_mut_vec() };
        let nline = decode_utf8_sjis(&line, &mut decoded)?;

        Ok(nline)
    }
}


fn decode_utf8_sjis(src: &Vec<u8>, dst: &mut Vec<u8>) -> io::Result<usize> {
    let mut utf8_decoder = UTF_8.new_decoder();

    let mut buf: Box<[u8]> = unsafe {
        // NOTE: *4は適当、とりあえずのOutputFull回避
        let size = src.len() * 4;
        let mut v = Vec::with_capacity(size);
        v.set_len(size);
        v.into_boxed_slice()
    };

    let mut src_cur = 0usize;
    let src_len = src.len();
    while src_cur < src_len {
        // UTF-8でデコード
        let (result, nread, nwrite) =
            utf8_decoder.decode_to_utf8_without_replacement(&src[src_cur..],
                                                            &mut buf[..],
                                                            false);

        let bad_bytes: usize = if let DecoderResult::Malformed(bad_bytes, _consumed_bytes) = result {
            bad_bytes as usize
        } else {
            0
        };

        if nread > 0 && nread > bad_bytes {
            // Success
            dst.extend_from_slice(&buf[..nwrite]);
            src_cur += nread - bad_bytes;

            match result {
                DecoderResult::InputEmpty => {
                    break; // decode complete
                }
                DecoderResult::OutputFull => {
                    eprintln!("buf size is short");
                    assert!(false); // bufが足りない
                }
                DecoderResult::Malformed(_, _) => {
                    // no-op
                }
            }
        }


        if let DecoderResult::Malformed(_, _) = result {
            if src_cur + 2 <= src_len {
                // Shift_JISの全角文字としてリトライ
                let (result, nread, nwrite) =
                    SHIFT_JIS.new_decoder().decode_to_utf8_without_replacement(&src[src_cur..src_cur+2],
                                                                               &mut buf[..],
                                                                               true);

                let bad_bytes: usize = if let DecoderResult::Malformed(bad_bytes, _consumed_bytes) = result {
                    bad_bytes as usize
                } else {
                    0
                };

                if nread > 0 && nread > bad_bytes {
                    // Success
                    dst.extend_from_slice(&buf[..nwrite]);
                    src_cur += nread - bad_bytes;

                    continue;
                }
            }
        }

        // Failed

        // 不明バイト
        dst.push(0x3f); // "?"
        src_cur += 1;
    }

    // Process EOF
    {
        let (result, _nread, nwrite, _had_errors) =
            utf8_decoder.decode_to_utf8(b"",
                                        &mut buf[..],
                                        true);

        for s in buf[..nwrite].iter() {
            dst.push(*s);
        }

        assert!(result == CoderResult::InputEmpty);
    }

    Ok(dst.len())
}
