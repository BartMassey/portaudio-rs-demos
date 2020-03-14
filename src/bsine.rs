// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Emit a monophonic sine wave on audio output using the
//! PulseAudio blocking interface.

use portaudio as pa;
use std::f32::consts::PI;

/// Sample rate in frames per second.
const SAMPLE_RATE: f32 = 44_100.0;

/// Frequency in cycles per second.
const FREQ: f32 = 400.0;

/// Output time in milliseconds.
const MSECS: usize = 3000;

/// Size of output buffer in frames. Less than 1024 is not
/// recommended, as most audio interfaces will choke
/// horribly.
const BUFFER_SIZE: usize = 1024;

/// Total number of frames to be sent.
const FRAMES: usize = (SAMPLE_RATE * MSECS as f32 / 1000.0) as usize;

/// Total number of buffers to be sent. The audio
/// interface requires whole buffers, so this number
/// may be one low due to truncation.
const BUFFERS: usize = FRAMES / BUFFER_SIZE;

fn main() -> Result<(), pa::Error> {
    println!("blocking sine wave");
    println!(
        "sample_rate: {}, msecs: {}, freq: {}",
        SAMPLE_RATE, MSECS, FREQ
    );
    println!("buffer size: {}, buffers: {}", BUFFER_SIZE, BUFFERS);
    println!(
        "last buffer nominal size: {}",
        BUFFER_SIZE * (BUFFERS + 1) - FRAMES
    );

    // Set up the stream.
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings(
        1, // 1 channel
        SAMPLE_RATE as f64,
        BUFFER_SIZE as u32,
    )?;
    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;

    // State for the sine generator.
    let mut angle: f32 = 0.0;

    // Bump the state forward by the given number of frames.
    let advance_state = |angle: &mut f32, advance| {
        *angle += advance as f32 * 2.0 * PI * FREQ / SAMPLE_RATE;
        while *angle >= 2.0 * PI {
            *angle -= 2.0 * PI;
        }
        assert!(*angle >= 0.0 && *angle < 2.0 * PI);
    };

    // Write all the frames.
    let mut written = 0;
    while written < FRAMES {
        let status = stream.write(BUFFER_SIZE as u32, |buffer| {
            assert_eq!(buffer.len(), BUFFER_SIZE);
            for sample in buffer.iter_mut() {
                *sample = 0.8 * angle.sin();
                advance_state(&mut angle, 1);
            }
        });

        // On underflow, do not panic, but skip ahead to the
        // next buffer.
        match status {
            Ok(_) => (),
            Err(pa::Error::OutputUnderflowed) => {
                eprintln!("underflow: written = {}", written);
                advance_state(&mut angle, BUFFER_SIZE);
            }
            _ => {
                status?;
            }
        }

        // Advance to next buffer.
        written += BUFFER_SIZE;
    }

    // Tear down the stream.
    stream.stop()?;
    stream.close()?;
    Ok(())
}
