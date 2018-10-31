extern crate portaudio;

use portaudio as pa;
use std::f64::consts::PI;

const SAMPLE_RATE: u32 = 44_100;
const FREQ: u32 = 1000;
const MSECS: u32 = 3000;
const FRAME_SIZE: u32 = 128;
const NTABLE: usize = (SAMPLE_RATE * FREQ) as usize;

fn main() -> Result<(), pa::Error> {
    let pa = pa::PortAudio::new()?;
    let settings =
        pa.default_output_stream_settings(1, SAMPLE_RATE as f64, FRAME_SIZE)?;


    let scale =  2.0 * PI as f32 * FREQ as f32 / SAMPLE_RATE as f32;
    println!("ntable: {}", NTABLE);
    let mut table = vec![0.0; NTABLE];
    for i in 0..NTABLE {
        table[i] = (i as f32 * scale).sin();
    }

    let nsamples = MSECS * SAMPLE_RATE;
    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;
    
    let mut sample = 0;
    let mut written = 0;
    while written < nsamples {
        let to_write = u32::min(nsamples - written, 1024);
        println!("sample: {}, written: {}, to_write: {}", sample, written, to_write);
        stream.write(to_write, |buffer| {
            println!("writing");
            for j in 0..(to_write as usize) {
                buffer[j] = table[sample];
                sample += 1;
                if sample >= NTABLE {
                    sample = 0;
                }
            }
        })?;
        written += to_write;
    }

    stream.stop()?;
    stream.close()?;
    Ok(())
}
