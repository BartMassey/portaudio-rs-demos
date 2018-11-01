// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Read audio input from a WAV file and play it "fuzzed" using
//! PortAudio's blocking interfaces.

extern crate portaudio;
extern crate hound;

use portaudio as pa;
use hound::WavReader;

use std::io::stdin;

/// Size of input buffer in frames. Less than 1024 frames
/// is not recommended, as most audio interfaces will choke
/// horribly.
const BUFFER_SIZE: usize = 1024;

/// Clip point. This is super-aggressive.
const CLAMP_VALUE: i32 = std::i32::MAX / 16;

fn main() -> Result<(), pa::Error> {
    eprintln!("read WAV file from stdin and play it fuzzed");

    // Set up the WAV reader.
    let stdin = stdin();
    let wav = WavReader::new(stdin).expect("WAV reader open failed");
    let spec = wav.spec();
    eprintln!("sample rate: {}, channels: {}, sample bits: {}, format: {:?}",
              spec.sample_rate, spec.channels,
              spec.bits_per_sample, spec.sample_format);
    let mut samples = wav.into_samples::<i32>();

    // Set up the stream.
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings(
        1,   // 1 channel
        spec.sample_rate as f64,
        BUFFER_SIZE as u32,
    )?;
    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;
    
    // Read and write all the frames.
    let mut done = false;
    while !done {
        let status = stream.write(BUFFER_SIZE as u32, |buffer| {
            assert_eq!(buffer.len(), BUFFER_SIZE);
            for b in buffer.iter_mut() {
                let s = if done { 0 } else {
                    match samples.next() {
                        Some(s) => s.expect("bad sample during WAV read"),
                        None => { done = true; 0 },
                    }
                };
                let s = if s > CLAMP_VALUE {
                    CLAMP_VALUE
                } else if s < -CLAMP_VALUE {
                    -CLAMP_VALUE
                } else {
                    s
                };
                *b = s;
            }
        });

        // On underflow, do not panic, but skip ahead to the
        // next buffer.
        match status {
            Ok(_) => (),
            Err(pa::Error::OutputUnderflowed) => {
                eprintln!("underflow");
                for _ in 0..BUFFER_SIZE {
                    let _ = samples.next()
                        .expect("bad sample during underflow");
                }
            },
            _ => {
                status?;
            },
        }
    }

    // Tear down the stream.
    stream.stop()?;
    stream.close()?;
    Ok(())
}
