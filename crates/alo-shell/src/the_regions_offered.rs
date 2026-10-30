//! **The region a machine is set up in**, as frame `09c · Choose region`
//! offers it: the region the machine already thinks it is in, chosen, and
//! *Other region* under it — which opens every region, named in the language
//! the person just picked, in that language's order.
//!
//! # What the design draws, and the one thing it does not
//!
//! `09c` draws two rows: `Belgium · Selected` over *Selected region*, and
//! *Other region* over *Choose another country or region.* It does not draw
//! what *Other region* opens. **The owner's answer of 2026-10-28**: the same
//! list Settings uses, named in the person's language and worked with the
//! arrow keys. So *Other region* turns the two rows into a window onto that
//! list — three rows at a time, the language sheet's own window
//! (`crate::the_languages_offered::ROWS_AT_A_TIME`) — and choosing a row turns
//! it back into the two, with the region just chosen in the first.
//!
//! # Which region arrives chosen
//!
//! **The region the machine is in, as far as it honestly knows** — the
//! owner's decision of 2026-10-28: the country of its time zone where it keeps
//! one that is a place, and otherwise the region CLDR writes the chosen
//! language in. The second is `US` for English, and that is why the row says
//! its name: a person somewhere else sees *United States · Selected* and
//! picks where they are, rather than being written to as an American without
//! being asked. The design draws the row arriving chosen, and it is: the
//! person sees the choice before they take *Use this region*.
//!
//! # Nothing here writes anything
//!
//! The choice leaves through `SignInScreen::the_region_chosen`, and the
//! compositor keeps it (`crate::the_region_a_person_chose`).

use alo_formats::{NamedRegion, RegionCode};

use crate::the_languages_offered::{ROWS_AT_A_TIME, WhereTheRingIs};

/// What the sheet is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Showing {
    /// The design's two rows: the chosen region, and *Other region*.
    TheTwo,
    /// Every region, a window of them at a time.
    EveryRegion,
}

/// The rows of the region sheet in sight: `crate::a_window_onto_a_list::AWindow`,
/// owned, because the design's two rows are made rather than kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegionsInSight<'a> {
    /// Where in what is showing the first row drawn is.
    pub(crate) first: usize,
    /// Each row as the name it reads; [`None`] is *Other region*.
    pub(crate) rows: Vec<Option<&'a str>>,
    /// The cursor's row among them.
    pub(crate) cursor: Option<usize>,
    /// The chosen region's row among them, while it is in sight.
    pub(crate) chosen: Option<usize>,
}

/// The regions, what is chosen, and where the cursor and the ring are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheRegionsOffered {
    /// Every region, named in the person's language and in its order.
    all: Vec<NamedRegion>,
    /// Which row of `all` is chosen. Always one: the sheet arrives with the
    /// region the machine thinks it is in.
    chosen: usize,
    /// Which of the two views is drawn.
    showing: Showing,
    /// Where the cursor is: `0` or `1` on the two rows, a row of `all` on
    /// every region.
    on: usize,
    /// Where the focus ring is.
    ring: WhereTheRingIs,
}

impl TheRegionsOffered {
    /// Every region named in `language`, with `where_it_is` chosen — or, where
    /// that is not on the list, the first row.
    pub(crate) fn named_in(language: &str, where_it_is: Option<&RegionCode>) -> Self {
        let all = alo_formats::the_regions_named_in(language);
        let chosen = where_it_is
            .and_then(|region| all.iter().position(|it| it.code == region.code()))
            .unwrap_or_default();
        Self {
            all,
            chosen,
            showing: Showing::TheTwo,
            on: 0,
            ring: WhereTheRingIs::OnTheList,
        }
    }

    /// Which view is drawn.
    pub fn showing(&self) -> Showing {
        self.showing
    }

    /// Where the focus ring is.
    pub fn ring(&self) -> WhereTheRingIs {
        self.ring
    }

    /// The region chosen.
    pub fn chosen(&self) -> Option<&NamedRegion> {
        self.all.get(self.chosen)
    }

    /// Down one row, stopping at the last.
    pub(crate) fn down(&mut self) {
        self.ring = WhereTheRingIs::OnTheList;
        let last = match self.showing {
            Showing::TheTwo => 1,
            Showing::EveryRegion => self.all.len().saturating_sub(1),
        };
        self.on = self.on.saturating_add(1).min(last);
    }

    /// Up one row, stopping at the first.
    pub(crate) fn up(&mut self) {
        self.ring = WhereTheRingIs::OnTheList;
        self.on = self.on.saturating_sub(1);
    }

    /// **A press on a row**, by where it is in what is showing — the two rows,
    /// or every region: the cursor to it and the ring to the list, so the Enter
    /// that follows does what Enter on that row does. Answers whether there is
    /// such a row.
    pub(crate) fn pointed_at(&mut self, row: usize) -> bool {
        let there = match self.showing {
            Showing::TheTwo => row < 2,
            Showing::EveryRegion => row < self.all.len(),
        };
        if there {
            self.ring = WhereTheRingIs::OnTheList;
            self.on = row;
        }
        there
    }

