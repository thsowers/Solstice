use byteorder::{BigEndian, ReadBytesExt};
use clap::{App, Arg, SubCommand};
use hound::{SampleFormat, WavReader, WavSamples, WavSpec, WavWriter};
use num::complex::Complex;
use rustfft::FFTplanner;
use rustfft::FFT;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;

trait Signal {
    fn energy(self) -> f64;
}

impl<'a, R> Signal for WavSamples<'a, R, i16>
where
    R: std::io::Read,
{
    fn energy(self) -> f64 {
        self.map(|x| {
            let sample = x.unwrap() as f64;
            sample * sample
        })
        .sum()
    }
}

fn find_spectral_peak(filename: &str) -> Option<f32> {
    let mut reader = WavReader::open(filename).expect("Failed to open WAV file");
    let num_samples = reader.len() as usize;
    let mut planner = FFTplanner::new(false);
    let mut fft = planner.plan_fft(num_samples);
    println!("Collecting {} samples...", num_samples);
    let mut signal = reader
        .samples::<i16>()
        .map(|x| Complex::new(x.unwrap() as f32, 0f32))
        .collect::<Vec<_>>();
    let mut spectrum = signal.clone();
    println!("Performing FFT...");
    fft.process(&mut signal[..], &mut spectrum[..]);
    println!("Searching for max spectrum...");
    let max_peak = spectrum
        .iter()
        .take(num_samples / 2)
        .enumerate()
        .max_by_key(|&(_, freq)| freq.norm() as u32);
    println!("...Done");
    if let Some((i, _)) = max_peak {
        let bin = 44100f32 / num_samples as f32;
        Some(i as f32 * bin)
    } else {
        None
    }
}

fn main() {
    let matches = App::new("Solstice")
        .version("0.1")
        .author("Tyler H. Sowers <thsowers@gmail.com>")
        .about("Audio analysis tool")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("input")
                .help("Sets an optional output file")
                .takes_value(true)
                .index(1),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .multiple(true)
                .help("Turn debugging information on"),
        )
        .subcommand(
            SubCommand::with_name("test")
                .about("does testing things")
                .arg(Arg::with_name("list").short("l").help("lists test values")),
        )
        .get_matches();

    if let Some(o) = matches.value_of("input") {
        let spec = find_spectral_peak(o);
        println!("{}", spec.unwrap())
    } else {
        println!("Please provide an audio file");
    }
}
