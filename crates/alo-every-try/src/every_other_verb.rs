//! **A request for every verb the three demo roads do not ask for**, so that a
//! change to any verb's words is tried against the model (law 9).
//!
//! Each is one plain sentence a person could type, and each road is the
//! shortest one a machine at full access takes for it: the verb, then an answer
//! in words. Where a road needs to know what is on the screen first, it reads
//! the window first, as the instructions tell a model to.
//!
//! # Where every name comes from
//!
//! As in [`crate::the_set()`]: verbs and arguments from the crates that declare
//! them, applications from the decided lists (`shipped.toml` and
//! `catalogue.toml`) found by role or by kind of work, and folders from
//! `alo-their-folders`. **The files named** (`notes.txt`, `report.pdf`, the
//! invoices) are this set's own: a request names them, and the scripted
//! answers say they are there.

use std::path::Path;
use std::time::SystemTime;

use alo_acting::{AKey, Acting};
use alo_adapters::fallback_verbs::{
    self, ACTIVATE_CONTROL, CHOOSE_OPTION, KEY, KIND, NAME, OPEN_ITEM, OPEN_MENU, OPTION,
    PRESS_KEY, READ_WINDOW, SCROLL_PAGE, SCROLL_TO, SELECT_ITEM, SET_VALUE, TOWARD,
    TYPE_WITHOUT_SAVING, VALUE,
};
use alo_adapters::scrolling::Toward;
use alo_adapters::{Pressable, Role as OnScreen, Seen, Shown, ShownWindow};
use alo_applications::verbs::{APPLICATION, CLOSE_APPLICATION, FOCUS_APPLICATION};
use alo_applications::{AnsweringTo, Application, Installed};
use alo_files::verbs::{
    ARCHIVE_FOLDER, FILE, FIND_IN_FOLDER, FOLDER, INTO, LIST_FOLDER, MOVE_FILE, NAMED, READ_FILE,
    RENAME_FILE,
};
use alo_protocol::Done;
use alo_software::role::Role;
use alo_software::shipped::Shipped;
use alo_software::{Catalogue, Category, INSTALL_APPLICATION, WHAT_CAN_BE_INSTALLED};
use alo_their_folders::Folder;

use crate::a_road::{ARequest, Answered, Step};
use crate::the_set::NotTheSet;

/// The file the reading, renaming and record requests name, in Documents.
pub const NOTES: &str = "notes.txt";

/// What [`NOTES`] says, when it is read.
pub const WHAT_THE_NOTES_SAY: &str = "Buy bread and call Anna on Friday.";

/// What [`NOTES`] is renamed to.
pub const RENAMED: &str = "old-notes.txt";

/// The file the moving request names, in Downloads.
pub const REPORT: &str = "report.pdf";

/// The words the search request looks for.
pub const INVOICE: &str = "invoice";

/// What the archiving request calls the archive.
pub const THE_ARCHIVE: &str = "pictures.zip";

/// The button the pressing request names, and the window it is in.
pub const RELOAD: &str = "Reload";

/// The slider the setting request names, in the image viewer.
pub const ZOOM: &str = "Zoom";

/// The number the setting request asks for.
pub const ZOOMED: i64 = 200;

/// The title of the image viewer's window the setting request reads.
pub const THE_PICTURE: &str = "holiday.jpg";

/// The option the choosing request names, in the document viewer's
/// [`ZOOM`] drop-down.
pub const FIT_WIDTH: &str = "Fit Width";

/// What the [`ZOOM`] drop-down shows before anything is chosen.
pub const AT_FULL_SIZE: &str = "100%";

/// The title of the browser's window the pressing request reads.
pub const THE_BROWSERS_WINDOW: &str = "Mozilla Firefox";

/// The heading the scrolling request names, below what the window shows.
pub const FURTHER_DOWN: &str = "Downloads";

/// The page the scrolling requests read.
pub const THE_PAGE: &str = "alo OS";

/// The heading on the top of [`THE_PAGE`], which is on the screen.
pub const ON_ITS_TOP: &str = "Getting started";
/// The field the typing-without-saving request names.
pub const SEARCH: &str = "Search";

/// What the typing-without-saving request types.
pub const SEARCHED_FOR: &str = "alo OS";

