//! Real element states verify retained connections and every release path.

use super::*;

fn running() -> Result<(gst::Pipeline, gst::Element), Box<dyn std::error::Error>> {
    gst::init()?;
    let pipeline = gst::parse::launch(
        "appsrc name=source is-live=true format=time ! fakesink sync=false async=false",
    )?
    .downcast::<gst::Pipeline>()
    .map_err(|_| "No connection test pipeline")?;
    pipeline.set_state(gst::State::Playing)?;
    let source = pipeline.by_name("source").ok_or("No source")?;
    Ok((pipeline, source))
}

#[test]
fn a_retained_source_keeps_playing_while_its_parent_pauses()
-> Result<(), Box<dyn std::error::Error>> {
    let (pipeline, source) = running()?;
    assert!(
        Connected::native(&pipeline).is_none(),
        "non-native sources stay parent-controlled"
    );
    let kept = Connected::keep(source.clone()).ok_or("No retained source")?;
    assert!(
        Connected::keep(source.clone()).is_none(),
        "an existing lock is never acquired twice"
    );
    pipeline.set_state(gst::State::Paused)?;
    assert_eq!(source.current_state(), gst::State::Playing);
    pipeline.set_state(gst::State::Playing)?;
    kept.release()?;
    assert!(!source.is_locked_state());
    assert_eq!(source.current_state(), gst::State::Playing);
    assert_eq!(source.base_time(), pipeline.base_time());
    pipeline.set_state(gst::State::Null)?;
    Ok(())
}

#[test]
fn an_early_return_restores_parent_control_and_teardown_never_restarts_inputs()
-> Result<(), Box<dyn std::error::Error>> {
    let (pipeline, source) = running()?;
    let kept = Connected::keep(source.clone()).ok_or("No retained source")?;
    pipeline.set_state(gst::State::Paused)?;
    drop(kept);
    assert!(!source.is_locked_state());
    assert_eq!(source.current_state(), gst::State::Paused);
    pipeline.set_state(gst::State::Playing)?;
    let kept = Connected::keep(source.clone()).ok_or("No retained source")?;
    source.set_state(gst::State::Null)?;
    kept.retire();
    assert!(!source.is_locked_state());
    assert_eq!(source.current_state(), gst::State::Null);
    pipeline.set_state(gst::State::Null)?;
    Ok(())
}

#[test]
fn a_lost_parent_refuses_release_but_never_leaves_the_source_locked()
-> Result<(), Box<dyn std::error::Error>> {
    let (pipeline, source) = running()?;
    let kept = Connected::keep(source.clone()).ok_or("No retained source")?;
    pipeline.remove(&source)?;
    assert!(kept.release().is_err());
    assert!(!source.is_locked_state());
    source.set_state(gst::State::Null)?;
    pipeline.set_state(gst::State::Null)?;
    Ok(())
}
