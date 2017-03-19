extern crate byteorder;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;

use byteorder::{LittleEndian, WriteBytesExt};

struct HeaderChunk {
    chunk_id: &'static str,
    file_length: u32,
    riff_type: &'static str
}

impl HeaderChunk {
    pub fn new() -> HeaderChunk {
        HeaderChunk {
            chunk_id: "RIFF",
            file_length: 0,         // total file length minus 8, which is taken by "RIFF"
            riff_type: "WAVE"
        }
    }
}

struct FormatChunk {
    chunk_id: &'static str,
    chunk_size: usize,        // Length of header in bytes
    tag: u16,               // 1 (MS PCM)
    channels: u16,          // # of channels
    samples_per_sec: u32,   // frequency of audio in HZ ... 44.1 khz
    avg_bytes_per_sec: u32, // For estimating RAM allocation
    block_align: u16,
    bits_per_sample: u16
}

impl FormatChunk {
    pub fn new() -> FormatChunk {
        let channels = 2u16;
        let bits_per_sample = 16u16;
        let samples_per_sec = 44100u32;

        let block_align = channels * (bits_per_sample / 8);
        let avg_bytes_per_sec = samples_per_sec * block_align as u32;

        FormatChunk {
            chunk_id: "fmt ",
            chunk_size: 16,
            tag: 1,
            channels: channels,
            samples_per_sec: samples_per_sec,
            bits_per_sample: bits_per_sample,
            block_align: block_align,
            avg_bytes_per_sec: avg_bytes_per_sec
        }
    }
}

/**
 * 8-bit audio use [u8]
 * 16-bit audio use [u16]
 * 32-bit audio use [f32]
 */
struct DataChunk {
    chunk_id: &'static str,
    chunk_size: usize,
    samples: Vec<i16>
}

impl DataChunk {
    pub fn new() -> DataChunk {
        DataChunk {
            chunk_id: "data",
            chunk_size: 0,
            samples: Vec::new()
        }
    }
}

enum WaveType {
    SineWave
}

struct WaveGenerator {
    header: HeaderChunk,
    format: FormatChunk,
    data: DataChunk
}

impl WaveGenerator {
    pub fn new(wave_type: WaveType) -> WaveGenerator {
        let header = HeaderChunk::new();
        let format = FormatChunk::new();
        let mut data = DataChunk::new();

        match wave_type {
            WaveType::SineWave => {
                // Calculate size of data array
                // size = sample_rate * channel
                let num_samples: u32 = format.samples_per_sec * format.channels as u32;
                data.samples.reserve(num_samples as usize);

                // For 16-bit audio samples range from -32760 to 32760
                let amplitude = 32760i32; // Max amplitude for 16-bit audio

                // 440hz is concert A
                let frequency = 440.0f32; // Tutorial uses a double but float should be fine

                // Time value
                let t = (std::f32::consts::PI * 2.0 * frequency) / (num_samples as f32);

                // For number of samples
                for i in 0..num_samples - 1 {
                    // Per channel becaues Wav sample data is interleaved per channel
                    for channel in 0..format.channels {
                        // Theres gotta be a better way to handle type matching
                        // let index = i as usize + channel as usize;

                        // Calculate the sample value
                        // data.sample_data[index] = (amplitude as f32 * (t * i as f32).sin()) as u16;
                        let sample = (amplitude as f32 * (t * i as f32).sin()) as i16;
                        data.samples.push(sample);
                    }
                }

                // Calculate chunk size
                data.chunk_size = data.samples.len() * (format.bits_per_sample as usize / 8);
            }
        }

        WaveGenerator {
            header: header,
            format: format,
            data: data
        }
    }

    pub fn save(&self, filepath: &str) -> io::Result<()> {
        let file = File::create(filepath)?;
        let mut bw = BufWriter::new(file);

        // Write header
        {
            let header = &self.header;

            // Write the header chunk
            bw.write(b"RIFF")?;
            // bw.write(header.chunk_id.as_bytes())?;
            // bw.write(header.chunk_id.as_ref())?;
            // bw.write(&[header.file_length])?;
            // write!(bw, "{}", header.file_length)?;
            bw.write_u32::<LittleEndian>(header.file_length)?;
            bw.write(b"WAVE")?;
            // bw.write(header.riff_type.as_bytes())?;
        }

        // Write format chunk
        {
            let format = &self.format;

            bw.write(b"fmt ")?;
            // bw.write(format.chunk_id.as_bytes())?;
            bw.write_u32::<LittleEndian>(format.chunk_size as u32)?;
            bw.write_u16::<LittleEndian>(format.tag)?;
            bw.write_u16::<LittleEndian>(format.channels)?;
            bw.write_u32::<LittleEndian>(format.samples_per_sec)?;
            bw.write_u32::<LittleEndian>(format.avg_bytes_per_sec)?;
            bw.write_u16::<LittleEndian>(format.block_align)?;
            bw.write_u16::<LittleEndian>(format.bits_per_sample)?;
        }

        // Write data chunk
        {
            let data = &self.data;

            // bw.write(data.chunk_id.as_bytes())?;
            bw.write(b"data")?;
            // bw.write(&[data.chunk_size])?;
            bw.write_u32::<LittleEndian>(data.chunk_size as u32)?;

            for sample in &data.samples {
                bw.write_i16::<LittleEndian>(sample.clone())?;
                // bw.write(&[sample])?;
                // write!(bw, "{}", sample)?;
            }
        }

        // Rewrite filesize
        {
            use std::io::SeekFrom;

            bw.seek(SeekFrom::Start(4))?;
            let metadata = bw.get_ref().metadata()?;
            let filesize = metadata.len() - 8;

            println!("File size is {}", filesize);
            bw.write_u32::<LittleEndian>(filesize as u32)?;
            // bw.write((filesize).to_string().as_bytes())?;
            // let filesize: usize = bw.get_ref().metadata()?.len();
        }
        // Set writer to 4th byte posiion
        // let filesize: usize = writer.length
        // writer.write(filesize - 8)

        Ok(())
    }
}

fn main() {
    let wave_gen = WaveGenerator::new(WaveType::SineWave);

    wave_gen.save("sine.wav");

    ()
}