/// The kind of work the catalogue request asks about, by a word with one
/// meaning in the catalogue (`catalogue.toml`): painting is Painting, where
/// *editing photographs* could fairly be Pictures or Photographs.
const LOOKED_UNDER: Category = Category::Painting;

/// The application the installing request names, as a person says it. Which
/// identifier that is comes from the catalogue, matched the way the machine
/// matches a name (`alo_applications::Installed::answering_to`).
pub const TO_INSTALL: &str = "GIMP";

/// A folder's path under `home`.
fn in_their(home: &Path, folder: Folder) -> std::path::PathBuf {
    home.join(folder.xdg_name())
}

/// A path as a road expects it.
fn written(path: &Path) -> String {
    path.display().to_string()
}

/// The browser's window read again — a look a scrolling road may take aside,
/// after the scroll, as the model is told to.
fn the_window_read_again(window: &Shown, browser: &str) -> Step {
    Step::of(
        READ_WINDOW,
        Answered::With(Done::of_a_window_read(window, &[])),
    )
    .expecting(fallback_verbs::APPLICATION, browser.to_owned())
}

/// A page down in the browser, as often as the model asks — each one a
/// change, answered as the level answers it.
fn a_page_down(browser: &str) -> Step {
    Step::of(
        SCROLL_PAGE,
        Answered::With(Done::of_a_scroll(browser, Toward::Down.named(), true, true)),
    )
    .expecting(fallback_verbs::APPLICATION, browser.to_owned())
    .expecting(TOWARD, Toward::Down.named())
    .as_often_as_asked()
}

/// One thing in a folder, as the protocol carries it.
fn a_file(name: &str, bytes: u64) -> Result<alo_protocol::Thing, NotTheSet> {
    serde_json::from_value(serde_json::json!({ "name": name, "kind": "file", "bytes": bytes }))
        .map_err(|why| NotTheSet::because(format!("a file in a listing: {why}")))
}

/// The one application in the catalogue that `said` names, matched as the
/// machine matches a name a person or a model says.
fn the_one_called<'c>(
    catalogue: &'c Catalogue,
    said: &str,
) -> Result<&'c alo_software::Listed, NotTheSet> {
    let listed = Installed::holding(
        catalogue
            .every()
            .iter()
            .map(|one| one.application().clone()),
    );
    let AnsweringTo::One(application) = listed.answering_to(said) else {
        return Err(NotTheSet::because(format!(
            "the catalogue has no one application called {said}"
        )));
    };
    catalogue
        .every()
        .iter()
        .find(|one| one.application() == application)
        .ok_or_else(|| NotTheSet::because(format!("{said} is not in the catalogue")))
}

