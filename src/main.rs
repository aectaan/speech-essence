use riff_wave::WaveReader;
use scan_dir::{ScanDir, Walker};
use std::ffi::OsStr;
use std::fs::{create_dir, create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use vosk::{Model, Recognizer, SpeakerModel, SpeakerRecognizer};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "speech essence",
    about = "Offline speech recognition tool. Uses VOSK library from https://github.com/alphacep"
)]
struct InputArgs {
    #[structopt(short = "f", long = "file")]
    /// Path to input audio file. Only mono WAV is supported at the moment
    inputs: PathBuf,
    #[structopt(short = "m", long = "model")]
    /// Path to the model. Get one from https://alphacephei.com/vosk/models and unpack.
    recognition_model: PathBuf,
    #[structopt(short = "s", long = "speaker_model")]
    /// Path to the speaker identification model. Available at https://alphacephei.com/vosk/models
    speaker_model: Option<PathBuf>,
    #[structopt(short = "o", long = "output")]
    /// Path to the output file. Decoded text will be routed to stdout if path is not provided.
    output_path: PathBuf,
}

fn read_sample(r: &mut WaveReader<BufReader<File>>, buf: &mut [i16]) -> usize {
    let mut i = 0;
    for _ in 0..buf.len() {
        match r.read_sample_i16() {
            Ok(s) => {
                buf[i] = s;
                i += 1;
            }
            Err(e) => {
                println!("{:?}", e);
                break;
            }
        }
    }
    i
}

fn process_wav(
    input_path: PathBuf,
    recognition_model: &PathBuf,
    speaker_model: Option<&PathBuf>,
    output_path: PathBuf,
) {
    let input_file = File::open(&input_path).unwrap();
    let mut text = File::create(output_path).unwrap();

    let reader = BufReader::new(input_file);
    let mut wave_reader = WaveReader::new(reader).unwrap();
    let fmt = &wave_reader.pcm_format;

    let mut buf = [0; 1024];
    let recognition_model = Model::new(recognition_model).unwrap();
    if speaker_model.is_some() {
        let speaker_model = SpeakerModel::new(speaker_model.unwrap()).unwrap();
        let mut speaker_recognizer =
            SpeakerRecognizer::new(&recognition_model, &speaker_model, fmt.sample_rate as f32);
    }

    let mut speech_recognizer = Recognizer::new(&recognition_model, fmt.sample_rate as f32);
    let mut last_part = String::new();

    loop {
        let n = read_sample(&mut wave_reader, &mut buf);
        if n == 0 {
            let result = speech_recognizer.final_result();
            writeln!(text, "{}", result.text).unwrap();
            // println!("Final result: {}", result.text);
            break;
        } else {
            let completed = speech_recognizer.accept_waveform(&buf[..n]);
            if completed {
                let result = speech_recognizer.final_result();
                writeln!(text, "{}", result.text).unwrap();
            // println!("Result: {}", result.text);
            } else {
                let result = speech_recognizer.partial_result();
                if result.partial != last_part {
                    last_part.clear();
                    last_part.insert_str(0, &result.partial);
                    // println!("Partial: {}", result.partial);
                }
            }
        }
    }
}

fn main() {
    let args: InputArgs = InputArgs::from_args();
    let mut files = Vec::new();
    let recognition_model = args.recognition_model;
    let speaker_model = args.speaker_model;
    let output_path = args.output_path;

    if !output_path.exists() {
        create_dir_all(output_path.clone()).unwrap();
    }

    if args.inputs.is_dir() {
        ScanDir::files()
            .walk(args.inputs, |iter| {
                for (entry, name) in iter {
                    files.push((
                        entry.path(),
                        String::from(entry.path().file_stem().unwrap().to_str().unwrap()),
                    ));
                }
            })
            .unwrap();
    } else {
        let name = String::from(args.inputs.file_stem().unwrap().to_str().unwrap());
        files.push((args.inputs, name));
    }

    files.sort();
    println!("Processing files:\n{:#?}", files);

    for f in files {
        let ext = f.0.extension().and_then(OsStr::to_str);
        let mut filename = output_path.clone();
        filename.push(f.1 + ".txt");
        match ext {
            Some("wav") => {
                println!("processing file {}", f.0.to_str().unwrap());
                process_wav(f.0, &recognition_model, speaker_model.as_ref(), filename);
            }
            Some(_) => eprintln!("Unsupported file {}", f.0.to_str().unwrap()),
            None => eprintln!("File has no extension {}", f.0.to_str().unwrap()),
        }
    }
}
