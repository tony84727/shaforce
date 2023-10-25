use std::fmt::LowerHex;
use std::fs::File;
use std::io::BufWriter;
use std::num::Wrapping;
use std::time::{Duration, Instant};
use std::{io::Write, ops::RangeInclusive};

use clap::{Parser, Subcommand};
use itertools::Itertools;
use rayon::prelude::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    include_wgsl, AdapterInfo, BindGroupEntry, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor,
    Features, InstanceDescriptor,
};

const CHARS: RangeInclusive<char> = '!'..='~';

struct Sha1([u8; 20]);

fn preprocessing(mut input: Vec<u8>) -> Vec<u8> {
    let ml = (input.len() * 8) as u64;
    input.push(0x80);
    let padding = {
        let r = input.len() % 64;
        if r > 56 {
            64 + 56 - r
        } else {
            56 - r
        }
    };
    for _ in 0..padding {
        input.push(0x0);
    }
    let mut length = ml.to_be_bytes().to_vec();
    input.append(&mut length);
    input
}

fn sha1(input: &str) -> Sha1 {
    let mut hash: [Wrapping<u32>; 5] = [
        Wrapping(0x67452301),
        Wrapping(0xEFCDAB89),
        Wrapping(0x98BADCFE),
        Wrapping(0x10325476),
        Wrapping(0xC3D2E1F0),
    ];
    for chunk in preprocessing(input.as_bytes().to_vec())
        .chunks(64)
        .into_iter()
    {
        let bytes = {
            let mut messages: Vec<u32> = chunk
                .chunks(4)
                .map(|bytes| u32::from_be_bytes(bytes.try_into().unwrap()))
                .collect();
            messages.resize(80, 0);
            for i in 16..80 {
                messages[i] =
                    (messages[i - 3] ^ messages[i - 8] ^ messages[i - 14] ^ messages[i - 16])
                        .rotate_left(1);
            }
            messages
        };
        let mut chunk_hash = hash.clone();

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => (
                    (chunk_hash[1] & chunk_hash[2]) | (!chunk_hash[1] & chunk_hash[3]),
                    0x5A827999_u32,
                ),
                20..=39 => (
                    chunk_hash[1] ^ chunk_hash[2] ^ chunk_hash[3],
                    0x6ED9EBA1_u32,
                ),
                40..=59 => (
                    (chunk_hash[1] & chunk_hash[2])
                        | (chunk_hash[1] & chunk_hash[3])
                        | (chunk_hash[2] & chunk_hash[3]),
                    0x8F1BBCDC_u32,
                ),
                _ => (
                    chunk_hash[1] ^ chunk_hash[2] ^ chunk_hash[3],
                    0xCA62C1D6_u32,
                ),
            };
            let temp: Wrapping<u32> = Wrapping(chunk_hash[0].0.rotate_left(5))
                + f
                + chunk_hash[4]
                + Wrapping(k)
                + Wrapping(bytes[i]);
            chunk_hash.rotate_right(1);
            chunk_hash[2] = Wrapping(chunk_hash[2].0.rotate_left(30));
            chunk_hash[0] = temp;
        }
        for (i, chunk_parts) in chunk_hash.into_iter().enumerate() {
            hash[i] += chunk_parts;
        }
    }
    let mut sha1 = [0; 20];
    for (i, u) in hash.into_iter().enumerate() {
        for (j, byte) in u.0.to_be_bytes().iter().enumerate() {
            sha1[i * 4 + j] = *byte;
        }
    }
    Sha1(sha1)
}

impl LowerHex for Sha1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in self.0 {
            write!(f, "{i:02x}")?;
        }
        Ok(())
    }
}

#[derive(Parser)]
struct CommandOption {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compute sha1 hash by CPU
    Cpu(CpuOption),
    /// Compute sha1 hash by GPU
    Gpu,
}

#[derive(Parser)]
struct CpuOption {
    /// Optional destination to dump results to a file
    output_file: Option<String>,
}

struct Sink<I>
where
    I: Iterator<Item = String>,
{
    counter: usize,
    last: Option<Instant>,
    output: Option<BufWriter<File>>,
    source: I,
}

