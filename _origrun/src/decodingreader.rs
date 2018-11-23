use std::io::Read;
use std::io;

extern crate encoding_rs;
use encoding_rs::Encoding;
use encoding_rs::Decoder;
use encoding_rs::DecoderResult;

use encoding_rs::UTF_8;
use encoding_rs::SHIFT_JIS;
use encoding_rs::EUC_JP;

use rawbufreader::RawBufReader;

pub struct DecodingReader<R> {
    encodings: Box<[&'static Encoding]>,
    raw_reader: RawBufReader<R>,

    //decoders: Box<[Decoder]>,
}

// デコードリスト
struct DecodeBuf {
    // 元のバイト列のread_from - read_to、
    // デコード先bufのwrite_from - write_toの紐づけリスト
    read_from: Vec<usize>,
    read_to: Vec<usize>,
    write_from: Vec<usize>,
    write_to: Vec<usize>,
    write_buf: Vec<u8>,
}

impl <R: Read> DecodingReader<R> {
    pub fn new(encodings: &[&'static Encoding], read: R) -> Self {
        assert!(encodings.len() > 0);

        //let mut decoders: Vec<Decoder> = self.encodings.iter().map(|d| d.new_decoder()).collect();

        //let decoders: Vec<Decoder> = encodings.iter().map(|enc| enc.new_decoder()).collect();

        DecodingReader {
            encodings: encodings.to_vec().into_boxed_slice(),
            raw_reader: RawBufReader::new(read),

            //decoders: decoders.into_boxed_slice(),
        }
    }

    #[allow(dead_code)]
    // エンコーディングのデフォルトプリセットを指定
    pub fn new_default(read: R) -> Self {
        DecodingReader::new(&[UTF_8, SHIFT_JIS, EUC_JP], read)
    }

    // 一行分を読み取り、デコードして返す
    pub fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        // バイト列を一行読み込み
        let mut line = Vec::new();
        let nread = self.raw_reader.read_line(&mut line)?;
        if nread == 0 {
            return Ok(0)
        }

        // 各エンコーディングでデコードする
        let decode_bufs: Vec<DecodeBuf> = self.encodings.iter().map(|enc| decode_to_utf8_for_decodebuf_from_encoding(&line, enc).unwrap()).collect();

        // デコード結果を合成する
        let mut decoded: &mut Vec<u8> = unsafe { buf.as_mut_vec() };
        let nread = merge_decodebufs(&line, &decode_bufs, &mut decoded)?;
        Ok(nread)
    }


}

// バイト列を指定エンコーディングでデコードする
fn decode_to_utf8_for_decodebuf_from_encoding(src: &Vec<u8>, encoding: &'static Encoding) -> io::Result<DecodeBuf> {
    let mut decode_buf = DecodeBuf {
        read_from: Vec::new(),
        read_to: Vec::new(),
        write_from: Vec::new(),
        write_to: Vec::new(),
        write_buf: Vec::new(),
    };

    let mut decoder: Decoder = encoding.new_decoder();

    let mut buf: Box<[u8]> = unsafe {
        // NOTE: 適当
        //       UTF-8は最大4バイト文字のため * 4
        //       他のエンコーディングで1バイトで表される古代文字、漢字が無いと保証出来れば減らせる
        let size = src.len() * 4;
        let mut v = Vec::with_capacity(size);
        v.set_len(size);
        v.into_boxed_slice()
    };

    let mut src_cur: usize = 0usize;
    loop {
        // NOTE: nread: 読み込みバイト数(bad_bytes含む)
        //       nwrite: デコード成功バイト数
        let (result, nread, nwrite) =
            decoder.decode_to_utf8_without_replacement(&src[src_cur..],
                                                       &mut buf[..],
                                                       false);

        // 不正バイト数
        // NOTE: bad_bytesだけ使えばよい(consumed_bytesはISO-2022-JPで値が入るが、不要)
        let bad_bytes: usize = if let DecoderResult::Malformed(bad_bytes, _consumed_bytes) = result {
            bad_bytes as usize
        } else {
            0
        };

        // デコード成功領域を取得
        if nread > 0 {
            {
                let from = src_cur;
                let to = from + (nread - bad_bytes);
                decode_buf.read_from.push(from);
                decode_buf.read_to.push(to);
            }
            {
                let from = decode_buf.write_from.len();
                let to = from + nwrite;
                decode_buf.write_from.push(from);
                decode_buf.write_to.push(to);

                for s in buf[..nwrite].iter() {
                    decode_buf.write_buf.push(*s);
                }
            }
        }

        src_cur += nread;
        match result {
            DecoderResult::InputEmpty => {
                break;
            }
            DecoderResult::OutputFull => {
                assert!(false); // bufのsizeが足りなかった
            }
            DecoderResult::Malformed(_, _) => {
                continue; // no-op
            }
        }
    }

    // Process EOF
    loop {
        let (result, nread, nwrite) =
            decoder.decode_to_utf8_without_replacement(b"",
                                                       &mut buf[..],
                                                       true);

        let bad_bytes: usize = if let DecoderResult::Malformed(bad_bytes, _consumed_bytes) = result {
            bad_bytes as usize
        } else {
            0
        };

        if nread > 0 {
            {
                let from = src_cur;
                let to = from + (nread - bad_bytes);
                decode_buf.read_from.push(from);
                decode_buf.read_to.push(to);
            }
            {
                let from = decode_buf.write_from.len();
                let to = from + nwrite;
                decode_buf.write_from.push(from);
                decode_buf.write_to.push(to);

                for s in buf[..nwrite].iter() {
                    decode_buf.write_buf.push(*s);
                }
            }
        }

        src_cur += nread;
        match result {
            DecoderResult::InputEmpty => {
                break;
            }
            DecoderResult::OutputFull => {
                assert!(false);
            }
            DecoderResult::Malformed(_, _) => {
                continue; // no-op
            }
        }
    }

    Ok(decode_buf)
}

// 各DecodeBufを合成し、bufで返す
// デコード失敗領域はそのまま返す
fn merge_decodebufs(src: &Vec<u8>, decodebufs: &Vec<DecodeBuf>, dst: &mut Vec<u8>) -> io::Result<usize> {

    // read範囲が一番広いエンコーディングを採用する
    // 同値の場合は先のエンコーディングを優先

    let mut cur = 0usize;

    let src_len = src.len();
    while cur < src_len {
        // TODO: 最適化

        let mut max_range = 0usize;
        let mut target_it: Option<&DecodeBuf> = None;
        let mut target_idx = 0usize;
        // find max range some decodebufs
        for it in decodebufs.into_iter() {
            let len = it.read_from.len();
            for i in 0..len {
                if it.read_from[i] == cur {
                    let range = it.read_to[i] - it.read_from[i] + 1;
                    if max_range < range {
                        max_range = range;
                        target_it = Some(it);
                        target_idx = i;
                        break;
                    }
                }
            }
        }

        // found decode bufs
        if let Some(it) = target_it {
            for s in it.write_buf[it.write_from[target_idx]..it.write_to[target_idx]].iter() {
                dst.push(*s);
            }
            cur = it.read_to[target_idx] + 1;
        } else {
            dst.push(src[cur]);
            cur += 1;
        }
    }

    Ok(dst.len())
}
