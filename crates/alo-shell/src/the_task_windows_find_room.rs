//! **A task window opened into a full view finds room** as every other window
//! the person opens does: the nearest empty space on the Place, covering
//! nothing, with the view gliding there — or jumping, with less motion asked
//! for.
//!
//! The owner's *"Yes, the same"* of 2026-11-07, asked whether alo's own windows
//! glide too (`crate::room_in_a_full_view`). A task window is placed by
//! `crate::the_tasks_a_person_opened::TheTasks`, which holds no view of the
//! other windows; this hands it them, and the person's choice, and moves the
//! view where it says. A task window opens only for something the person asked
//! for, so this asks every time.

use smithay::utils::{Logical, Rectangle, Size};

impl crate::Server {
    /// Place every task window that has nowhere to be, finding room where the
    /// view is full. Answers how many were placed.
    pub(crate) fn the_task_windows_find_room(
        &mut self,
        view: Rectangle<i32, Logical>,
        reading: alo_strings::Direction,
        display_scale: u16,
    ) -> usize {
        let others = self.the_windows_on_this_place();
        let reach = self.as_far_as_show_all_reaches().and_then(|reach| {
            Some(Size::<i32, Logical>::from((
                i32::try_from(reach.width()).ok()?,
                i32::try_from(reach.height()).ok()?,
            )))
        });
        let full = crate::where_a_task_window_opens::WhenTheViewIsFull {
            others: &others,
            glides: self.when_there_is_no_room.glides(),
            reach,
        };
        let (placed, goes_to) = self
            .tasks_mut()
            .put_whatever_has_nowhere_to_be_finding_room(view, reading, &[], display_scale, &full);
        if let Some(to) = goes_to.and_then(|to| alo_canvas::At::checked(to.x, to.y)) {
            let _moving = self.the_view_glides_to(to, std::time::Instant::now());
        }
        placed
    }
}
