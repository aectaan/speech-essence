use minimp3::{Decoder, Error};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use vosk::{Model, Recognizer, SpeakerModel};

pub fn process(
    input_path: PathBuf,
    recognition_model_path: &PathBuf,
    speaker_model_path: Option<&PathBuf>,
    output_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut meta = Decoder::new(File::open(input_path.clone())?);
    let header = meta.next_frame()?;
    let sample_rate = header.sample_rate as f32;
    let channels = header.channels;

    let mut data: Vec<Vec<i16>> = Vec::new();
    for _ in 0..channels {
        data.push(Vec::new())
    }
    loop {
        match meta.next_frame() {
            Ok(frame) => {
                for channel in 0..frame.channels {
                    let frame_data = frame.data.iter().skip(channel).step_by(frame.channels);
                    data[channel].extend(frame_data);
                }
            }
            Err(Error::Eof) => break,
            Err(e) => panic!("mp3 handling error {}", e),
        }
    }

    for channel in 0..channels {
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
                Recognizer::new(&recognition_model, sample_rate)
                // Not implemented yet
                //SpeakerRecognizer::new(&recognition_model, &speaker_model, sample_rate)
            }
            None => Recognizer::new(&recognition_model, sample_rate),
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
        let completed = recognizer.accept_waveform(&data[channel]);
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