impl<I> Sink<I>
where
    I: Iterator<Item = String>,
{
    fn new(source: I) -> Self {
        Self {
            counter: 0,
            last: None,
            output: None,
            source,
        }
    }
    fn with_output(&mut self, output: File) -> &mut Self {
        self.output = Some(BufWriter::new(output));
        self
    }
    fn sink(self) {
        let Self {
            mut counter,
            mut last,
            mut output,
            source,
        } = self;
        let second = Duration::from_secs(1);
        for result in source {
            let now = Instant::now();
            counter += 1;
            if let Some(output) = output.as_mut() {
                output.write_all(format!("{result}\n").as_bytes()).unwrap();
            }
            match last {
                Some(l) => {
                    if now - l >= second {
                        eprintln!("{counter}/s");
                        counter = 0;
                        last = Some(now);
                    }
                }
                None => {
                    last = Some(now);
                }
            }
        }
    }
}

fn print_gpu_info(info: &AdapterInfo) {
    println!("using gpu {}", info.name);
}

#[tokio::main]
async fn main() {
    let option = CommandOption::parse();
    match option.command {
        Command::Cpu(cpu_option) => {
            let (sender, receiver) = crossbeam::channel::unbounded();
            std::thread::spawn(move || {
                (0..8)
                    .into_par_iter()
                    .flat_map(|length| {
                        CHARS
                            .permutations(length)
                            .par_bridge()
                            .map(|chars| chars.into_iter().collect::<String>())
                    })
                    .map(|input: String| {
                        let hash = sha1(&input);
                        format!("{input}\t{hash:x}")
                    })
                    .for_each(|result| {
                        sender.send(result).unwrap();
                    });
            });
            let mut sink = Sink::new(receiver.into_iter());
            if let Some(out) = cpu_option.output_file {
                sink.with_output(File::create(out).unwrap());
            }
            sink.sink();
        }
        Command::Gpu => {
            let instance = wgpu::Instance::new(InstanceDescriptor::default());
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptionsBase::default())
                .await
                .expect("wgpu adapter");
            let info = adapter.get_info();
            let project_label = Some("shaforce");
            print_gpu_info(&info);
            let (device, queue) = adapter
                .request_device(
                    &DeviceDescriptor {
                        label: project_label,
                        features: Features::default(),
                        limits: wgpu::Limits::downlevel_defaults(),
                    },
                    None,
                )
                .await
                .unwrap();
            let compute_module = device.create_shader_module(include_wgsl!("sha1.wgsl"));
            let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: project_label,
                layout: None,
                module: &compute_module,
                entry_point: "main",
            });
            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: project_label,
            });
            let input = preprocessing("!".as_bytes().to_vec());
            let input_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: project_label,
                contents: &input,
                usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            });
            let result_buffer = device.create_buffer(&BufferDescriptor {
                label: project_label,
                size: 20,
                usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            let staging_buffer = device.create_buffer(&BufferDescriptor {
                label: project_label,
                size: 20,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: project_label,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: input_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: result_buffer.as_entire_binding(),
                    },
                ],
            });
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: project_label,
                });
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(1, 1, 1);
            }
            encoder.copy_buffer_to_buffer(&result_buffer, 0, &staging_buffer, 0, 20);
            queue.submit(Some(encoder.finish()));
            let (sender, receiver) = crossbeam::channel::unbounded();
            let output_slice = staging_buffer.slice(..);
            output_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
            device.poll(wgpu::MaintainBase::Wait);
            let _r = receiver.recv().unwrap().unwrap();
            let view = output_slice.get_mapped_range();
            let mut result: [u8; 20] = Default::default();
            for (i, byte) in view.into_iter().enumerate() {
                result[i] = *byte;
            }
            let result = Sha1(result);
            println!("{result:x}");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{preprocessing, sha1};
    use proptest::proptest;

    #[test]
    fn test_sha1_empty() {
        let hash = sha1("");
        let hex = format!("{hash:x}");
        assert_eq!(40, hex.len());
        assert_eq!("da39a3ee5e6b4b0d3255bfef95601890afd80709", hex);
    }

    #[test]
    fn test_preprocessing() {
        assert_eq!(0, preprocessing(Vec::new()).len() % 64);
    }

    proptest! {
        #[test]
        fn match_sha1_crate(s: String) {
            let expected = {
                use sha1::{Digest, Sha1};
                let mut hash = Sha1::new();
                hash.update(s.as_bytes());
                format!("{:x}", hash.finalize())
            };
            assert_eq!(expected, format!("{:x}", sha1(&s)));
        }
    }
}
