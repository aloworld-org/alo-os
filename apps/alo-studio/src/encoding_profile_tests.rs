//! The lightweight profile saves work by batching, while retaining every sample.

use gst::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[test]
fn lightweight_opus_uses_fewer_packets_and_decodes_every_input_sample()
-> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;
    let pipeline = gst::parse::launch(&format!(
        "audiotestsrc num-buffers=10 samplesperbuffer=1600 ! {} ! identity name=packets ! opusdec ! audio/x-raw,format=S16LE,rate=16000,channels=1 ! fakesink name=decoded sync=false async=false",
        super::audio(5),
    ))?
    .downcast::<gst::Pipeline>()
    .map_err(|_| "No test pipeline")?;
    let packets = Arc::new(AtomicUsize::new(0));
    let counted = packets.clone();
    pipeline
        .by_name("packets")
        .and_then(|element| element.static_pad("sink"))
        .ok_or("No packet observation point")?
        .add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            if info
                .buffer()
                .is_some_and(|buffer| !buffer.flags().contains(gst::BufferFlags::HEADER))
            {
                counted.fetch_add(1, Ordering::Relaxed);
            }
            gst::PadProbeReturn::Ok
        })
        .ok_or("Could not observe encoded packets")?;
    let samples = Arc::new(AtomicUsize::new(0));
    let decoded = samples.clone();
    pipeline
        .by_name("decoded")
        .and_then(|element| element.static_pad("sink"))
        .ok_or("No decoded observation point")?
        .add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            if let Some(buffer) = info.buffer() {
                decoded.fetch_add(buffer.size() / 2, Ordering::Relaxed);
            }
            gst::PadProbeReturn::Ok
        })
        .ok_or("Could not observe decoded audio")?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        pipeline.set_state(gst::State::Playing)?;
        let message = pipeline
            .bus()
            .ok_or("No pipeline bus")?
            .timed_pop_filtered(
                gst::ClockTime::from_seconds(10),
                &[gst::MessageType::Eos, gst::MessageType::Error],
            )
            .ok_or("The encoded audio did not drain")?;
        assert_eq!(message.type_(), gst::MessageType::Eos);
        assert_eq!(samples.load(Ordering::Relaxed), 16_000);
        assert!((16..=18).contains(&packets.load(Ordering::Relaxed)));
        Ok(())
    })();
    pipeline.set_state(gst::State::Null)?;
    result
}

#[test]
fn lightweight_microphone_reads_larger_chunks_with_the_same_bounded_ring()
-> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;
    for (rate, read_micros) in [(5, 100_000_i64), (30, 20_000_i64)] {
        let source = gst::parse::launch(super::microphone(rate))?;
        assert_eq!(source.property::<i64>("latency-time"), read_micros);
        assert_eq!(source.property::<i64>("buffer-time"), 2_000_000);
        assert!(source.property::<bool>("do-timestamp"));
    }
    Ok(())
}
