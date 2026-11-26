//! A WAV file's length, read from its header: what a recording costs is priced by its minutes,
//! and without FFmpeg (or when FFmpeg cannot read the file) the header still says them. Only
//! the chunks before the samples are read.

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// The length of the WAV file at `path`, in milliseconds; `None` when it is not a plain RIFF
/// WAV file whose header gives its byte rate and the size of its samples.
pub fn duration_ms(path: &Path) -> Option<u64> {
    let file = File::open(path).ok()?;
    let size = file.metadata().ok()?.len();
    read_duration(&mut BufReader::new(file), size)
}

fn read_duration(reader: &mut (impl Read + Seek), size: u64) -> Option<u64> {
    let mut riff = [0u8; 12];
    reader.read_exact(&mut riff).ok()?;
    if &riff[0..4] != b"RIFF" || &riff[8..12] != b"WAVE" {
        return None;
    }
    let mut byte_rate: Option<u64> = None;
    let mut at: u64 = 12;
    loop {
        let mut header = [0u8; 8];
        reader.read_exact(&mut header).ok()?;
        at += 8;
        let id = &header[0..4];
        let len = u64::from(u32::from_le_bytes(header[4..8].try_into().ok()?));
        match id {
            b"fmt " => {
                let mut fmt = [0u8; 16];
                if len < 16 {
                    return None;
                }
                reader.read_exact(&mut fmt).ok()?;
                let rate = u64::from(u32::from_le_bytes(fmt[8..12].try_into().ok()?));
                byte_rate = (rate > 0).then_some(rate);
                // Chunks are padded to an even length.
                let rest = len - 16 + (len & 1);
                reader
                    .seek(SeekFrom::Current(i64::try_from(rest).ok()?))
                    .ok()?;
                at += len + (len & 1);
            }
            b"data" => {
                // A file still being written, or one longer than its header can say, gives
                // its samples' size as all ones, or larger than the file: what is there counts.
                let there = size.saturating_sub(at);
                let samples = if len == u64::from(u32::MAX) {
                    there
                } else {
                    len.min(there)
                };
                return byte_rate.map(|rate| samples * 1_000 / rate);
            }
            _ => {
                let skip = len + (len & 1);
                reader
                    .seek(SeekFrom::Current(i64::try_from(skip).ok()?))
                    .ok()?;
                at += skip;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::Cursor;

    use super::*;

    /// A WAV header of `fmt_len` bytes of format (16 or 18) at `rate` samples a second, 16-bit
    /// mono, an odd-sized chunk before the samples, then `seconds` of silence.
    fn wav(fmt_len: u32, rate: u32, seconds: u32, data_len: Option<u32>) -> Vec<u8> {
        let samples = rate * 2 * seconds;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF\0\0\0\0WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&fmt_len.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * 2).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.resize(out.len() + (fmt_len as usize - 16), 0);
        out.extend_from_slice(b"LIST");
        out.extend_from_slice(&3u32.to_le_bytes());
        out.extend_from_slice(b"abc\0");
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.unwrap_or(samples).to_le_bytes());
        out.resize(out.len() + samples as usize, 0);
        out
    }

    fn length(bytes: &[u8]) -> Option<u64> {
        read_duration(&mut Cursor::new(bytes), bytes.len() as u64)
    }

    #[test]
    fn a_wav_files_length_is_read_from_its_header() {
        assert_eq!(length(&wav(16, 16_000, 3, None)), Some(3_000));
        // A format chunk of 18 bytes, as the weekly live check's sample has.
        assert_eq!(length(&wav(18, 8_000, 2, None)), Some(2_000));
    }

    #[test]
    fn a_header_that_overstates_its_samples_counts_what_the_file_holds() {
        assert_eq!(length(&wav(16, 8_000, 2, Some(u32::MAX))), Some(2_000));
        assert_eq!(length(&wav(16, 8_000, 2, Some(1_000_000))), Some(2_000));
    }

    #[test]
    fn anything_else_has_no_length() {
        assert_eq!(length(b""), None);
        assert_eq!(length(b"ID3\x04\0\0\0\0\0\0\0\0\0\0"), None);
        let mut no_format = wav(16, 8_000, 1, None);
        no_format[12..16].copy_from_slice(b"junk");
        assert_eq!(length(&no_format), None);
    }

    #[test]
    fn the_repositorys_sample_recording_is_read() {
        let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples/interview.wav");
        let ms = duration_ms(&sample).unwrap();
        assert!((21_000..22_000).contains(&ms), "{ms}");
    }
}
