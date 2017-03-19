extern crate audio;

use audio::{WaveGenerator, WaveType};

fn main() {
    let mut wave_gen = WaveGenerator::new();

    wave_gen.generate(WaveType::White);

    wave_gen.save("white.wav");
}
