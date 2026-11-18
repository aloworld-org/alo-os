//! Call SaveFile from inside the actual recorder sandbox and copy a test video.
//!
//! This is a verification binary, never shipped with the recorder or the OS.
//! The native chooser remains the person's only way to select its destination.

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    use zbus::{
        blocking::{Connection, Proxy},
        zvariant::{OwnedObjectPath, OwnedValue, Value},
    };
    let source = std::env::args()
        .nth(1)
        .ok_or("a private test recording is required")?;
    if source == "--check-write" {
        let uri = std::env::args()
            .nth(2)
            .ok_or("an exported document URI is required")?;
        let expected = std::env::args()
            .nth(3)
            .ok_or("expected access is required")?;
        let path = url::Url::parse(&uri)?
            .to_file_path()
            .map_err(|()| "the document URI is not local")?;
        let access = std::fs::OpenOptions::new().write(true).open(path);
        match (expected.as_str(), access) {
            ("allowed", Ok(_)) => {}
            ("denied", Err(why))
                if matches!(
                    why.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
                ) => {}
            ("allowed", Err(why)) => {
                return Err(format!("the selected export was not writable: {why}").into());
            }
            ("denied", Ok(_)) => return Err("the export remained writable after revocation".into()),
            _ => return Err("the access check did not produce the expected result".into()),
        }
        println!("native-save-access: {expected}");
        return Ok(());
    }
    let client = Connection::session()?;
    let sender = client
        .unique_name()
        .ok_or("the bus did not name this caller")?
        .as_str();
    let handle = alo_portals::handle_for(sender, Some("native_save_check"))
        .map_err(|_| "the request handle could not be made")?;
    let request = Proxy::new(
        &client,
        "org.freedesktop.portal.Desktop",
        handle.as_str(),
        "org.freedesktop.portal.Request",
    )?;
    let mut responses = request.receive_signal("Response")?;
    let options = HashMap::from([
        ("handle_token", Value::from("native_save_check")),
        ("current_name", Value::from("demo-recording.webm")),
    ]);
    let reply = client.call_method(
        Some("org.freedesktop.portal.Desktop"),
        "/org/freedesktop/portal/desktop",
        Some("org.freedesktop.portal.FileChooser"),
        "SaveFile",
        &("", "Save your recording", options),
    )?;
    let returned: OwnedObjectPath = reply.body().deserialize()?;
    if returned.as_str() != handle {
        return Err("the returned handle belonged to another request".into());
    }
    let response = responses
        .next()
        .ok_or("the frontend did not send an answer")?;
    let (code, mut results): (u32, HashMap<String, OwnedValue>) = response.body().deserialize()?;
    if code == 1 {
        println!("native-save-cancelled");
        return Ok(());
    }
    if code != 0 {
        return Err(format!("native save was refused with response {code}").into());
    }
    let uris = Vec::<String>::try_from(
        results
            .remove("uris")
            .ok_or("no destination was returned")?,
    )?;
    if uris.len() != 1 {
        return Err("the frontend returned more than one destination".into());
    }
    let uri = uris.first().ok_or("no destination was returned")?;
    let target = url::Url::parse(uri)?
        .to_file_path()
        .map_err(|()| "the destination was not a local exported document")?;
    let copied = std::fs::copy(source, &target)?;
    println!("native-save-complete: {copied} bytes; {uri}");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("The sandbox save integration probe requires Linux.");
}
