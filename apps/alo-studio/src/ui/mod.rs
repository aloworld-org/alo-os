//! Native GTK surfaces and the actions that join them to the recording engine.

mod controller;
mod layout;
mod review;

/// Open the app's one native window.
pub fn open(application: &gtk::Application) {
    controller::open(application);
}
