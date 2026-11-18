// Real-bus session ordering and identity checks; no synthetic screen is claimed.
use super::*;

const CAST: &str = "org.freedesktop.portal.ScreenCast";
const STUDIO: &str = "org.aloworld.Studio";

fn response(
    caller: &Connection,
    token: &str,
    invoke: impl FnOnce() -> OwnedObjectPath,
) -> (u32, HashMap<String, OwnedValue>) {
    let path =
        alo_portals::handle_for(caller.unique_name().unwrap().as_str(), Some(token)).unwrap();
    let request = Proxy::new(
        caller,
        DESKTOP,
        path.as_str(),
        "org.freedesktop.portal.Request",
    )
    .unwrap();
    let mut responses = request.receive_signal("Response").unwrap();
    assert_eq!(invoke().as_str(), path);
    responses.next().unwrap().body().deserialize().unwrap()
}

fn create(caller: &Connection, token: &str) -> (u32, Option<OwnedObjectPath>) {
    let (code, mut result) = response(caller, token, || {
        caller
            .call_method(
                Some(DESKTOP),
                PORTALS,
                Some(CAST),
                "CreateSession",
                &(HashMap::from([
                    ("handle_token", Value::from(token)),
                    ("session_handle_token", Value::from(token)),
                ]),),
            )
            .unwrap()
            .body()
            .deserialize()
            .unwrap()
    });
    let session = result
        .remove("session_handle")
        .map(|v| OwnedObjectPath::try_from(String::try_from(v).unwrap()).unwrap());
    (code, session)
}

fn select<'a>(
    caller: &Connection,
    session: &OwnedObjectPath,
    token: &'a str,
    mut options: HashMap<&'static str, Value<'a>>,
) -> u32 {
    options.insert("handle_token", Value::from(token));
    response(caller, token, || {
        caller
            .call_method(
                Some(DESKTOP),
                PORTALS,
                Some(CAST),
                "SelectSources",
                &(session, options),
            )
            .unwrap()
            .body()
            .deserialize()
            .unwrap()
    })
    .0
}

fn embedded() -> HashMap<&'static str, Value<'static>> {
    HashMap::from([("cursor_mode", Value::from(2_u32))])
}

#[test]
fn creating_a_recording_session_does_not_grant_or_share_anything() {
    let serving = Serving::started("capture-prepare");
    serving.become_(Some(STUDIO));
    let (code, session) = create(&serving.client, "prepare");
    assert_eq!(code, 0);
    assert!(session.is_some());
    assert!(serving.machine.grants.read().unwrap().is_empty());
    assert!(matches!(
        serving.last_answer().outcome(),
        Outcome::CapturePrepared
    ));
}

#[test]
fn an_unidentified_caller_cannot_create_a_recording_session() {
    let serving = Serving::started("capture-unidentified");
    serving.become_(None);
    assert_eq!(create(&serving.client, "unidentified"), (2, None));
    assert!(matches!(
        serving.last_answer().outcome(),
        Outcome::Unanswered(Unanswered::NotIdentified)
    ));
}

#[test]
fn another_connection_with_the_same_app_and_pid_cannot_use_or_close_the_session() {
    let serving = Serving::started("capture-owner");
    serving.become_(Some(STUDIO));
    let (_, session) = create(&serving.client, "owned");
    let session = session.unwrap();
    let other = zbus::blocking::connection::Builder::address(serving._bus.address().as_str())
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(select(&other, &session, "foreign_selection", embedded()), 2);
    assert!(
        other
            .call_method(
                Some(DESKTOP),
                session.as_str(),
                Some("org.freedesktop.portal.Session"),
                "Close",
                &()
            )
            .is_err()
    );
    assert_eq!(
        select(&serving.client, &session, "owner_selection", embedded()),
        0
    );
    assert!(serving.machine.grants.read().unwrap().is_empty());
}

#[test]
fn unsupported_sharing_modes_close_the_session_without_a_grant() {
    let serving = Serving::started("capture-unsupported");
    serving.become_(Some(STUDIO));
    for (index, (key, value)) in [
        ("persist_mode", Value::from(2_u32)),
        ("multiple", Value::from(true)),
        ("cursor_mode", Value::from(4_u32)),
        ("types", Value::from(4_u32)),
    ]
    .into_iter()
    .enumerate()
    {
        let token = format!("unsupported_{index}");
        let (_, session) = create(&serving.client, &token);
        let session = session.unwrap();
        let mut options = embedded();
        options.insert(key, value);
        assert_eq!(
            select(
                &serving.client,
                &session,
                &format!("invalid_{index}"),
                options
            ),
            2
        );
        assert_eq!(
            select(
                &serving.client,
                &session,
                &format!("retry_{index}"),
                embedded()
            ),
            2
        );
    }
    assert!(serving.machine.grants.read().unwrap().is_empty());
}

#[test]
fn selecting_sources_twice_is_refused_and_cannot_reuse_the_session() {
    let serving = Serving::started("capture-repeat");
    serving.become_(Some(STUDIO));
    let (_, session) = create(&serving.client, "repeat");
    let session = session.unwrap();
    assert_eq!(select(&serving.client, &session, "first", embedded()), 0);
    assert_eq!(select(&serving.client, &session, "second", embedded()), 2);
    assert_eq!(select(&serving.client, &session, "third", embedded()), 2);
}

#[test]
fn an_unavailable_compositor_never_uses_an_old_capture_grant() {
    let serving = Serving::started("capture-no-compositor");
    serving.become_(Some(STUDIO));
    serving.grant(
        STUDIO,
        Reach::Facility(alo_capability::Facility::ScreenContinuously),
    );
    let before = serving.machine.grants.read().unwrap().clone();
    let (_, session) = create(&serving.client, "no_compositor");
    let session = session.unwrap();
    assert_eq!(select(&serving.client, &session, "types", embedded()), 0);
    assert_eq!(
        response(&serving.client, "start", || serving
            .client
            .call_method(
                Some(DESKTOP),
                PORTALS,
                Some(CAST),
                "Start",
                &(
                    &session,
                    "",
                    HashMap::from([("handle_token", Value::from("start"))])
                )
            )
            .unwrap()
            .body()
            .deserialize()
            .unwrap())
        .0,
        2
    );
    assert_eq!(*serving.machine.grants.read().unwrap(), before);
    assert!(matches!(
        serving.last_answer().outcome(),
        Outcome::Unanswered(Unanswered::CaptureUnavailable)
    ));
}
