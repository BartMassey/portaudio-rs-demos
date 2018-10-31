# `portaudio-rs` Demos
Copyright (c) 2018 Bart Massey

This collection of demos is an alternative to those provided
with the source code to the
[`portaudio-rs`](http://crates.io/crates/portaudio)
[PortAudio](http://portaudio.com) bindings crate.

As someone new to PortAudio and `portaudio-rs`, I found the
existing demos to be overly complex and mysterious. This
code builds up from a series of minimal examples. It is also
a bit more Rustic.

## Demos

* `nbsquare.rs`: Emit a monophonic square wave on audio
  output using the PulseAudio non-blocking interface.

* `bsquare.rs`: Emit a monophonic square wave on audio
  output using the PulseAudio blocking interface.

* `bsine.rs`: Emit a monophonic sine wave on audio
  output using the PulseAudio blocking interface.

## License

This program is licensed under the "MIT License". Please see
the file `LICENSE` in this distribution for license terms.
