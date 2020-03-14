// Copyright (c) 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Read audio input from a WAV file and play it "fuzzed" using
//! PortAudio's blocking interfaces.

extern crate portaudio;
extern crate hound;

use portaudio as pa;

use std::io::{stdout, Write};

/// Clip point. This is super-aggressive.
const CLAMP_VALUE: f32 = 0.125;

fn main() -> Result<(), pa::Error> {
    eprintln!("read audio and write it fuzzed");

    // Set up the stream.
    let pa = pa::PortAudio::new()?;

    let dev_input = pa.default_input_device()?;
    let info_input = pa.device_info(dev_input)?;
    let input_latency =
        info_input.default_low_input_latency;
    let sample_rate = info_input.default_sample_rate;

    let dev_output = pa.default_output_device()?;
    let output_latency =
        pa.device_info(dev_output)?.default_low_output_latency;

    //let period = (input_latency.max(output_latency) * sample_rate) as u32;
    let period = 4096;

    println!("sample rate: {}, period: {}",
             sample_rate, period);
    println!("input_latency: {}, output_latency: {}",
              input_latency * 1000.0, output_latency * 1000.0);

    let params = pa::StreamParameters::new(
        dev_input,   // Device.
        1,   // Channels.
        true,   // Interleaved if true.
        input_latency,   // Latency in seconds.
    );
    let settings = pa::InputStreamSettings::new(
        params,
        sample_rate,
        period,
    );
    let mut input_stream = pa.open_blocking_stream(settings)?;

    let params = pa::StreamParameters::new(
        dev_output,   // Device.
        1,   // Channels.
        true,   // Interleaved if true.
        output_latency,   // Latency in seconds.
    );
    let settings = pa::OutputStreamSettings::new(
        params,
        sample_rate,
        period,
    );
    let mut output_stream = pa.open_non_blocking_stream(settings)?;
    
    let mut input_buffer: Option<&[f32]>;

    // Read and write all the frames.
    input_stream.start()?;
    match input_stream.read(period) {
        Ok(buf) => {
            input_buffer = Some(buf);
        },
        Err(e) => {
            eprintln!("could not start input: {}", e);
            std::process::exit(1);
        },
    }

    let mut output_underruns = 0;
    let mut input_overruns = 0;
    let mut stdout = stdout();

    output_stream.start()?;
    loop {
        if let Some(ref inp) = input_buffer {
            let callback = |output_buffer: &mut [f32]| {
                // This cute idea taken from the `rust-portaudio`
                // non-blocking example.
                for (output_sample, input_sample)
                    in output_buffer.iter_mut().zip(inp.iter())
                {
                        let s: f32 = *input_sample;
                        *output_sample = if s > CLAMP_VALUE {
                            CLAMP_VALUE
                        } else if s < -CLAMP_VALUE {
                            -CLAMP_VALUE
                        } else {
                            s
                        };
                    }
                };

            if let Err(e) = output_stream.write(period, callback) {
                if output_underruns == 0 {
                    println!("write error: {}", e);
                } else {
                    print!("\runderruns: {}", output_underruns);
                    let _ = stdout.flush();
                }
                output_underruns += 1;
            }
        }

        match input_stream.read(period) {
            Ok(buf) => {
                assert_eq!(buf.len() as u32, period);
                input_buffer = Some(buf);
            },
            Err(e) => {
                if input_overruns == 0 {
                    println!("read error: {}", e);
                } else {
                    print!("\roverruns: {}", input_overruns);
                    let _= stdout.flush();
                }
                input_overruns += 1;
                input_buffer = None;
            },
        }
    }
}
