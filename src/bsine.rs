extern crate portaudio;

use portaudio as pa;
use std::f32::consts::PI;

const SAMPLE_RATE: f32 = 44_100.0;
const FREQ: f32 = 400.0;
const MSECS: usize = 3000;

const FRAME_SIZE: usize = 1024;
const SAMPLES: usize = SAMPLE_RATE as usize * MSECS / 1000;
const FRAMES: usize =  SAMPLES / FRAME_SIZE;

fn main() -> Result<(), pa::Error> {
    println!("blocking sine wave");
    println!("sample_rate: {}, msecs: {}, freq: {}",
            SAMPLE_RATE, MSECS, FREQ);
    println!("frame_size: {}, frames: {}",
            FRAME_SIZE, FRAMES);
    println!("last frame nominal size: {}",
             FRAME_SIZE * (FRAMES + 1) - SAMPLES);

    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings(
        1,
        SAMPLE_RATE as f64,
        FRAME_SIZE as u32,
    )?;

    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;
    
    let mut written = 0;
    let mut angle: f32 = 0.0;
    while written < SAMPLES {
        let status = stream.write(FRAME_SIZE as u32, |buffer| {
            assert_eq!(buffer.len(), FRAME_SIZE);
            for i in 0..buffer.len() {
                buffer[i] = 0.8 * angle.sin();
                angle +=
                    2.0 * PI * FREQ / SAMPLE_RATE;
                if angle >= 2.0 * PI {
                    angle -= 2.0 * PI;
                }
                assert!(angle >= 0.0 && angle < 2.0 * PI);
            }
        });
        match status {
            Ok(_) =>
                written += FRAME_SIZE,
            Err(pa::Error::OutputUnderflowed) =>
                eprintln!("underflow: written = {}", written),
            _ => {status?;},
        }
    }

    stream.stop()?;
    stream.close()?;
    Ok(())
}
