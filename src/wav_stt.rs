use hound::WavReader;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use vosk::{Model, Recognizer, SpeakerModel};

fn read_sample(r: &mut WavReader<BufReader<File>>, channel: usize) -> Vec<i16> {
    let step = r.spec().channels as usize;
    r.samples::<i16>()
        .skip(channel)
        .step_by(step)
        .map(|x| x.unwrap())
        .collect()
}

pub fn process(
    input_path: PathBuf,
    recognition_model_path: &PathBuf,
    speaker_model_path: Option<&PathBuf>,
    output_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    vosk::set_log_level(1);
    let fmt = WavReader::open(input_path.clone())?.spec();

    for channel in 0..fmt.channels {
        let mut reader = WavReader::open(input_path.clone())?;
        // Attach provided recognition model
        let recognition_model = match Model::new(recognition_model_path) {
            Ok(model) => model,
            Err(_) => panic!(
                "no valid recognition model at {}",
                recognition_model_path.to_str().unwrap()
            ),
        };
        // Create recognizer.
        let mut recognizer = match speaker_model_path {
            Some(path) => {
                let _speaker_model = match SpeakerModel::new(path) {
                    Ok(model) => model,
                    Err(_) => panic!(
                        "no valid recognition model at {}",
                        recognition_model_path.to_str().unwrap()
                    ),
                };
                Recognizer::new(&recognition_model, fmt.sample_rate as f32)
                // Not implemented yet
                //SpeakerRecognizer::new(&recognition_model, &speaker_model, fmt.sample_rate as f32)
            }
            None => Recognizer::new(&recognition_model, fmt.sample_rate as f32),
        };

        let mut name = output_path.clone();
        name = name
            .with_file_name(
                name.file_stem().unwrap().to_str().unwrap().to_owned()
                    + "_channel_"
                    + channel.to_string().as_str(),
            )
            .with_extension("txt");
        let mut text = File::create(name).unwrap();
        let buf = read_sample(&mut reader, channel as usize);
        let completed = recognizer.accept_waveform(&buf);
        if completed {
            let result = recognizer.result();
            writeln!(text, "{}", result.text)?;
        } else {
            let result = recognizer.final_result();
            writeln!(text, "{}", result.text)?;
        }
    }

    Ok(())
}
