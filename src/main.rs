extern crate alsa;
extern crate byteorder;

mod wav_loader;

use std::{u8, i16, i32};
use std::f32::consts::PI;
use std::io::Write;
use std::ffi::CString;
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access};

use wav_loader::{Wav, WavData};

fn main() {
    //Sound card settings
    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();
    let hwp = HwParams::any(&pcm).unwrap();

    //let wav = Wav::from_file("1khz.wav").unwrap();
    let wav = Wav::from_file("now_stand_aside.wav").unwrap();
    let samples = match wav.data {
        WavData::U8(ref data) => u8_to_floats(&data),
        WavData::I16(ref data) => i16_to_floats(&data),
        WavData::I32(ref data) => i32_to_floats(&data),
    };

    hwp.set_channels(wav.num_channels as u32).unwrap();
    hwp.set_rate(wav.sample_rate, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::FloatLE).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();

    let mut io = pcm.io_f32().unwrap();
    io.writei(&samples);
    
    dft(&samples[0..1024], wav.sample_rate as usize / 2, wav.sample_rate);
}

fn hanning_window(index: usize, total: usize) -> f32 {
    0.5 * (1.0 - ((2.0 * index as f32 * PI) / (total as f32 - 1.0)).cos())
}

fn dft(samples: &[f32], bin_count: usize, sample_rate: u32) {
    //wave with frequency f = sin(f * 2 * pi)
    
    let step_size = 1.0 / sample_rate as f32;
    let freq_step = sample_rate as f32 / 2.0 / bin_count as f32;

    for bin in 0 .. bin_count {
        let mut sum_sin = 0.0;
        let mut sum_cos = 0.0;
        for (i, sample) in samples.iter().enumerate() {
            let input = freq_step * bin as f32 * 2.0 * PI * step_size * i as f32;
            let window = hanning_window(bin, bin_count);
            //let sin_val = sample * window * input.sin();
            //let cos_val = sample * window * input.cos();
            let sin_val = sample * input.sin();
            let cos_val = sample * input.cos();
            sum_sin += sin_val;
            sum_cos += cos_val;
        }

        let avg_sin = sum_sin / samples.len() as f32;
        let avg_cos = sum_cos / samples.len() as f32;
        let magnitude = (avg_sin.powi(2) + avg_cos.powi(2)).sqrt();
        println!("{} {}", bin, magnitude);
    }
}

fn u8_to_floats(src: &[u8]) -> Vec<f32> {
    let half_max = (u8::MAX / 2) as isize;
    let mut samples = Vec::with_capacity(src.len());
    for sample in src {
        samples.push((*sample as isize - half_max) as f32 / half_max as f32);
    }
    samples
}

fn i16_to_floats(src: &[i16]) -> Vec<f32> {
    let mut samples = Vec::with_capacity(src.len());
    for sample in src {
        samples.push(*sample as f32 / i16::MAX as f32);
    }
    samples
}

fn i32_to_floats(src: &[i32]) -> Vec<f32> {
    let mut samples = Vec::with_capacity(src.len());
    for sample in src {
        samples.push(*sample as f32 / i32::MAX as f32);
    }
    samples
}
