// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Emit a monophonic square wave on audio output using the
//! PulseAudio non-blocking interface.

extern crate portaudio;

use portaudio as pa;

/// Type used for generic integers. `u32` would also
/// be a reasonable choice.
type Int = u64;

/// Sample rate in frames per second.
const SAMPLE_RATE: Int = 44_100;

/// Frequency in cycles per second.
const FREQ: Int = 400;

/// Output time in milliseconds.
const MSECS: Int = 3000;


/// Size of output buffer in frames. Less than 1024 is not
/// recommended, as most audio interfaces will choke
/// horribly.
const BUFFER_SIZE: Int = 1024;

/// Total number of frames to be sent.
const FRAMES: Int = SAMPLE_RATE * MSECS / 1000;

/// Total number of buffers to be sent. The audio
/// interface requires whole buffers, so this number
/// may be one low due to truncation.
const BUFFERS: Int = FRAMES / BUFFER_SIZE;

/// Number of frames constituting a half cycle of the square
/// wave at the given frequency. The code only supports
/// whole numbers, so the frequency may be slightly higher
/// than desired due to truncation.
const FRAMES_PER_HALFCYCLE: Int = SAMPLE_RATE / (2 * FREQ);

fn main() -> Result<(), pa::Error> {
    println!("non-blocking square wave");
    println!("sample_rate: {}, msecs: {}, freq: {}",
            SAMPLE_RATE, MSECS, FREQ);
    println!("buffer size: {}, buffers: {}, halfcycle: {}",
            BUFFER_SIZE, BUFFERS, FRAMES_PER_HALFCYCLE);
    println!("last buffer nominal size: {}",
             BUFFER_SIZE * (BUFFERS + 1) - FRAMES);

    // Persistent callback state.
    let mut cycle = 0;
    let mut sign = 0.8;

    // Build a callback that fills a given buffer. It is not
    // clear to me what happens on underflow here, so I have
    // just assumed it doesn't happen. This may cause
    // underflow glitches.
    let callback = move |pa::OutputStreamCallbackArgs {buffer, frames, ..}| {
        assert_eq!(frames, BUFFER_SIZE as usize);
        for i in 0..frames {
            buffer[i] = sign;
            cycle += 1;
            if cycle >= FRAMES_PER_HALFCYCLE {
                sign = -sign;
                cycle = 0;
            }
        }
        pa::Continue
    };
    
    // Set up and start the stream.
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings(
        1,   // 1 channel
        SAMPLE_RATE as f64,
        BUFFER_SIZE as u32,
    )?;
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;
    stream.start()?;

    // Wait for the stream to play. Work could be done on
    // this thread during this time.
    pa.sleep(MSECS as i32);

    // Stop and close down the stream.
    stream.stop()?;
    stream.close()?;
    Ok(())
}
