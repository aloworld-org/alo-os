//! Clock stalls, rollback and container tick rounding cannot repeat video time.

use super::*;

#[test]
fn a_stopped_or_rewound_clock_drops_video_without_advancing_its_time() {
    let mut order = Order::default();
    assert!(!order.admit(None));
    assert!(order.admit(Some(gst::ClockTime::from_mseconds(100))));
    for stamp in [100_000_000, 99_000_000, 100_999_999] {
        assert!(!order.admit(Some(gst::ClockTime::from_nseconds(stamp))));
    }
    assert!(order.admit(Some(gst::ClockTime::from_mseconds(101))));
    assert_eq!(order.last, Some(gst::ClockTime::from_mseconds(101)));
    assert!(Order::default().admit(Some(gst::ClockTime::ZERO)));
}

#[test]
fn rejected_bursts_cannot_reappear_as_repeated_timestamps_in_decoded_webm()
-> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;
    // Streamable output removes seeking from this in-memory fixture. It uses
    // the production muxer's timestamp scale; native fixtures cover finalization.
    let pipeline = gst::parse::launch(
        "appsrc name=input format=time caps=video/x-raw,format=I420,width=16,height=16,framerate=30/1 \
         ! vp8enc name=encoder deadline=1 lag-in-frames=0 \
         ! webmmux offset-to-zero=true timecodescale=1000000 streamable=true \
         ! matroskademux ! vp8dec ! appsink name=frames sync=false async=false",
    )?.downcast::<gst::Pipeline>().map_err(|_| "No test pipeline")?;
    prepare(&pipeline.by_name("encoder").ok_or("No encoder")?)?;
    let input = pipeline
        .by_name("input")
        .ok_or("No input")?
        .downcast::<gst_app::AppSrc>()
        .map_err(|_| "Not an app source")?;
    let sink = pipeline
        .by_name("frames")
        .ok_or("No output")?
        .downcast::<gst_app::AppSink>()
        .map_err(|_| "Not an app sink")?;
    pipeline.set_state(gst::State::Playing)?;
    let result = (|| -> Result<Vec<gst::ClockTime>, Box<dyn std::error::Error>> {
        for stamp in [
            0, 0, 100_000, 999_999, 1_000_000, 33_000_000, 20_000_000, 33_000_000, 66_000_000,
        ] {
            let mut buffer = gst::Buffer::with_size(16 * 16 * 3 / 2)?;
            buffer.make_mut().map_writable()?.as_mut_slice().fill(16);
            buffer
                .make_mut()
                .set_pts(gst::ClockTime::from_nseconds(stamp));
            buffer
                .make_mut()
                .set_duration(gst::ClockTime::from_mseconds(33));
            input.push_buffer(buffer)?;
        }
        input.end_of_stream()?;
        let mut times = Vec::new();
        while let Some(sample) = sink.try_pull_sample(gst::ClockTime::from_seconds(5)) {
            times.push(
                sample
                    .buffer()
                    .and_then(|buffer| buffer.pts())
                    .ok_or("Missing video timestamp")?,
            );
            if times.len() > 9 {
                return Err("Unexpected extra video frames".into());
            }
        }
        if !sink.is_eos() {
            return Err("The muxed recording did not finish decoding".into());
        }
        Ok(times)
    })();
    pipeline.set_state(gst::State::Null)?;
    let times = result?;
    assert_eq!(times, [0, 1, 33, 66].map(gst::ClockTime::from_mseconds));
    Ok(())
}
