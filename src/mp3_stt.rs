use minimp3::{Decoder, Error};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use vosk::{Model, Recognizer};

pub fn process(
    input_path: PathBuf,
    recognition_model_path: &PathBuf,
    _speaker_model_path: Option<&PathBuf>,
    output_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut meta = Decoder::new(File::open(input_path.clone())?);
    let header = meta.next_frame()?;
    let sample_rate = header.sample_rate as f32;
    let channels = header.channels;

    let mut recognizers = Vec::new();
    let mut output_files = Vec::new();

    for channel in 0..channels {
        let recognition_model = match Model::new(recognition_model_path) {
            Ok(model) => model,
            Err(_) => panic!(
                "no valid recognition model at {}",
                recognition_model_path.to_str().unwrap()
            ),
        };
        let recognizer = Recognizer::new(&recognition_model, sample_rate);
        recognizers.push(recognizer);

        let mut name = output_path.clone();
        name = name
            .with_file_name(
                name.file_stem().unwrap().to_str().unwrap().to_owned()
                    + "_channel_"
                    + channel.to_string().as_str(),
            )
            .with_extension("txt");
        output_files.push(File::create(name).unwrap());
    }
    loop {
        match meta.next_frame() {
            Ok(frame) => {
                for channel in 0..frame.channels {
                    let frame_data = frame
                        .data
                        .iter()
                        .skip(channel)
                        .step_by(frame.channels);
                        let mut channel_data :Vec<i16> = Vec::new();
                        channel_data.extend(frame_data);
                    let completed = recognizers[channel].accept_waveform(&channel_data);
                    if completed {
                        let result = recognizers[channel].result();
                        writeln!(output_files[channel], "{:?}", result)?;
                        println!("{:?}", result);
                    } else {
                        let result = recognizers[channel].partial_result();
                        writeln!(output_files[channel], "{:?}", result.partial)?;
                        println!("{:?}", result.partial);
                    }
                }
            }
            Err(Error::Eof) => break,
            Err(e) => panic!("mp3 handling error {}", e),
        }
    }

    // for channel in 0..channels {
    //     let completed = recognizer.accept_waveform(&data[channel]);
    //     if completed {
    //         let result = recognizer.result();
    //         writeln!(text, "{}", result.text)?;
    //     } else {
    //         let result = recognizer.final_result();
    //         writeln!(text, "{}", result.text)?;
    //     }
    // }
    Ok(())
}
