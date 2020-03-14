// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Read monaural audio output using the PulseAudio blocking
//! interface and write it to stdout as signed 16-bit
//! big-endian samples. On Linux, you can save the output in
//! a file named "test.s16" and play it with the
//! [SoX](http://sox.sourceforge.net) `play` command:
//! `play -B -r 44100 test.s16`.

use byteorder::{BigEndian, WriteBytesExt};
use portaudio as pa;

use std::io::stdout;

/// Sample rate in frames per second.
const SAMPLE_RATE: f32 = 44_100.0;

/// Input time in milliseconds.
const MSECS: usize = 3000;

/// Size of input buffer in frames. Less than 1024 frames
/// is not recommended, as most audio interfaces will choke
/// horribly.
const BUFFER_SIZE: usize = 1024;

/// Total number of frames to be received.
const FRAMES: usize = (SAMPLE_RATE * MSECS as f32 / 1000.0) as usize;

/// Total number of buffers to be received. The audio
/// interface requires whole buffers, so this number
/// may be one low due to truncation.
const BUFFERS: usize = FRAMES / BUFFER_SIZE;

fn main() -> Result<(), pa::Error> {
    eprintln!("read audio input and write to stdout");
    eprintln!("sample_rate: {}, msecs: {}", SAMPLE_RATE, MSECS);
    eprintln!("buffer size: {}, buffers: {}", BUFFER_SIZE, BUFFERS);
    eprintln!(
        "last buffer nominal size: {}",
        BUFFER_SIZE * (BUFFERS + 1) - FRAMES
    );

    // Set up the stream.
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_input_stream_settings(
        1, // 1 channel
        SAMPLE_RATE as f64,
        BUFFER_SIZE as u32,
    )?;
    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;

    // Get a handle to the output.
    let mut stdout = stdout();

    // Read all the frames.
    let mut read = 0;
    while read < FRAMES {
        // On overflow, do not panic, but fill with zeros
        // and skip ahead to the next buffer. This may not
        // be quite right, but is as good as we can do.
        match stream.read(BUFFER_SIZE as u32) {
            Ok(buffer) => {
                assert_eq!(buffer.len(), BUFFER_SIZE);
                for &b in buffer {
                    stdout
                        .write_i16::<BigEndian>(b)
                        .expect("bad write of audio buffer");
                }
            }
            Err(pa::Error::InputOverflowed) => {
                eprintln!("overflow: read = {}", read);
                for &b in [0; BUFFER_SIZE].iter() {
                    stdout
                        .write_i16::<BigEndian>(b)
                        .expect("bad write of zeros");
                }
            }
            status => {
                status?;
            }
        }

        // Advance to next buffer.
        read += BUFFER_SIZE;
    }

    // Tear down the stream.
    stream.stop()?;
    stream.close()?;
    Ok(())
}
