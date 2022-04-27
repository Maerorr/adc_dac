use std::fs::File;
use std::io::{BufWriter, Read};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, Sample, SampleFormat, StreamConfig};
use hound::{WavSpec, WavWriter};

pub fn write_data_to_writer_f<U>(
    data: &[f32],
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) where
    U: cpal::Sample + hound::Sample,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in data.iter() {
                let sample: U = cpal::Sample::from(&sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

pub fn write_data_to_writer_i<U>(
    data: &[i16],
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) where
    U: cpal::Sample + hound::Sample,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in data.iter() {
                let sample: U = cpal::Sample::from(&sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

pub fn adc(path: &str) {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("no output device");
    let mut supported_configs_range = device
        .supported_input_configs()
        .expect("error while querying configs");

    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();

    let spec: WavSpec = WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate.0,
        bits_per_sample: match sample_format {
            SampleFormat::F32 => 32,
            SampleFormat::I16 => 16,
            _ => panic!("unsupported sample format"),
        },
        sample_format: match sample_format {
            SampleFormat::F32 => hound::SampleFormat::Float,
            SampleFormat::I16 => hound::SampleFormat::Int,
            _ => panic!("unsupported sample format"),
        },
    };

    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    let writer = Arc::new(Mutex::new(Some(writer)));

    let writer2 = writer.clone();

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| write_data_to_writer_f::<f32>(data, &writer2),
            move |err| {
                println!("ERROR: {:?}", err);
            },
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| {
                write_data_to_writer_i::<i16>(data, &writer2);
            },
            move |err| {
                println!("ERROR: {:?}", err);
            },
        ),
        SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _| {
                panic!("unsupported sample format");
            },
            move |err| {
                println!("ERROR: {:?}", err);
            },
        ),
    }
    .unwrap();

    stream.play().unwrap();
    println!("recording... Press enter to stop.");
    loop {
        let input: Option<i32> = std::io::stdin()
            .bytes()
            .next()
            .and_then(|result| result.ok())
            .map(|byte| byte as i32);
        if input.unwrap() == 13 {
            break;
        }
    }

    drop(stream);
    println!("recording stopped.");
    writer.lock().unwrap().take().unwrap().finalize().unwrap();
}
