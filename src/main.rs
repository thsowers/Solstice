use clap::{App, Arg, SubCommand};
use hound::{WavReader, WavSamples};
use num::complex::Complex;
use rustfft::FFTplanner;
use std::fs::File;
use std::io::BufReader;

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

// Much acknowledgment to http://siciarz.net/24-days-rust-hound/
fn get_audio_data(filename: &str) -> (Vec<Complex<f32>>, usize) {
    // Get audio data and size
    let (mut reader, num_samples) = read_audio_file(filename);

    // Setup FFT
    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(num_samples);

    // Collect audio data into a complex vector for FFT
    let mut signal = reader
        .samples::<i16>()
        .map(|x| Complex::new(x.unwrap() as f32, 0f32))
        .collect::<Vec<_>>();

    // Set output to be same length as input
    let mut spectrum = signal.clone();

    // Perform FFT
    println!("Performing FFT...");
    fft.process(&mut signal[..], &mut spectrum[..]);

    (spectrum, num_samples)
}

fn read_audio_file(filename: &str) -> (WavReader<BufReader<File>>, usize) {
    // Read WAV audio file
    let reader = WavReader::open(filename).expect("Failed to open WAV file");
    let num_samples = reader.len() as usize; // Get length of transform to be performed

    (reader, num_samples)
}

fn find_spectral_peak(spectrum: Vec<Complex<f32>>, num_samples: usize) -> Option<f32> {
    println!("Searching for max spectrum...");
    let max_peak = spectrum
        .iter()
        .take(num_samples / 2) // FFT is symmetrical, ignore first half
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
        let (spectrum, num_samples) = get_audio_data(o);
        println!("{}", find_spectral_peak(spectrum, num_samples).unwrap())
    } else {
        println!("Please provide an audio file");
    }
}
