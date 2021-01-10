mod mp3_stt;
mod opus_stt;
mod wav_stt;

use scan_dir::ScanDir;
use std::error::Error;
use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "speech essence",
    about = "Offline speech recognition tool. Uses VOSK library from https://github.com/alphacep"
)]
struct InputArgs {
    #[structopt(short = "f", long = "file")]
    /// Path to input audio file or folder with files. WAV, MP3 and OPUS is supported at the moment
    inputs: PathBuf,
    #[structopt(short = "m", long = "model")]
    /// Path to the model. Get one from https://alphacephei.com/vosk/models and unpack.
    recognition_model: PathBuf,
    #[structopt(short = "s", long = "speaker_model")]
    /// Path to the speaker identification model. Not implemented at the moment. Available at https://alphacephei.com/vosk/models
    speaker_model: Option<PathBuf>,
    #[structopt(short = "o", long = "output")]
    /// Output directory.
    output_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: InputArgs = InputArgs::from_args();
    let mut files = Vec::new();
    let recognition_model = args.recognition_model;
    let speaker_model = args.speaker_model;
    let output_path = args.output_path;

    if !output_path.exists() {
        std::fs::create_dir_all(output_path.clone())?;
    }

    if args.inputs.is_dir() {
        let walk_res = ScanDir::files().walk(args.inputs, |iter| {
            for (entry, _name) in iter {
                files.push((
                    entry.path(),
                    String::from(entry.path().file_stem().unwrap().to_str().unwrap()),
                ));
            }
        });
        if let Err(errors) = walk_res {
            for e in errors {
                eprintln!("Files discovering error {}", e);
            }
        }
    } else {
        let name = String::from(args.inputs.file_stem().unwrap().to_str().unwrap());
        files.push((args.inputs, name));
    }

    files.sort();
    println!("Processing files:\n{:#?}", files);

    for f in files {
        let ext = f.0.extension().and_then(OsStr::to_str);
        let mut filename = output_path.clone();
        filename.push(f.1);
        match ext {
            Some("wav") => {
                println!("processing file {}", f.0.to_str().unwrap());
                wav_stt::process(f.0, &recognition_model, speaker_model.as_ref(), filename)?;
            }
            Some("mp3") => {
                println!("processing file {}", f.0.to_str().unwrap());
                mp3_stt::process(f.0, &recognition_model, speaker_model.as_ref(), filename)?;
            }
            Some("opus") => {
                println!("processing file {}", f.0.to_str().unwrap());
                opus_stt::process(f.0, &recognition_model, speaker_model.as_ref(), filename)?;
            }
            Some(_) => eprintln!("Unsupported file {}", f.0.to_str().unwrap()),
            None => eprintln!("File has no extension {}", f.0.to_str().unwrap()),
        }
    }
    Ok(())
}
