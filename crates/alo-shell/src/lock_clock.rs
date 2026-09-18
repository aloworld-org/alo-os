//! Civil time from the selected timezone, formatted by the existing CLDR owner.
use crate::RenderError;
use alo_formats::{Regionally, Timezone};
use std::time::SystemTime;

/// Convert a policy moment in the selected zone, then ask CLDR to write it.
pub(crate) fn written(
    at: SystemTime,
    zone: &Timezone,
    region: &Regionally,
) -> Result<String, RenderError> {
    let instant = jiff::Timestamp::try_from(at).map_err(|_| RenderError::LockScene)?;
    let local = instant
        .in_tz(zone.name())
        .map_err(|_| RenderError::LockScene)?;
    region
        .time(local.hour() as u8, local.minute() as u8)
        .ok_or(RenderError::LockScene)
}
