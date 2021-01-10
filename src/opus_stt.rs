
use std::path::PathBuf;
use std::result::Result::Ok;

pub fn process(
    input_path: PathBuf,
    recognition_model_path: &PathBuf,
    speaker_model_path: Option<&PathBuf>,
    output_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut  file = opusfile::OggOpusFile::from_slice(input_path.to_str().unwrap().as_bytes())?;
    let head = file.head(None).unwrap();
    let sample_rate = head.input_sample_rate;
    let channels = head.channel_count;

    let mut data: Vec<i16> = Vec::new();

    match channels {
        1 => while let Ok(count) = file.read(&mut *data, None){
            if count ==0{
                break;
            }
        },
        2 => {
            while let Ok(count) =file.read_stereo(&mut data){
                if count ==0{
                    break;
                }
            }
        },
        _ => panic!("wrong channel number {}", channels),
    }
    Ok(())
}