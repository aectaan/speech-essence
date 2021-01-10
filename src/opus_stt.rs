use std::fs::File;
use std::path::PathBuf;
use std::result::Result::Ok;
use std::io::{Write, Read};
use vosk::{Model, Recognizer, SpeakerModel};

pub fn process(
    input_path: PathBuf,
    recognition_model_path: &PathBuf,
    speaker_model_path: Option<&PathBuf>,
    output_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut opus = File::open(input_path)?;
    let mut file_raw:Vec<u8> = Vec::new();
    opus.read_to_end(&mut file_raw)?;
    let mut cursor = std::io::Cursor::new(file_raw.as_slice());
    let mut file = opusfile::OggOpusFile::from_read(&mut cursor).unwrap();
    let head = file.head(None).unwrap();
    let sample_rate = head.input_sample_rate;
    let channels = head.channel_count as usize;


    let mut data = Vec::new();
    for _ in 0..channels {
        data.push(Vec::new());
    }

    match channels {
        1 => {
            let mut buf : [f32; 11520] = [0.0; 11520];
            while let Ok(count) = file.read_float(&mut buf[..], None) {
                if count == 0 {
                    break;
                }
                data[0].extend_from_slice(&mut buf);
            }
        }
        2 => {
            let mut buf : [f32; 11520] = [0.0;11520];
            while let Ok(count) = file.read_float_stereo(&mut buf[..]) {
                if count == 0 {
                    break;
                }
                for channel in 0..channels {
                    let frame_data = buf.iter().skip(channel as usize).step_by(channels as usize);
                    data[channel].extend(frame_data);
                }
            }
        }
        _ => panic!("wrong channel number {}", channels),
    };

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
                Recognizer::new(&recognition_model, sample_rate as f32)
                // Not implemented yet
                //SpeakerRecognizer::new(&recognition_model, &speaker_model, sample_rate)
            }
            None => Recognizer::new(&recognition_model, sample_rate as f32),
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
        let completed = recognizer.accept_waveform_f32(&data[channel]);
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