    /// **A window of `rows` rows onto what is showing**, each as the name it
    /// reads — the chosen region and [`None`] for *Other region* on the two
    /// rows (`crate::a_window_onto_a_list`). Owned, because the two rows are
    /// made rather than kept.
    pub(crate) fn window_of(&self, rows: usize) -> RegionsInSight<'_> {
        let names: Vec<Option<&str>> = match self.showing {
            Showing::TheTwo => vec![self.chosen().map(|it| it.name.as_str()), None],
            Showing::EveryRegion => self.all.iter().map(|it| Some(it.name.as_str())).collect(),
        };
        let chosen = match self.showing {
            Showing::TheTwo => 0,
            Showing::EveryRegion => self.chosen,
        };
        let window = crate::a_window_onto_a_list::a_window(&names, self.on, Some(chosen), rows);
        RegionsInSight {
            first: window.first,
            rows: window.rows.to_vec(),
            cursor: window.cursor,
            chosen: window.chosen,
        }
    }

    /// **A press on *Use this region***: a region is always chosen, so the
    /// ring may always go there.
    pub(crate) fn ring_to_the_way_on(&mut self) {
        self.ring = WhereTheRingIs::OnTheWayOn;
    }

    /// Tab: between the list and *Use this region*. There is always a region
    /// chosen, so the way on is always somewhere a ring may go.
    pub(crate) fn ring_crosses(&mut self) {
        self.ring = match self.ring {
            WhereTheRingIs::OnTheList => WhereTheRingIs::OnTheWayOn,
            WhereTheRingIs::OnTheWayOn => WhereTheRingIs::OnTheList,
        };
    }

    /// **Enter on the list**: on the chosen row it is chosen again and the
    /// ring goes to the way on; on *Other region* every region opens with the
    /// cursor on the one chosen; on a row of every region that row is chosen
    /// and the two rows come back, with the ring on the way on.
    pub(crate) fn enter_on_the_list(&mut self) {
        match (self.showing, self.on) {
            (Showing::TheTwo, 0) => self.ring = WhereTheRingIs::OnTheWayOn,
            (Showing::TheTwo, _) => {
                self.showing = Showing::EveryRegion;
                self.on = self.chosen;
            }
            (Showing::EveryRegion, on) => {
                if on < self.all.len() {
                    self.chosen = on;
                }
                self.showing = Showing::TheTwo;
                self.on = 0;
                self.ring = WhereTheRingIs::OnTheWayOn;
            }
        }
    }

    /// **Escape**: from every region, back to the two rows with nothing
    /// changed. Answers whether there was somewhere to go back to within the
    /// sheet; `false` means the sheet itself goes back.
    pub(crate) fn back(&mut self) -> bool {
        match self.showing {
            Showing::EveryRegion => {
                self.showing = Showing::TheTwo;
                self.on = 1;
                self.ring = WhereTheRingIs::OnTheList;
                true
            }
            Showing::TheTwo => false,
        }
    }

    /// The first row of every region in the window, derived from the cursor
    /// the way the language sheet's is.
    fn first_in_the_window(&self) -> usize {
        let last_start = self.all.len().saturating_sub(ROWS_AT_A_TIME);
        self.on.saturating_sub(ROWS_AT_A_TIME / 2).min(last_start)
    }

    /// **The rows drawn now**, as the names they read: on the two rows the
    /// chosen region and [`None`] for *Other region*, whose words are the
    /// sheet's; on every region, a window of names.
    pub fn rows(&self) -> Vec<Option<&str>> {
        match self.showing {
            Showing::TheTwo => vec![self.chosen().map(|it| it.name.as_str()), None],
            Showing::EveryRegion => {
                let from = self.first_in_the_window();
                let to = from.saturating_add(ROWS_AT_A_TIME).min(self.all.len());
                self.all
                    .get(from..to)
                    .unwrap_or_default()
                    .iter()
                    .map(|it| Some(it.name.as_str()))
                    .collect()
            }
        }
    }

    /// Where in the rows drawn the cursor is.
    pub fn cursor_in_the_rows(&self) -> Option<usize> {
        match self.showing {
            Showing::TheTwo => Some(self.on),
            Showing::EveryRegion => self.on.checked_sub(self.first_in_the_window()),
        }
    }

    /// Where in the rows drawn the chosen region is — always the first of the
    /// two, and on every region only while it is in the window.
    pub fn chosen_in_the_rows(&self) -> Option<usize> {
        match self.showing {
            Showing::TheTwo => Some(0),
            Showing::EveryRegion => {
                let within = self.chosen.checked_sub(self.first_in_the_window())?;
                (within < ROWS_AT_A_TIME).then_some(within)
            }
        }
    }
}

#[cfg(test)]
#[path = "the_regions_offered_tests.rs"]
mod tests;
