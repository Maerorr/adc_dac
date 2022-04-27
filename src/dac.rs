use std::fs::File;
use std::io::{BufWriter, Cursor, Read};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, Sample, SampleFormat, StreamConfig};
use hound::{WavReader, WavSamples, WavSpec, WavWriter};

pub fn dac(path: &str) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let sample_format = supported_config.sample_format();
    let config = supported_config.into();

    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut reader = WavReader::new(Cursor::new(buffer)).unwrap();
    let mut temp: Vec<f32> = Vec::new();
    for sample in reader.samples::<f32>() {
        temp.push(sample.unwrap());
    }

    let stream = match sample_format {
        SampleFormat::F32 => {
            device.build_output_stream(&config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        *sample = Sample::from(&temp[0]);
                        temp.remove(0);
                    }
                },
                move |err| {
                    println!("ERROR: {:?}", err);
                })
        }
        SampleFormat::I16 => {
            device.build_output_stream(&config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        *sample = Sample::from(&temp[0]);
                        temp.remove(0);
                    }
                },
                move |err| {
                    println!("ERROR: {:?}", err);
                })
        }
        SampleFormat::U16 => {
            device.build_output_stream(&config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        *sample = Sample::from(&temp[0]);
                        temp.remove(0);
                    }
                },
                move |err| {
                    println!("ERROR: {:?}", err);
                })
        }
    }
    .unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs((reader.duration()/config.sample_rate.0) as u64));
}