/// The requests, on a machine whose person's home is `home` and whose
/// applications are `shipped`.
///
/// # Errors
/// [`NotTheSet`], when the catalogue would not build or offers nothing for a
/// kind of work a request names.
#[expect(
    clippy::too_many_lines,
    reason = "one request after another, each read on its own; splitting them would only scatter the list"
)]
pub fn every_other_verb(home: &Path, shipped: &Shipped) -> Result<Vec<ARequest>, NotTheSet> {
    let documents = in_their(home, Folder::Documents);
    let downloads = in_their(home, Folder::Downloads);
    let pictures = in_their(home, Folder::Pictures);
    let notes = documents.join(NOTES);
    let report = downloads.join(REPORT);
    let browser = shipped
        .the(Role::WebBrowser)
        .application()
        .identifier()
        .to_owned();
    let files = shipped
        .the(Role::FileManager)
        .application()
        .identifier()
        .to_owned();
    let catalogue = Catalogue::decided().map_err(|why| NotTheSet::because(format!("{why:?}")))?;
    let to_install = the_one_called(&catalogue, TO_INSTALL)?;
    let documents_named = Folder::Documents.xdg_name();
    let downloads_named = Folder::Downloads.xdg_name();
    let pictures_named = Folder::Pictures.xdg_name();

    let listing = ARequest::asking(
        "list a folder",
        format!("What is in my {downloads_named} folder?"),
        vec![
            Step::of(
                LIST_FOLDER,
                Answered::With(Done::Listed {
                    things: vec![a_file(REPORT, 48_213)?],
                    could_not_be_named: 0,
                    cut_short: false,
                }),
            )
            .expecting(FOLDER, written(&downloads)),
        ],
    );
    let reading = ARequest::asking(
        "read a file",
        format!("What does the file {NOTES} in my {documents_named} say?"),
        vec![
            Step::of(
                READ_FILE,
                Answered::With(Done::Read {
                    text: WHAT_THE_NOTES_SAY.to_owned(),
                }),
            )
            .expecting(FILE, written(&notes)),
        ],
    );
    let finding = ARequest::asking(
        "find files by name",
        format!("Find the files in my {documents_named} whose names have {INVOICE} in them."),
        vec![
            Step::of(
                FIND_IN_FOLDER,
                Answered::With(Done::Found {
                    files: vec![
                        written(&documents.join(format!("{INVOICE}-march.pdf"))),
                        written(&documents.join(format!("{INVOICE}-april.pdf"))),
                    ],
                    could_not_be_named: 0,
                    cut_short: false,
                }),
            )
            .expecting(FOLDER, written(&documents))
            .expecting(NAMED, INVOICE),
        ],
    );
    let renaming = ARequest::asking(
        "rename a file",
        format!("Rename {NOTES} in my {documents_named} to {RENAMED}."),
        vec![
            Step::of(
                RENAME_FILE,
                Answered::With(Done::Renamed {
                    now_at: Some(written(&documents.join(RENAMED))),
                }),
            )
            .expecting(FILE, written(&notes))
            .expecting(alo_files::verbs::NAME, RENAMED),
        ],
    );
    let moving = ARequest::asking(
        "move a file",
        format!("Move {REPORT} from my {downloads_named} into my {documents_named}."),
        vec![
            Step::of(
                MOVE_FILE,
                Answered::With(Done::Moved {
                    now_at: Some(written(&documents.join(REPORT))),
                }),
            )
            .expecting(FILE, written(&report))
            .expecting(INTO, written(&documents)),
        ],
    );
    let archiving = ARequest::asking(
        "archive a folder",
        format!(
            "Make an archive of my {pictures_named} folder called {THE_ARCHIVE}, and put it in my \
             {documents_named}."
        ),
        vec![
            Step::of(
                ARCHIVE_FOLDER,
                Answered::With(Done::Archived {
                    at: Some(written(&documents.join(THE_ARCHIVE))),
                    things: 12,
                    left_out: 0,
                    bytes: 3_482_115,
                }),
            )
            .expecting(FOLDER, written(&pictures))
            .expecting(INTO, written(&documents))
            .expecting(alo_files::verbs::NAME, THE_ARCHIVE),
        ],
    );
    let looking = ARequest::asking(
        "see what can be installed",
        "What could I install for painting?",
        vec![
            Step::of(
                WHAT_CAN_BE_INSTALLED,
                Answered::TheCatalogue(catalogue.clone()),
            )
            .expecting(
                alo_software::words::KIND_OF_WORK,
                LOOKED_UNDER.as_an_option(),
            ),
        ],
    );
    let installing = ARequest::asking(
        "install an application",
        format!("Install {TO_INSTALL}."),
        vec![
            // **A look**: the catalogue is where the identifier is, and a
            // model that already knows it may go straight on.
            Step::of(
                WHAT_CAN_BE_INSTALLED,
                Answered::TheCatalogue(catalogue.clone()),
            )
            .a_look(),
            Step::of(
                INSTALL_APPLICATION,
                Answered::With(Done::Installed {
                    application: to_install.application().identifier().to_owned(),
                    sandboxed: true,
                }),
            )
            .expecting(
                alo_software::words::APPLICATION,
                to_install.application().identifier(),
            )
            .expecting(alo_software::words::SOURCE, to_install.source().as_str()),
        ],
    );
    let acted = |act: fn(Application) -> Acting| -> Result<Done, NotTheSet> {
        let which = Application::identified(&browser)
            .map_err(|why| NotTheSet::because(format!("{browser}: {why:?}")))?;
        Ok(Done::of_an_application_act(
            act(which).on_the_wire(),
            &browser,
        ))
    };
    let focusing = ARequest::asking(
        "bring an application to the front",
        "Bring Firefox to the front.",
        vec![
            Step::of(FOCUS_APPLICATION, Answered::With(acted(Acting::Focus)?))
                .expecting(APPLICATION, browser.as_str()),
        ],
    );
    let closing = ARequest::asking(
        "close an application",
        "Close Firefox.",
        vec![
            Step::of(CLOSE_APPLICATION, Answered::With(acted(Acting::Close)?))
                .expecting(APPLICATION, browser.as_str()),
        ],
    );
    let recalled = alo_record::what_touched(&alo_record::Record::default(), &written(&notes));
    let asking_the_record = ARequest::asking(
        "ask the record about a file",
        format!("What has alo done to {NOTES} in my {documents_named}?"),
        vec![
            Step::of(
                alo_record::what_touched_a_file::THE_VERB,
                Answered::With(Done::of_a_record_read(&recalled, SystemTime::now())),
            )
            .expecting(alo_record::what_touched_a_file::THE_FILE, written(&notes)),
        ],
    );
    let the_window = Shown::of(
        browser.clone(),
        vec![ShownWindow::titled(
            THE_BROWSERS_WINDOW.to_owned(),
            vec![Seen::of(OnScreen::Button, RELOAD.to_owned(), None, 1)],
        )],
        false,
    );
    let pressing = ARequest::asking(
        "press a button in an application",
        format!("Press the {RELOAD} button in Firefox."),
        vec![
            // **A look**: the person named the button, so a model that presses
            // it straight away has what the machine needs.
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_window, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .a_look(),
            Step::of(
                ACTIVATE_CONTROL,
                Answered::With(Done::of_a_press(
                    &browser,
                    Pressable::Button.named(),
                    RELOAD,
                    false,
                )),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .expecting(KIND, Pressable::Button.named())
            .expecting(NAME, RELOAD),
        ],
    )
    // **Reading the window again after acting is aside of the road**: the
    // model checking what its act did is a read the machine answers.
    .or_looking_at(the_window_read_again(&the_window, &browser));
    // **Scrolling, 2026-11-17**: to a thing the person names, and by a page.
    // Each may read the window first; neither needs to, since the request says
    // what to scroll to. **The window read is a page**, its top on the screen,
    // as a long web page reads.
    let the_page = Shown::of(
        browser.clone(),
        vec![ShownWindow::titled(
            THE_BROWSERS_WINDOW.to_owned(),
            vec![
                Seen::of(OnScreen::Document, THE_PAGE.to_owned(), None, 1),
                Seen::of(OnScreen::Label, ON_ITS_TOP.to_owned(), None, 2),
            ],
        )],
        false,
    );
    // **The same page a page further down**, the heading now on the screen.
    let the_page_further_down = Shown::of(
        browser.clone(),
        vec![ShownWindow::titled(
            THE_BROWSERS_WINDOW.to_owned(),
            vec![
                Seen::of(OnScreen::Document, THE_PAGE.to_owned(), None, 1),
                Seen::of(OnScreen::Label, FURTHER_DOWN.to_owned(), None, 2),
            ],
        )],
        false,
    );
    let scrolling_to = ARequest::asking(
        "scroll to a named thing in an application",
        format!("In Firefox, scroll down until the heading {FURTHER_DOWN} is on the screen."),
        vec![
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_page, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .a_look(),
            Step::of(
                SCROLL_TO,
                Answered::With(Done::of_a_scroll(&browser, FURTHER_DOWN, false, true)),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .expecting(fallback_verbs::TO, FURTHER_DOWN),
        ],
    )
    // **Pages down, and then the heading on the screen, reach the same place**:
    // "scroll down until" is how a page at a time is asked for, so these are
    // other roads, counted apart, and the request's words are not changed to
    // steer the model off them. A page down is answered without what it now
    // shows, so the heading is reached either by reading the window, which
    // shows it, or by scrolling to it by name.
    .or_by_another_road(vec![
        Step::of(
            READ_WINDOW,
            Answered::With(Done::of_a_window_read(&the_page, &[])),
        )
        .expecting(fallback_verbs::APPLICATION, browser.as_str())
        .a_look(),
        a_page_down(&browser),
        the_window_read_again(&the_page_further_down, &browser),
    ])
    .or_by_another_road(vec![
        Step::of(
            READ_WINDOW,
            Answered::With(Done::of_a_window_read(&the_page, &[])),
        )
        .expecting(fallback_verbs::APPLICATION, browser.as_str())
        .a_look(),
        a_page_down(&browser),
        Step::of(
            SCROLL_TO,
            Answered::With(Done::of_a_scroll(&browser, FURTHER_DOWN, false, true)),
        )
        .expecting(fallback_verbs::APPLICATION, browser.as_str())
        .expecting(fallback_verbs::TO, FURTHER_DOWN),
    ])
    // **Reading the window again after is aside of the road**: the model is
    // told to, and the machine answers it.
    .or_looking_at(the_window_read_again(&the_page, &browser));
    let scrolling_a_page = ARequest::asking(
        "scroll a page in an application",
        "Scroll down one page in Firefox.",
        vec![
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_page, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .a_look(),
            Step::of(
                SCROLL_PAGE,
                Answered::With(Done::of_a_scroll(
                    &browser,
                    Toward::Down.named(),
                    true,
                    true,
                )),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .expecting(TOWARD, Toward::Down.named()),
        ],
    )
    .or_looking_at(the_window_read_again(&the_page, &browser));
    let viewer = shipped
        .the(Role::ImageViewer)
        .application()
        .identifier()
        .to_owned();
    let the_viewers_window = Shown::of(
        viewer.clone(),
        vec![ShownWindow::titled(
            THE_PICTURE.to_owned(),
            vec![Seen::of(OnScreen::Range, ZOOM.to_owned(), None, 1).valued(Some(100.0))],
        )],
        false,
    );
    let setting = ARequest::asking(
        "move a slider in an application",
        format!("In Loupe, set the {ZOOM} slider to {ZOOMED}."),
        vec![
            // **A look**: the person named the slider and the number.
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_viewers_window, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, viewer.as_str())
            .a_look(),
            Step::of(
                SET_VALUE,
                Answered::With(Done::of_a_setting(&viewer, ZOOM, ZOOMED, true, false)),
            )
            .expecting(fallback_verbs::APPLICATION, viewer.as_str())
            .expecting(NAME, ZOOM)
            .expecting(VALUE, ZOOMED.to_string()),
        ],
    )
    .or_looking_at(the_window_read_again(&the_viewers_window, &viewer));
    let papers = shipped
        .the(Role::DocumentViewer)
        .application()
        .identifier()
        .to_owned();
    let the_papers_window = Shown::of(
        papers.clone(),
        vec![ShownWindow::titled(
            REPORT.to_owned(),
            vec![Seen::of(
                OnScreen::DropDown,
                ZOOM.to_owned(),
                Some(AT_FULL_SIZE.to_owned()),
                1,
            )],
        )],
        false,
    );
    let choosing = ARequest::asking(
        "choose an option in a drop-down",
        format!("In Papers, set the {ZOOM} drop-down to {FIT_WIDTH}."),
        vec![
            // **A look**: the person named the drop-down and the option.
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_papers_window, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, papers.as_str())
            .a_look(),
            Step::of(
                CHOOSE_OPTION,
                Answered::With(Done::of_a_choice(
                    &papers,
                    ZOOM,
                    Some(FIT_WIDTH),
                    true,
                    false,
                )),
            )
            .expecting(fallback_verbs::APPLICATION, papers.as_str())
            .expecting(NAME, ZOOM)
            .expecting(OPTION, FIT_WIDTH),
        ],
    )
    .or_looking_at(the_window_read_again(&the_papers_window, &papers));
    let the_folders_window = Shown::of(
        files.clone(),
        vec![ShownWindow::titled(
            downloads_named.to_owned(),
            vec![Seen::of(OnScreen::Item, REPORT.to_owned(), None, 1)],
        )],
        false,
    );
    let selecting = ARequest::asking(
        "select a file in the file manager",
        format!("Select {REPORT} in Dolphin, but do not open it."),
        vec![
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_folders_window, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, files.as_str())
            .a_look(),
            Step::of(
                SELECT_ITEM,
                Answered::With(Done::of_a_choice(&files, REPORT, None, true, false)),
            )
            .expecting(fallback_verbs::APPLICATION, files.as_str())
            .expecting(NAME, REPORT),
        ],
    )
    .or_looking_at(the_window_read_again(&the_folders_window, &files));
    let opening = ARequest::asking(
        "open a file in the file manager",
        format!("Open {REPORT} in Dolphin."),
        vec![
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_folders_window, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, files.as_str())
            .a_look(),
            Step::of(
                OPEN_ITEM,
                Answered::With(Done::of_an_item_opened(&files, REPORT, false)),
            )
            .expecting(fallback_verbs::APPLICATION, files.as_str())
            .expecting(NAME, REPORT),
        ],
    )
    .or_looking_at(the_window_read_again(&the_folders_window, &files));
    // **A thing's own menu, 2026-11-17**: the person names the file, so reading
    // the window first is a look.
    let the_folder = Shown::of(
        files.clone(),
        vec![ShownWindow::titled(
            documents_named.to_owned(),
            vec![Seen::of(OnScreen::Part, NOTES.to_owned(), None, 1)],
        )],
        false,
    );
    let a_menu = ARequest::asking(
        "open a thing's right-click menu in an application",
        format!("In Dolphin, open the right-click menu of {NOTES}."),
        vec![
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_folder, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, files.as_str())
            .a_look(),
            Step::of(
                OPEN_MENU,
                Answered::With(Done::of_a_menu(&files, NOTES, false)),
            )
            .expecting(fallback_verbs::APPLICATION, files.as_str())
            .expecting(fallback_verbs::THING, NOTES),
        ],
    )
    // **Reading the window again after is aside of the road**: the model is
    // told to read the menu's items, and the machine answers it.
    .or_looking_at(
        Step::of(
            READ_WINDOW,
            Answered::With(Done::of_a_window_read(&the_folder, &[])),
        )
        .expecting(fallback_verbs::APPLICATION, files.as_str()),
    );
    // **A key, 2026-11-17**: the person names it, so reading first is a look.
    let a_key = ARequest::asking(
        "press a key in an application",
        "Press Enter in Firefox.",
        vec![
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_window, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .a_look(),
            Step::of(
                PRESS_KEY,
                Answered::With(Done::of_a_key(&browser, AKey::Enter.named())),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .expecting(KEY, AKey::Enter.named()),
        ],
    )
    // **Reading the window again after is aside of the road**: what a key did
    // is seen by reading, and the machine answers it.
    .or_looking_at(
        Step::of(
            READ_WINDOW,
            Answered::With(Done::of_a_window_read(&the_window, &[])),
        )
        .expecting(fallback_verbs::APPLICATION, browser.as_str()),
    );
    // **Typing that saves nothing, 2026-11-17**: a web page's search box,
    // where `type_text`'s `Ctrl+S` would save the page.
    let the_search = Shown::of(
        browser.clone(),
        vec![ShownWindow::titled(
            THE_BROWSERS_WINDOW.to_owned(),
            vec![Seen::of(OnScreen::TextField, SEARCH.to_owned(), None, 1)],
        )],
        false,
    );
    let typing_without_saving = ARequest::asking(
        "type into a search box without saving",
        format!("In Firefox, type {SEARCHED_FOR} into the field called {SEARCH}."),
        vec![
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_search, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .a_look(),
            Step::of(
                TYPE_WITHOUT_SAVING,
                Answered::With(Done::of_a_typing_without_saving(
                    &browser,
                    SEARCH,
                    SEARCHED_FOR.chars().count(),
                    false,
                    true,
                )),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .expecting(NAME, SEARCH)
            .expecting(fallback_verbs::TEXT, SEARCHED_FOR),
        ],
    )
    .or_looking_at(
        Step::of(
            READ_WINDOW,
            Answered::With(Done::of_a_window_read(&the_search, &[])),
        )
        .expecting(fallback_verbs::APPLICATION, browser.as_str()),
    );
    Ok(vec![
        listing,
        reading,
        finding,
        renaming,
        moving,
        archiving,
        looking,
        installing,
        focusing,
        closing,
        asking_the_record,
        pressing,
        scrolling_to,
        scrolling_a_page,
        setting,
        choosing,
        selecting,
        opening,
        a_menu,
        a_key,
        typing_without_saving,
    ])
}
