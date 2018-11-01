// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Emit a stereo white noise on audio output using the
//! PulseAudio blocking interface, via a white noise table.

extern crate rand;
extern crate portaudio;

use rand::random;
use portaudio as pa;

/// Sample rate in frames per second.
const SAMPLE_RATE: f32 = 44_100.0;

/// Output time in milliseconds.
const MSECS: usize = 3000;


/// Two channels for stereo.
const CHANNELS: usize = 2;

/// Size of output buffer in frames. Less than 1024 frames
/// is not recommended, as most audio interfaces will choke
/// horribly.
const BUFFER_SIZE: usize = 1024;

/// Size of output table in frames. This is arbitrary,
/// except that it should be at least as large as
/// BUFFER_SIZE.
const TABLE_SIZE: usize = SAMPLE_RATE as usize;

/// Total number of frames to be sent.
const FRAMES: usize = (SAMPLE_RATE * MSECS as f32 / 1000.0) as usize;

/// Total number of buffers to be sent. The audio
/// interface requires whole buffers, so this number
/// may be one low due to truncation.
const BUFFERS: usize =  FRAMES / BUFFER_SIZE;

fn main() -> Result<(), pa::Error> {
    println!("blocking stereo white noise from table");
    println!("sample_rate: {}, msecs: {}",
            SAMPLE_RATE, MSECS);
    println!("buffer size: {}, buffers: {}",
            BUFFER_SIZE, BUFFERS);
    println!("last buffer nominal size: {}",
             BUFFER_SIZE * (BUFFERS + 1) - FRAMES);

    // Build the white noise table.
    let table: Vec<[f32; CHANNELS]> = (0..TABLE_SIZE)
        .map(|_| {
            let mut frame = [0.0; CHANNELS];
            for sample in frame.iter_mut() {
                *sample = 0.8 * random::<f32>();
            }
            frame
        })
        .collect();

    // Set up the stream.
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings(
        CHANNELS as i32,
        SAMPLE_RATE as f64,
        BUFFER_SIZE as u32,
    )?;
    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;
    
    // State for the table reader.
    let mut entry = 0;

    // Bump the state forward by the given number of frames.
    let advance_state = |entry: &mut usize, advance| {
        *entry += advance;
        while *entry >= TABLE_SIZE {
            *entry -= TABLE_SIZE;
        }
    };

    // Write all the frames.
    let mut written = 0;
    while written < FRAMES {
        let status = stream.write(BUFFER_SIZE as u32, |buffer| {
            assert_eq!(buffer.len(), BUFFER_SIZE * CHANNELS);
            for frame in buffer.chunks_mut(CHANNELS) {
                for (channel, sample) in frame.iter_mut().enumerate() {
                    *sample = table[entry][channel];
                }
                advance_state(&mut entry, 1);
            }
        });

        // On underflow, do not panic, but skip ahead to the
        // next buffer.
        match status {
            Ok(_) => (),
            Err(pa::Error::OutputUnderflowed) => {
                eprintln!("underflow: written = {}", written);
                advance_state(&mut entry, BUFFER_SIZE);
            },
            _ => {
                status?;
            },
        }

        // Advance to next buffer.
        written += BUFFER_SIZE;
    }

    // Tear down the stream.
    stream.stop()?;
    stream.close()?;
    Ok(())
}
