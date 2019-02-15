use clap::{App, Arg, SubCommand};
use hound::{WavReader, WavSamples};
use num::complex::Complex;
use rustfft::FFTplanner;
use std::fs::File;
use std::io::BufReader;
use stft::{STFT, WindowType};

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

fn output_data(spectrum: Vec<Complex<f32>>, num_samples: usize) {
    println!("Outputing some data");
    let audio_data = spectrum
        .iter()
        .take(num_samples / 2) // FFT is symmetrical, ignore first half
        .enumerate();
    println!("...Done");
    for x in audio_data {
        println!("{:?}", x)
    }
}

fn generate_spectrogram() {
    // let's generate ten seconds of fake audio
    let sample_rate: usize = 44100;
    let seconds: usize = 10;
    let sample_count = sample_rate * seconds;
    let all_samples = (0..sample_count).map(|x| x as f64).collect::<Vec<f64>>();

    // let's initialize our short-time fourier transform
    let window_type: WindowType = WindowType::Hanning;
    let window_size: usize = 1024;
    let step_size: usize = 512;
    let mut stft = STFT::new(window_type, window_size, step_size);

    // we need a buffer to hold a computed column of the spectrogram
    let mut spectrogram_column: Vec<f64> =
        std::iter::repeat(0.).take(stft.output_size()).collect();

    // iterate over all the samples in chunks of 3000 samples.
    // in a real program you would probably read from something instead.
    for some_samples in (&all_samples[..]).chunks(10000) {
        // append the samples to the internal ringbuffer of the stft
        stft.append_samples(some_samples);

        // as long as there remain window_size samples in the internal
        // ringbuffer of the stft
        while stft.contains_enough_to_compute() {
            // compute one column of the stft by
            // taking the first window_size samples of the internal ringbuffer,
            // multiplying them with the window,
            // computing the fast fourier transform,
            // taking half of the symetric complex outputs,
            // computing the norm of the complex outputs and
            // taking the log10
            stft.compute_column(&mut spectrogram_column[..]);

            // here's where you would do something with the
            // spectrogram_column...
            println!("{:?}", spectrogram_column);

            // drop step_size samples from the internal ringbuffer of the stft
            // making a step of size step_size
            stft.move_to_next_column();
        }
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
        //println!("{}", find_spectral_peak(spectrum, num_samples).unwrap());
        generate_spectrogram();
        //output_data(spectrum, num_samples);
    } else {
        println!("Please provide an audio file");
    }
}
