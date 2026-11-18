//! Start alo Studio as an independently installed native desktop application.

use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    // Native dialogs without an explicit GtkApplication use this Wayland ID.
    gtk::glib::set_prgname(Some(alo_studio::APPLICATION_ID));
    gtk::glib::set_application_name("alo Studio");
    let application = gtk::Application::builder()
        .application_id(alo_studio::APPLICATION_ID)
        .build();
    application.connect_activate(|application| {
        if let Some(window) = application.active_window() {
            window.present();
            return;
        }
        alo_studio::ui::open(application);
    });
    application.run()
}
