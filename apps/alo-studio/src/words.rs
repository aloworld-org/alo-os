//! User-facing copy has a translation boundary from the first release.

/// Look up the application's gettext catalogue, falling back to its English key.
pub fn t(message: &str) -> String {
    gtk::glib::dgettext(Some("alo-studio"), message).to_string()
}
