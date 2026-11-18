//! Fixed requests at permission boundaries chosen by the person, never here.

use alo_protocol::{Done, FromAnAgent, ToAnAgent};

use crate::studio_agent_probe::{Stage, require_discovery, require_refused};

/// Check five permission boundaries using one caller-owned agent connection.
///
/// `boundary` returns only after the person-side controller has made the native
/// Settings choice. Discovery is checked immediately afterward, without retries
/// that could conceal delayed withdrawal. `ask` must retain the same connection
/// throughout; this function creates no transport, grants or approvals.
///
/// # Errors
/// Stop on the first boundary, transport, discovery or application-state error.
/// This probe never successfully changes Studio; actual approved execution and
/// execution records belong to the separate native approval/model walk.
pub fn run(
    ask: &mut impl FnMut(&FromAnAgent) -> Result<ToAnAgent, String>,
    boundary: &mut impl FnMut(Stage) -> Result<(), String>,
) -> Result<(), String> {
    for stage in [
        Stage::AskBeforeGrant,
        Stage::Granted,
        Stage::Revoked,
        Stage::Full,
        Stage::AskAgain,
    ] {
        boundary(stage)?;
        require_discovery(stage, &ask(&FromAnAgent::Capabilities)?)?;
        let status = ask(&FromAnAgent::Read {
            verb: "studio.status".to_owned(),
            given: Vec::new(),
        })?;
        match stage {
            Stage::Granted | Stage::Full => require_ready(&status)?,
            Stage::AskBeforeGrant | Stage::Revoked | Stage::AskAgain => {
                require_refused(&status)?;
                require_refused(&ask(&FromAnAgent::Propose {
                    verb: "studio.enable_microphone".to_owned(),
                    given: Vec::new(),
                })?)?;
            }
        }
    }
    Ok(())
}

/// Establish that the fixed proposed operation would be valid if permitted.
fn require_ready(reply: &ToAnAgent) -> Result<(), &'static str> {
    let ToAnAgent::Did(Done::ApplicationControlled {
        application,
        action,
        reply: Some(body),
        unanswered: false,
    }) = reply
    else {
        return Err("Studio did not answer the granted status read");
    };
    if application != "org.aloworld.Studio" || action != "studio.status" {
        return Err("the status reply belongs to another application or action");
    }
    let state: serde_json::Value =
        serde_json::from_str(body).map_err(|_| "Studio status is not valid JSON")?;
    if state.get("version").and_then(serde_json::Value::as_u64) != Some(1)
        || state.get("phase").and_then(serde_json::Value::as_str) != Some("ready")
        || state
            .get("microphone_enabled")
            .and_then(serde_json::Value::as_bool)
            != Some(false)
        || state
            .get("microphone_active")
            .and_then(serde_json::Value::as_bool)
            != Some(false)
    {
        return Err("the permission probe needs Ready Studio with its microphone off and inactive");
    }
    Ok(())
}
