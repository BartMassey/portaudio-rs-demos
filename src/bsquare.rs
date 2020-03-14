// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Emit a monophonic square wave on audio output using the
//! PulseAudio blocking interface.


use portaudio as pa;

/// Type used for generic integers. `u32` would also
/// be a reasonable choice.
type Int = u64;

/// Sample rate in frames per second.
const SAMPLE_RATE: Int = 24_000;

/// Frequency in cycles per second.
const FREQ: Int = 400;

/// Output time in milliseconds.
const MSECS: Int = 3000;


/// Size of output buffer in frames. Less than 1024 is not
/// recommended, as most audio interfaces will choke
/// horribly.
const BUFFER_SIZE: usize = 16;

/// Total number of frames to be sent.
const FRAMES: usize = (SAMPLE_RATE * MSECS / 1000) as usize;

/// Total number of buffers to be sent. The audio
/// interface requires whole buffers, so this number
/// may be one low due to truncation.
const BUFFERS: usize = FRAMES / BUFFER_SIZE;

/// Number of frames constituting a half cycle of the square
/// wave at the given frequency. The code only supports
/// whole numbers, so the frequency may be slightly higher
/// than desired due to truncation.
const FRAMES_PER_HALFCYCLE: usize = (SAMPLE_RATE / (2 * FREQ)) as usize;

fn main() -> Result<(), pa::Error> {
    println!("blocking square wave");
    println!("sample_rate: {}, msecs: {}, freq: {}",
            SAMPLE_RATE, MSECS, FREQ);
    println!("buffer size: {}, buffers: {}, halfcycle: {}",
            BUFFER_SIZE, BUFFERS, FRAMES_PER_HALFCYCLE);
    println!("last buffer nominal size: {}",
             BUFFER_SIZE * (BUFFERS + 1) - FRAMES);

    // Set up the stream.
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings(
        1,   // 1 channel
        SAMPLE_RATE as f64,
        BUFFER_SIZE as u32,
    )?;
    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;
    
    // State for the square generator.
    let mut cycle = 0;
    let mut sign = 0.8;

    // Bump the state forward by the given number of frames.
    let advance_state = |cycle: &mut usize, sign: &mut f32, advance| {
        *cycle += advance;
        while *cycle >= FRAMES_PER_HALFCYCLE {
            *sign = -*sign;
            *cycle -= FRAMES_PER_HALFCYCLE;
        }
    };

    // Write all the frames.
    let mut written = 0;
    while written < FRAMES {
        let status = stream.write(BUFFER_SIZE as u32, |buffer| {
            assert_eq!(buffer.len(), BUFFER_SIZE as usize);
            for i in 0..buffer.len() {
                buffer[i] = sign;
                advance_state(&mut cycle, &mut sign, 1);
            }
        });

        // On underflow, do not panic, but skip ahead to the
        // next buffer.
        match status {
            Ok(_) => (),
            Err(pa::Error::OutputUnderflowed) => {
                eprintln!("underflow: written = {}", written);
                advance_state(&mut cycle, &mut sign, BUFFER_SIZE);
            },
            _ => {status?;},
        }

        // Advance to next buffer.
        written += BUFFER_SIZE;
    }

    // Tear down the stream.
    stream.stop()?;
    stream.close()?;
    Ok(())
}
