extern crate audio;

use audio::{WaveGenerator, WaveType};

fn main() {
    let mut wave_gen = WaveGenerator::new();

    wave_gen.generate(WaveType::Sine);

    wave_gen.save("sine.wav");
}
