//! The recording workspace: source, sound, preview, and one primary action.

use crate::words::t;
use gtk::prelude::*;

/// The widgets whose state the controller updates.
#[derive(Clone)]
pub struct View {
    pub window: gtk::ApplicationWindow,
    pub pages: gtk::Stack,
    pub new_button: gtk::Button,
    pub library_button: gtk::Button,
    pub help_button: gtk::Button,
    pub library_count: gtk::Label,
    pub screen: gtk::ToggleButton,
    pub one_window: gtk::ToggleButton,
    pub microphone: gtk::Switch,
    pub microphone_text: gtk::Label,
    pub quality: gtk::DropDown,
    pub countdown: gtk::DropDown,
    pub preview: gtk::Picture,
    pub placeholder: gtk::Box,
    pub countdown_number: gtk::Label,
    pub timer: gtk::Label,
    pub status: gtk::Label,
    pub source_name: gtk::Label,
    pub record: gtk::Button,
    pub pause: gtk::Button,
    pub cancel: gtk::Button,
    pub banner: gtk::Revealer,
    pub banner_text: gtk::Label,
    pub library_list: gtk::ListBox,
    pub search: gtk::SearchEntry,
    pub review: super::review::Review,
}

pub fn label(text: &str, class: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(&t(text)));
    label.set_xalign(0.0);
    label.add_css_class(class);
    label
}

pub fn icon_button(icon: &str, title: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(t(title))
        .build();
    button.update_property(&[gtk::accessible::Property::Label(&t(title))]);
    button
}

fn card(icon: &str, title: &str, subtitle: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(22);
    row.append(&image);
    let copy = gtk::Box::new(gtk::Orientation::Vertical, 4);
    copy.append(&label(title, "card-title"));
    copy.append(&label(subtitle, "muted"));
    row.append(&copy);
    row
}

