extern crate portaudio;

use portaudio as pa;

const SAMPLE_RATE: i32 = 44_100;
const FREQ: usize = 400;
const MSECS: i32 = 3000;

const FRAME_SIZE: u32 = 1024;
const SAMPLES: usize = SAMPLE_RATE as usize * MSECS as usize / 1000;
const FRAMES: usize =  SAMPLES / FRAME_SIZE as usize;
const SAMPLES_PER_HALFCYCLE: i32 = (SAMPLE_RATE / (2 * FREQ as i32)) as i32;

fn main() -> Result<(), pa::Error> {
    println!("blocking square wave");
    println!("sample_rate: {}, msecs: {}, freq: {}",
            SAMPLE_RATE, MSECS, FREQ);
    println!("frame_size: {}, frames: {}, halfcycle: {}",
            FRAME_SIZE, FRAMES, SAMPLES_PER_HALFCYCLE);
    println!("last frame nominal size: {}",
             FRAME_SIZE as i32 * (FRAMES as i32 + 1) - SAMPLES as i32);

    let pa = pa::PortAudio::new()?;
    let settings =
        pa.default_output_stream_settings(1, SAMPLE_RATE as f64, FRAME_SIZE)?;

    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;
    
    let mut written = 0;
    let mut cycle = 0;
    let mut sign = 0.8;
    while written < SAMPLES as u32 {
        let status = stream.write(FRAME_SIZE, |buffer| {
            assert_eq!(buffer.len(), FRAME_SIZE as usize);
            for i in 0..buffer.len() {
                buffer[i] = sign;
                cycle += 1;
                if cycle >= SAMPLES_PER_HALFCYCLE {
                    sign = -sign;
                    cycle = 0;
                }
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
