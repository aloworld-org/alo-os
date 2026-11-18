//! Private package/evidence transfer for one VM recording walk. Never shipped.

use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use alo_walking::studio_walk_files::Files;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = None;
    let mut evidence = None;
    let mut port = 8767_u16;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        let value = arguments.next().ok_or("each argument needs a value")?;
        match argument.as_str() {
            "--inputs" => inputs = Some(PathBuf::from(value)),
            "--evidence" => evidence = Some(PathBuf::from(value)),
            "--port" => port = value.parse()?,
            _ => return Err("use --inputs DIRECTORY --evidence DIRECTORY [--port PORT]".into()),
        }
    }
    let files = Files::new(
        inputs.ok_or("pass --inputs")?,
        evidence.ok_or("pass --evidence")?,
    )?;
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
    println!("STUDIO_WALK_FILES_READY {}", listener.local_addr()?.port());
    for connection in listener.incoming() {
        let mut connection = connection?;
        connection.set_read_timeout(Some(Duration::from_secs(15)))?;
        connection.set_write_timeout(Some(Duration::from_secs(15)))?;
        if let Err(why) = files.answer(&mut connection) {
            eprintln!("Studio walk transfer failed: {why}");
        }
    }
    Ok(())
}
