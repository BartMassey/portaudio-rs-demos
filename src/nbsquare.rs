// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

extern crate portaudio;

use portaudio as pa;

type Int = u64;

const SAMPLE_RATE: Int = 44_100;
const FREQ: Int = 400;
const MSECS: Int = 3000;

const FRAME_SIZE: Int = 1024;
const SAMPLES: Int = SAMPLE_RATE * MSECS / 1000;
const FRAMES: Int = SAMPLES / FRAME_SIZE;
const SAMPLES_PER_HALFCYCLE: Int = SAMPLE_RATE / (2 * FREQ);

fn main() -> Result<(), pa::Error> {
    println!("non-blocking square wave");
    println!("sample_rate: {}, msecs: {}, freq: {}",
            SAMPLE_RATE, MSECS, FREQ);
    println!("frame_size: {}, frames: {}, halfcycle: {}",
            FRAME_SIZE, FRAMES, SAMPLES_PER_HALFCYCLE);
    println!("last frame nominal size: {}",
             FRAME_SIZE * (FRAMES + 1) - SAMPLES);

    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings(
        1,
        SAMPLE_RATE as f64,
        FRAME_SIZE as u32,
    )?;

    let mut cycle = 0;
    let mut sign = 0.8;

    let callback = move |pa::OutputStreamCallbackArgs {buffer, frames, ..}| {
        assert_eq!(frames, FRAME_SIZE as usize);
        for i in 0..frames {
            buffer[i] = sign;
            cycle += 1;
            if cycle >= SAMPLES_PER_HALFCYCLE {
                sign = -sign;
                cycle = 0;
            }
        }
        pa::Continue
    };
    
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;
    stream.start()?;
    pa.sleep(MSECS as i32);
    stream.stop()?;
    stream.close()?;
    Ok(())
}
