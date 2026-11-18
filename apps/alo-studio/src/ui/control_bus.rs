//! Versioned semantic D-Bus controls; callbacks run on GTK's owning thread.

use super::*;
use crate::control::Command;

const OBJECT: &str = "/org/aloworld/Studio/Recorder";
const INTERFACE: &str = "org.aloworld.Studio.Recorder1";
const XML: &str = include_str!("../../data/org.aloworld.Studio.Recorder1.xml");

/// Keep registration tied to this exact application window's lifetime.
pub(super) fn register(state: &Rc<State>, application: &gtk::Application) {
    let result = (|| {
        let connection = application.dbus_connection().ok_or("no session bus")?;
        let node = gio::DBusNodeInfo::for_xml(XML).map_err(|_| "invalid control interface")?;
        let interface = node
            .lookup_interface(INTERFACE)
            .ok_or("missing control interface")?;
        let weak = Rc::downgrade(state);
        let registration = connection
            .register_object(OBJECT, &interface)
            .method_call(move |_, _, _, _, method, parameters, invocation| {
                let Some(state) = weak.upgrade().filter(|state| !state.closed.get()) else {
                    invocation.return_dbus_error(
                        "org.aloworld.Studio.Error.Unavailable",
                        &t("alo Studio is closed."),
                    );
                    return;
                };
                if method == "GetStatus" {
                    invocation
                        .return_value(Some(&(state.automation_status().to_string(),).to_variant()));
                    return;
                }
                let command = command(method, &parameters);
                match command.and_then(|command| state.automation_command(command)) {
                    Ok(()) => invocation.return_value(Some(&().to_variant())),
                    Err((name, message)) => invocation.return_dbus_error(name, &message),
                }
            })
            .build()
            .map_err(|_| "could not register control interface")?;
        *state.automation.borrow_mut() = Some((connection, registration));
        Ok::<(), &'static str>(())
    })();
    if let Err(error) = result {
        eprintln!("alo Studio automation unavailable: {error}");
    }
}

type Error = (&'static str, String);

fn command(method: &str, parameters: &glib::Variant) -> Result<Command, Error> {
    let invalid = || {
        (
            "org.freedesktop.DBus.Error.InvalidArgs",
            t("Choose a valid recording option."),
        )
    };
    let text = || {
        parameters
            .get::<(String,)>()
            .map(|value| value.0)
            .ok_or_else(invalid)
    };
    Ok(match method {
        "NewRecording" => Command::NewRecording,
        "ChooseScreen" => Command::ChooseScreen,
        "ChooseWindow" => Command::ChooseWindow,
        "Start" => Command::Start,
        "Pause" => Command::Pause,
        "Resume" => Command::Resume,
        "Finish" => Command::Finish,
        "EnableMicrophone" => Command::Microphone(true),
        "DisableMicrophone" => Command::Microphone(false),
        "SetQuality" => Command::Quality(match text()?.as_str() {
            "full_hd" => Quality::FullHd,
            "hd" => Quality::Hd,
            "original" => Quality::Original,
            "lightweight" => Quality::Lightweight,
            _ => return Err(invalid()),
        }),
        "SetCountdown" => {
            let (seconds,) = parameters.get::<(i64,)>().ok_or_else(invalid)?;
            if !matches!(seconds, 0 | 3 | 5) {
                return Err(invalid());
            }
            Command::Countdown(seconds as u8)
        }
        "CancelCountdown" => Command::CancelCountdown,
        "ShowLibrary" => Command::ShowLibrary,
        "ReviewRecording" => Command::Review(text()?),
        "RenameRecording" => {
            let title = text()?;
            if title.trim().is_empty()
                || title.chars().count() > 200
                || title.chars().any(char::is_control)
            {
                return Err(invalid());
            }
            Command::Rename(title.trim().to_owned())
        }
        "ExportRecording" => Command::Export,
        _ => {
            return Err((
                "org.freedesktop.DBus.Error.UnknownMethod",
                t("That recording action is unavailable."),
            ));
        }
    })
}
