use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use rustyllama::constants::LATENCY_MS;
// the output stream starts after the ring buffer is filled
// for now it's just an implementation of a feedback input/output loop but with a bigger ring buffer for helping debugging the input
fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();

    let input_device = host
        .default_input_device()
        .expect("No default input device");
    println!("Using input device: {:?}", input_device.name());
    let output_device = host
        .default_output_device()
        .expect("No default output device");

    let config: cpal::StreamConfig = input_device
        .default_input_config()
        .expect("failed to load config")
        .into();

    let seconds = 15.0;
    let samples_per_second = config.sample_rate.0 as usize * config.channels as usize;
    let total_samples = (seconds * samples_per_second as f32) as usize;

    let ring = HeapRb::<f32>::new(total_samples);
    let (mut producer, mut consumer) = ring.split();

    // populating the ring buffer with 0.0 samples (silence)
    for _ in 0..total_samples {
        producer.try_push(0.0).unwrap();
    }

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if producer.try_push(sample).is_err() {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut input_fell_behind = false;
        for sample in data {
            *sample = match consumer.try_pop() {
                Some(s) => s,
                None => {
                    input_fell_behind = true;
                    0.0
                }
            };
        }
        if input_fell_behind {
            eprintln!("input stream fell behind: try increasing latency");
        }
    };
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn, None)?;
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        LATENCY_MS
    );
    print_ring_buffer_info(&config, total_samples);
    loop {
        input_stream.play()?;
        output_stream.play()?;
    }

    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn print_ring_buffer_info(config: &cpal::StreamConfig, latency_samples: usize) {
    let ring_size = latency_samples * 2;
    let seconds_of_audio =
        ring_size as f32 / (config.sample_rate.0 as f32 * config.channels as f32);
    println!("Ring buffer size: {} samples", ring_size);
    println!(
        "Ring buffer can hold: {:.2} seconds of audio",
        seconds_of_audio
    );
}
