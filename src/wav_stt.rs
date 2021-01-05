use riff_wave::WaveReader;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use vosk::{Model, Recognizer, SpeakerModel, SpeakerRecognizer};

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

pub fn process(
    input_path: PathBuf,
    recognition_model_path: &PathBuf,
    speaker_model: Option<&PathBuf>,
    output_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(&input_path)?;
    let mut text = File::create(output_path)?;

    let reader = BufReader::new(input_file);
    let mut wave_reader = WaveReader::new(reader)?;
    let fmt = &wave_reader.pcm_format;

    let mut buf = [0; 1024];
    let recognition_model = match Model::new(recognition_model_path) {
        Ok(model) => model,
        Err(_) => panic!("no valid recognition model at {}", recognition_model_path.to_str().unwrap())
    };
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
            writeln!(text, "{}", result.text)?;
            break;
        } else {
            let completed = speech_recognizer.accept_waveform(&buf[..n]);
            if completed {
                let result = speech_recognizer.final_result();
                writeln!(text, "{}", result.text)?;
            } else {
                let result = speech_recognizer.partial_result();
                if result.partial != last_part {
                    last_part.clear();
                    last_part.insert_str(0, &result.partial);
                }
            }
        }
    }
    Ok(())
}
