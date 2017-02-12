use std::io;
use std::fmt;
use std::str;
use std::error;
use std::fs::File;
use std::io::prelude::*;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub struct Wav {
    pub num_channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub data: WavData,
}

#[derive(Debug)]
pub enum WavData {
    U8(Vec<u8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
}

#[derive(Debug)]
pub enum WavLoadError {
    Io(io::Error),
    Parse(String),
}

impl error::Error for WavLoadError {
    fn description(&self) -> &str {
        match *self {
            WavLoadError::Io(ref err) => err.description(),
            WavLoadError::Parse(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            WavLoadError::Io(ref err) => Some(err),
            WavLoadError::Parse(_) => None,
        }
    }
}

impl fmt::Display for WavLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WavLoadError::Io(ref err) => write!(f, "IO error: {}", err),
            WavLoadError::Parse(ref err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl From<io::Error> for WavLoadError {
    fn from(error: io::Error) -> WavLoadError {
        WavLoadError::Io(error)
    }
}

impl From<String> for WavLoadError {
    fn from(error_msg: String) -> WavLoadError {
        WavLoadError::Parse(error_msg)
    }
}

impl Wav {
    pub fn from_file(path: &str) -> Result<Wav, WavLoadError> {
        let mut f = try!(File::open(path));
        let mut buf = Vec::new();
        try!(f.read_to_end(&mut buf));

        Wav::buf_to_wav(&mut buf)
    }

    fn buf_to_wav(buf: &[u8]) -> Result<Wav, WavLoadError> {
        macro_rules! wav_assert_eq {
            ($a:expr, $b:expr, $msg:expr) => {
                if $a != $b {
                    return Err(WavLoadError::Parse($msg));
                }
            }
        }

        if buf.len() < 44 {
            return Err(WavLoadError::Parse("Incomplete header".to_string()));
        }

        let mut reader = io::Cursor::new(&buf);
        let chunk_id = str::from_utf8(&buf[0..4]).unwrap();
        wav_assert_eq!(chunk_id, "RIFF", format!("Unsupported wave file (got: \"{}\", expected \"RIFF\")", chunk_id));
        
        //Advance the reader until after the chunk_id
        reader.set_position(4);
        let chunk_size = reader.read_u32::<LittleEndian>().unwrap();
        wav_assert_eq!(chunk_size as usize, buf.len() - 8, format!("actual file size does not match reported size (got: \"{}\", expected \"{}\")", chunk_size, buf.len() - 8));

        let format = str::from_utf8(&buf[8..12]).unwrap();
        wav_assert_eq!(format, "WAVE", format!("Unsupported format (got: \"{}\", expected \"WAVE\")", format));

        let sub_chunk_1_id = str::from_utf8(&buf[12..16]).unwrap();
        wav_assert_eq!(sub_chunk_1_id, "fmt ", format!("sub chunk 1 has unknown format (got: \"{}\", expected \"WAVE\")", sub_chunk_1_id));

        reader.set_position(16);
        let sub_chunk_1_size = reader.read_u32::<LittleEndian>().unwrap();
        wav_assert_eq!(sub_chunk_1_size, 16, format!("sub chunk 1 has invalid size (got: {}, expected 16)", sub_chunk_1_size)); 

        let audio_format = reader.read_u16::<LittleEndian>().unwrap();
        wav_assert_eq!(audio_format, 1, format!("Invalid audio format (got: {}, expected 1 (=PCM))", audio_format)); 

        let num_channels = reader.read_u16::<LittleEndian>().unwrap();
        let sample_rate = reader.read_u32::<LittleEndian>().unwrap();
        let byte_rate = reader.read_u32::<LittleEndian>().unwrap();
        let block_align = reader.read_u16::<LittleEndian>().unwrap();
        let bits_per_sample = reader.read_u16::<LittleEndian>().unwrap();

        let sub_chunk_2_id = str::from_utf8(&buf[36..40]).unwrap();
        wav_assert_eq!(sub_chunk_2_id, "data", format!("sub chunk 2 has unknown format (got: \"{}\", expected \"data\")", sub_chunk_2_id));

        reader.set_position(40);
        let sub_chunk_2_size = reader.read_u32::<LittleEndian>().unwrap();
        wav_assert_eq!(buf.len() - 44, sub_chunk_2_size as usize, format!("actual file size does not match reported size (got: \"{}\", expected \"{}\")", sub_chunk_2_size, buf.len() - 44));

        let data_enum = match bits_per_sample {
            8 => {
                let mut data = Vec::new();
                while let Ok(val) = reader.read_u8() {
                    data.push(val);
                }
                WavData::U8(data)
            },
            16 => {
                let mut data = Vec::new();
                while let Ok(val) = reader.read_i16::<LittleEndian>() {
                    data.push(val);
                }
                WavData::I16(data)
            },
            32 => {
                let mut data = Vec::new();
                while let Ok(val) = reader.read_i32::<LittleEndian>() {
                    data.push(val);
                }
                WavData::I32(data)
            },
            _ => return Err(WavLoadError::Parse(format!("Unexpected bits per sample value (got: \"{}\", expected 8, 16, or 32)", bits_per_sample))),
        };

        Ok(Wav { 
            num_channels: num_channels, 
            sample_rate: sample_rate, 
            byte_rate:  byte_rate, 
            block_align: block_align, 
            bits_per_sample: bits_per_sample,
            data: data_enum,
        })
    }
}
