//! One-off real media-server check; never installed in the image.

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use pipewire::{self as pw, types::ObjectType};
    use std::{
        cell::RefCell,
        collections::BTreeSet,
        rc::Rc,
        time::{Duration, Instant},
    };
    let mut args = std::env::args().skip(1);
    let selected = alo_capture_protocol::MediaNode {
        id: args.next().ok_or("selected ID required")?.parse()?,
        serial: args.next().ok_or("selected serial required")?.parse()?,
    };
    let fd = alo_capture_remote::restricted_remote(selected, "org.aloworld.Studio")?;
    let main_loop = pw::main_loop::MainLoopBox::new(None)?;
    let context = pw::context::ContextBox::new(main_loop.loop_(), None)?;
    let core = context.connect_fd(fd, None)?;
    let registry = core.get_registry()?;
    let nodes = Rc::new(RefCell::new(BTreeSet::new()));
    let observed = nodes.clone();
    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            if global.type_ == ObjectType::Node {
                observed.borrow_mut().insert(global.id);
            }
        })
        .register();
    let until = Instant::now() + Duration::from_secs(5);
    println!("restricted-remote-connected");
    while Instant::now() < until {
        if main_loop
            .loop_()
            .iterate(pw::loop_::Timeout::Finite(Duration::from_millis(25)))
            < 0
        {
            return Err("restricted connection stopped".into());
        }
        if nodes.borrow().iter().any(|id| *id != selected.id) {
            return Err(format!("unrelated nodes were visible: {:?}", nodes.borrow()).into());
        }
    }
    if !nodes.borrow().contains(&selected.id) {
        return Err("the chosen node was not visible".into());
    }
    println!("restricted-remote-only-selected: {}", selected.id);
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {}