pub fn build(application: &gtk::Application) -> View {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(t("alo Studio"))
        .default_width(1100)
        .default_height(830)
        .width_request(780)
        .height_request(640)
        .build();
    window.add_css_class("studio");
    let header = gtk::HeaderBar::new();
    header.add_css_class("studio-header");
    header.set_title_widget(Some(&gtk::Label::new(Some(&t("alo Studio")))));
    window.set_titlebar(Some(&header));
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    window.set_child(Some(&root));

    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 8);
    sidebar.set_width_request(205);
    sidebar.add_css_class("sidebar");
    let brand = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    brand.add_css_class("brand");
    let mark = gtk::Image::from_file(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/org.aloworld.Studio.svg"
    ));
    // Installed builds use the icon theme; development builds read the same SVG.
    if !std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/org.aloworld.Studio.svg"
    ))
    .exists()
    {
        mark.set_icon_name(Some(crate::APPLICATION_ID));
    }
    mark.set_pixel_size(34);
    brand.append(&mark);
    let names = gtk::Box::new(gtk::Orientation::Vertical, 0);
    names.append(&label("alo Studio", "brand-name"));
    names.append(&label("A little more clarity", "brand-caption"));
    brand.append(&names);
    sidebar.append(&brand);
    let new_button = gtk::Button::new();
    new_button.set_child(Some(&card("list-add-symbolic", "New recording", "")));
    new_button.add_css_class("nav-button");
    new_button.add_css_class("nav-active");
    let library_button = gtk::Button::new();
    library_button.set_child(Some(&card("folder-videos-symbolic", "Your recordings", "")));
    library_button.add_css_class("nav-button");
    sidebar.append(&new_button);
    sidebar.append(&library_button);
    let library_count = label("No recordings yet", "library-count");
    sidebar.append(&library_count);
    let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar.append(&spacer);
    let privacy = gtk::Box::new(gtk::Orientation::Vertical, 8);
    privacy.add_css_class("privacy-note");
    let lock = gtk::Image::from_icon_name("changes-prevent-symbolic");
    lock.set_halign(gtk::Align::Start);
    privacy.append(&lock);
    privacy.append(&label("Made here. Kept here.", "card-title"));
    let detail = label(
        "Your recordings stay on this device. You choose what to share.",
        "muted",
    );
    detail.set_wrap(true);
    detail.set_max_width_chars(23);
    privacy.append(&detail);
    sidebar.append(&privacy);
    let help_button = gtk::Button::with_label(&t("Keyboard shortcuts"));
    help_button.add_css_class("flat");
    help_button.add_css_class("help-button");
    sidebar.append(&help_button);
    root.append(&sidebar);

    let main = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main.set_hexpand(true);
    root.append(&main);
    let banner = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideDown)
        .build();
    let banner_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    banner_row.add_css_class("notice");
    banner_row.append(&gtk::Image::from_icon_name("dialog-information-symbolic"));
    let banner_text = label("", "notice-text");
    banner_text.set_wrap(true);
    banner_text.set_hexpand(true);
    banner_row.append(&banner_text);
    let dismiss = icon_button("window-close-symbolic", "Dismiss message");
    dismiss.add_css_class("flat");
    let hiding = banner.clone();
    dismiss.connect_clicked(move |_| hiding.set_reveal_child(false));
    banner_row.append(&dismiss);
    banner.set_child(Some(&banner_row));
    main.append(&banner);
    let pages = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .transition_duration(140)
        .hexpand(true)
        .vexpand(true)
        .build();
    main.append(&pages);
    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    let workspace = gtk::Box::new(gtk::Orientation::Vertical, 0);
    workspace.add_css_class("workspace");
    scroller.set_child(Some(&workspace));
    let record_page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    scroller.set_vexpand(true);
    record_page.append(&scroller);
    pages.add_named(&record_page, Some("record"));
    let eyebrow = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let path = label("YOUR WORKSPACE  /  NEW RECORDING", "eyebrow");
    path.set_hexpand(true);
    eyebrow.append(&path);
    let local = label("●  On this device", "local-badge");
    eyebrow.append(&local);
    workspace.append(&eyebrow);
    workspace.append(&label("Show it. Simply.", "page-title"));
    workspace.append(&label(
        "Turn a moment on your screen into something worth sharing.",
        "page-subtitle",
    ));

    let overlay = gtk::Overlay::new();
    overlay.add_css_class("stage");
    let preview = gtk::Picture::builder()
        .can_shrink(true)
        .content_fit(gtk::ContentFit::Contain)
        .build();
    let ratio = gtk::AspectFrame::builder()
        .ratio(16.0 / 9.0)
        .obey_child(false)
        .child(&preview)
        .build();
    ratio.set_height_request(250);
    overlay.set_child(Some(&ratio));
    let placeholder = gtk::Box::new(gtk::Orientation::Vertical, 15);
    placeholder.set_halign(gtk::Align::Center);
    placeholder.set_valign(gtk::Align::Center);
    let display = gtk::Image::from_icon_name("video-display-symbolic");
    display.set_pixel_size(58);
    display.add_css_class("stage-icon");
    placeholder.append(&display);
    placeholder.append(&label("Your story starts here", "stage-title"));
    placeholder.append(&label(
        "Choose a screen or window to see a live preview.",
        "stage-subtitle",
    ));
    overlay.add_overlay(&placeholder);
    let stage_tag = label("LIVE PREVIEW", "stage-tag");
    stage_tag.set_halign(gtk::Align::Start);
    stage_tag.set_valign(gtk::Align::Start);
    overlay.add_overlay(&stage_tag);
    let countdown_number = gtk::Label::new(None);
    countdown_number.add_css_class("countdown-number");
    countdown_number.set_hexpand(true);
    countdown_number.set_vexpand(true);
    countdown_number.set_visible(false);
    overlay.add_overlay(&countdown_number);
    workspace.append(&overlay);
    let preview_footer = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    preview_footer.add_css_class("preview-footer");
    let status = label("●  Nothing is recording", "status");
    status.set_hexpand(true);
    preview_footer.append(&status);
    let timer = label("00:00:00", "timer");
    preview_footer.append(&timer);
    workspace.append(&preview_footer);

    workspace.append(&label("WHAT WOULD YOU LIKE TO CAPTURE?", "section-heading"));
    let cards = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    cards.set_homogeneous(true);
    let screen = gtk::ToggleButton::new();
    screen.set_child(Some(&card(
        "video-display-symbolic",
        "A screen",
        "The whole picture",
    )));
    screen.set_active(true);
    screen.add_css_class("source-card");
    let one_window = gtk::ToggleButton::new();
    one_window.set_child(Some(&card(
        "focus-windows-symbolic",
        "A window",
        "Just one application",
    )));
    one_window.set_group(Some(&screen));
    one_window.add_css_class("source-card");
    cards.append(&screen);
    cards.append(&one_window);
    workspace.append(&cards);
    let source_name = label(
        "You choose exactly what can be seen in the system picker.",
        "source-description",
    );
    source_name.set_wrap(true);
    workspace.append(&source_name);
    let sound = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    sound.add_css_class("sound-row");
    sound.append(&gtk::Image::from_icon_name(
        "microphone-sensitivity-muted-symbolic",
    ));
    let sound_copy = gtk::Box::new(gtk::Orientation::Vertical, 4);
    sound_copy.set_hexpand(true);
    sound_copy.append(&label("Add your voice", "card-title"));
    let microphone_text = label("Microphone off. A quiet recording, by default.", "muted");
    sound_copy.append(&microphone_text);
    sound.append(&sound_copy);
    let microphone = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .tooltip_text(t("Record the microphone for this recording"))
        .build();
    microphone.update_property(&[gtk::accessible::Property::Label(&t("Record microphone"))]);
    sound.append(&microphone);
    workspace.append(&sound);
    let settings = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    settings.add_css_class("settings-row");
    let quality = gtk::DropDown::from_strings(&[
        &t("1080p · up to 30 fps"),
        &t("720p · up to 30 fps"),
        &t("Original size · up to 30 fps"),
    ]);
    quality.set_tooltip_text(Some(&t(
        "Recording quality. Smaller sources keep their original size.",
    )));
    quality.update_property(&[gtk::accessible::Property::Label(&t("Recording quality"))]);
    let countdown = gtk::DropDown::from_strings(&[
        &t("3-second countdown"),
        &t("5-second countdown"),
        &t("No countdown"),
    ]);
    countdown.update_property(&[gtk::accessible::Property::Label(&t(
        "Countdown before recording",
    ))]);
    settings.append(&quality);
    settings.append(&countdown);
    let gap = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    gap.set_hexpand(true);
    settings.append(&gap);
    let cancel = gtk::Button::with_label(&t("Cancel"));
    cancel.set_visible(false);
    settings.append(&cancel);
    let pause = gtk::Button::with_label(&t("Pause"));
    pause.set_visible(false);
    settings.append(&pause);
    let record = gtk::Button::with_label(&t("Choose a screen"));
    record.add_css_class("primary");
    settings.append(&record);
    settings.set_margin_start(34);
    settings.set_margin_end(34);
    settings.set_margin_bottom(12);
    record_page.append(&settings);

    let library_page = gtk::Box::new(gtk::Orientation::Vertical, 16);
    library_page.add_css_class("workspace");
    library_page.append(&label("YOUR WORKSPACE  /  LIBRARY", "eyebrow"));
    library_page.append(&label("Your recordings", "page-title"));
    library_page.append(&label(
        "A small collection of things you made clear.",
        "page-subtitle",
    ));
    let search = gtk::SearchEntry::builder()
        .placeholder_text(t("Find a recording…"))
        .build();
    library_page.append(&search);
    let library_list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .build();
    library_list.add_css_class("recording-list");
    let library_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&library_list)
        .build();
    library_page.append(&library_scroll);
    pages.add_named(&library_page, Some("library"));
    let review = super::review::Review::build();
    pages.add_named(&review.root, Some("review"));
    pages.set_visible_child_name("record");
    View {
        window,
        pages,
        new_button,
        library_button,
        help_button,
        library_count,
        screen,
        one_window,
        microphone,
        microphone_text,
        quality,
        countdown,
        preview,
        placeholder,
        countdown_number,
        timer,
        status,
        source_name,
        record,
        pause,
        cancel,
        banner,
        banner_text,
        library_list,
        search,
        review,
    }
}
