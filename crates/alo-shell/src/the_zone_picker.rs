//! **Which time zone, in a region that has several** — asked after *Use this
//! region* on the first-start sheets, before the overview comes back.
//!
//! The owner's decision of 2026-10-28: the region picked sets the machine's
//! zone, and where a country has several the person is asked which. **No frame
//! of the design draws this sheet**, so it is the setup sheets' own
//! composition — the region sheet's, row for row — over a window of the
//! region's zones, each named by its exemplar city in the person's language
//! (`alo_formats::zones`). Nothing arrives chosen: a country's zones have no
//! honest first one, and guessing New York for somebody in Los Angeles is the
//! silent default the owner ruled out for regions.

use alo_formats::zones::NamedZone;

use crate::the_languages_offered::{ROWS_AT_A_TIME, WhereTheRingIs};

/// The zones, what is chosen, and where the cursor and ring are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheZonePicker {
    /// The region's zones, named and in the language's order.
    all: Vec<NamedZone>,
    /// Which is chosen, once one is.
    chosen: Option<usize>,
    /// Where the cursor is.
    on: usize,
    /// Where the ring is.
    ring: WhereTheRingIs,
}

impl TheZonePicker {
    /// These zones, with nothing chosen.
    pub(crate) fn of(all: Vec<NamedZone>) -> Self {
        Self {
            all,
            chosen: None,
            on: 0,
            ring: WhereTheRingIs::OnTheList,
        }
    }

    /// The zone chosen.
    pub fn chosen(&self) -> Option<&NamedZone> {
        self.all.get(self.chosen?)
    }

    /// Where the ring is.
    pub fn ring(&self) -> WhereTheRingIs {
        self.ring
    }

    /// Down one, stopping at the last.
    pub(crate) fn down(&mut self) {
        self.ring = WhereTheRingIs::OnTheList;
        self.on = self
            .on
            .saturating_add(1)
            .min(self.all.len().saturating_sub(1));
    }

    /// Up one, stopping at the first.
    pub(crate) fn up(&mut self) {
        self.ring = WhereTheRingIs::OnTheList;
        self.on = self.on.saturating_sub(1);
    }

    /// **A press on a row**, by where it is in the whole list: the cursor to
    /// it and the ring to the list, so the Enter that follows chooses it.
    /// Answers whether there is such a row.
    pub(crate) fn pointed_at(&mut self, row: usize) -> bool {
        let there = row < self.all.len();
        if there {
            self.ring = WhereTheRingIs::OnTheList;
            self.on = row;
        }
        there
    }

    /// **A window of `rows` rows** onto the zones (`crate::a_window_onto_a_list`).
    pub(crate) fn window_of(
        &self,
        rows: usize,
    ) -> crate::a_window_onto_a_list::AWindow<'_, NamedZone> {
        crate::a_window_onto_a_list::a_window(&self.all, self.on, self.chosen, rows)
    }

    /// **A press on the way on**: the ring to it only once a zone is chosen,
    /// which is Tab's rule. Answers whether it went.
    pub(crate) fn ring_to_the_way_on_if_it_may(&mut self) -> bool {
        if self.chosen.is_some() {
            self.ring = WhereTheRingIs::OnTheWayOn;
        }
        self.chosen.is_some()
    }

    /// Tab: to the way on and back, only once a zone is chosen.
    pub(crate) fn ring_crosses(&mut self) {
        self.ring = match (self.ring, self.chosen) {
            (WhereTheRingIs::OnTheList, Some(_)) => WhereTheRingIs::OnTheWayOn,
            (WhereTheRingIs::OnTheList, None) | (WhereTheRingIs::OnTheWayOn, _) => {
                WhereTheRingIs::OnTheList
            }
        };
    }

    /// Enter on the list: the zone under the cursor is chosen, and the ring
    /// goes to the way on.
    pub(crate) fn choose(&mut self) {
        if self.on < self.all.len() {
            self.chosen = Some(self.on);
            self.ring = WhereTheRingIs::OnTheWayOn;
        }
    }

    /// The first zone in the window.
    fn first_in_the_window(&self) -> usize {
        let last_start = self.all.len().saturating_sub(ROWS_AT_A_TIME);
        self.on.saturating_sub(ROWS_AT_A_TIME / 2).min(last_start)
    }

    /// The zones drawn now.
    pub fn window(&self) -> &[NamedZone] {
        let from = self.first_in_the_window();
        let to = from.saturating_add(ROWS_AT_A_TIME).min(self.all.len());
        self.all.get(from..to).unwrap_or_default()
    }

    /// Where in the window the cursor is.
    pub fn cursor_in_the_window(&self) -> Option<usize> {
        self.on.checked_sub(self.first_in_the_window())
    }

    /// Where in the window the chosen zone is, while it is in it.
    pub fn chosen_in_the_window(&self) -> Option<usize> {
        let within = self.chosen?.checked_sub(self.first_in_the_window())?;
        (within < ROWS_AT_A_TIME).then_some(within)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn american() -> TheZonePicker {
        TheZonePicker::of(alo_formats::zones::the_zones_in(
            &alo_formats::RegionCode::named("US").unwrap(),
            "en",
        ))
    }

    /// **Nothing arrives chosen**, and the way on cannot be reached until a
    /// zone is.
    #[test]
    fn nothing_arrives_chosen_and_the_way_on_waits_for_a_zone() {
        let mut picker = american();
        assert_eq!(picker.chosen(), None);
        picker.ring_crosses();
        assert_eq!(picker.ring(), WhereTheRingIs::OnTheList);
        picker.down();
        picker.choose();
        assert!(picker.chosen().is_some());
        assert_eq!(picker.ring(), WhereTheRingIs::OnTheWayOn);
        assert_eq!(picker.chosen_in_the_window(), picker.cursor_in_the_window());
    }
}
