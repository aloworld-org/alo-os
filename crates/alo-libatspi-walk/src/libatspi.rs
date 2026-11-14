//! **libatspi's C API, and the only `unsafe` in this package.**
//!
//! Twelve functions, opened by soname when the walk starts: nine of
//! libatspi's, `g_main_context_iteration`, `g_free` and `g_error_free` from
//! GLib, and `g_object_unref` from GObject. Each is wrapped once here in a
//! function that takes and returns ordinary Rust values, so `crate::walk` holds
//! no pointer and has no `unsafe` in it.
//!
//! # What makes each call sound
//!
//! - **The signatures are libatspi's own**, from at-spi2-core 2.60's
//!   `atspi/atspi-accessible.h` and `atspi/atspi-misc.h`, and GLib's. An
//!   `AtspiAccessible *` is only ever passed back to libatspi, never read.
//! - **Every object handed out is owned exactly once.** libatspi's getters
//!   return a new reference (`transfer full`); [`Thing`] holds it and gives it
//!   back with `g_object_unref` when dropped. A string is copied and then
//!   given back with `g_free`, and a `GError` is read and freed with
//!   `g_error_free`.
//! - **The libraries outlive every function taken from them**: [`Libatspi`]
//!   keeps them, and a [`Thing`] borrows the [`Libatspi`] it came from.
//! - **One thread.** libatspi is not thread-safe, and the walk never starts a
//!   thread.

#![allow(
    unsafe_code,
    reason = "calling a C library is unsafe by definition; this file is the only one \
              allowed it, and the crate root says why"
)]

use std::ffi::{CStr, c_char, c_int, c_void};
use std::ptr::{self, NonNull};

use libloading::Library;

/// `AtspiCache`'s `ATSPI_CACHE_DEFAULT`: parent, children, name, description,
/// states, role and interfaces (`atspi-constants.h`).
const CACHE_DEFAULT: c_int = 0x7f;

/// GLib's `GError`, laid out as GLib lays it out so that `message` is found.
#[repr(C)]
#[allow(
    dead_code,
    reason = "GLib builds it and this only reads it, so whether rustc calls it constructed \
              or its first two fields read depends on the compiler version; the layout is the point"
)]
struct GError {
    /// Which library the error is from.
    domain: u32,
    /// Its number in that library.
    code: c_int,
    /// What it says.
    message: *const c_char,
}

/// `gint f (void)`.
type Init = unsafe extern "C" fn() -> c_int;
/// `AtspiAccessible *atspi_get_desktop (gint)`.
type Desktop = unsafe extern "C" fn(c_int) -> *mut c_void;
/// `gint f (AtspiAccessible *, GError **)`.
type ANumber = unsafe extern "C" fn(*mut c_void, *mut *mut GError) -> c_int;
/// `AtspiAccessible *atspi_accessible_get_child_at_index (AtspiAccessible *, gint, GError **)`.
type ChildAt = unsafe extern "C" fn(*mut c_void, c_int, *mut *mut GError) -> *mut c_void;
/// `AtspiAccessible *f (AtspiAccessible *, GError **)`.
type AThing = unsafe extern "C" fn(*mut c_void, *mut *mut GError) -> *mut c_void;
/// `gchar *f (AtspiAccessible *, GError **)`.
type AString = unsafe extern "C" fn(*mut c_void, *mut *mut GError) -> *mut c_char;
/// `void atspi_accessible_set_cache_mask (AtspiAccessible *, AtspiCache)`.
type CacheMask = unsafe extern "C" fn(*mut c_void, c_int);
/// `gboolean g_main_context_iteration (GMainContext *, gboolean)`.
type Iteration = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
/// `void f (gpointer)`.
type Release = unsafe extern "C" fn(*mut c_void);

/// libatspi, opened and started.
pub(crate) struct Libatspi {
    /// `atspi_get_desktop`.
    desktop: Desktop,
    /// `atspi_accessible_get_child_count`.
    child_count: ANumber,
    /// `atspi_accessible_get_child_at_index`.
    child_at: ChildAt,
    /// `atspi_accessible_get_name`.
    name: AString,
    /// `atspi_accessible_get_role`.
    role: ANumber,
    /// `atspi_accessible_get_parent`.
    parent: AThing,
    /// `atspi_accessible_get_index_in_parent`.
    index_in_parent: ANumber,
    /// `atspi_accessible_get_toolkit_name`.
    toolkit_name: AString,
    /// `atspi_accessible_set_cache_mask`.
    cache_mask: CacheMask,
    /// `g_main_context_iteration`.
    iteration: Iteration,
    /// `g_free`.
    free: Release,
    /// `g_error_free`.
    error_free: Release,
    /// `g_object_unref`.
    unref: Release,
    /// The libraries every function above lives in, kept open as long as
    /// this is.
    _open: [Library; 3],
}

/// One thing in the tree, as libatspi holds it.
pub(crate) struct Thing<'a> {
    /// libatspi's object, one reference of which this owns.
    at: NonNull<c_void>,
    /// The library it came from.
    from: &'a Libatspi,
}

/// A library, opened by soname.
fn opened(soname: &str) -> Result<Library, String> {
    // SAFETY: opening runs the library's initialisers. These are libatspi,
    // GLib and GObject, which this process is about to use anyway, and whose
    // initialisers have no requirements of the caller.
    unsafe { Library::new(soname) }.map_err(|why| format!("{soname} could not be opened: {why}"))
}

/// One function out of `library`, by name, as type `T`.
fn taken<T: Copy>(library: &Library, name: &str) -> Result<T, String> {
    // SAFETY: every `T` this is called with is the C signature of `name` as
    // the library's own header declares it (this file's documentation), and the
    // library stays open for as long as the function is kept: both live in one
    // `Libatspi`.
    unsafe { library.get::<T>(name.as_bytes()) }
        .map(|symbol| *symbol)
        .map_err(|why| format!("{name} is not in the library: {why}"))
}

impl Libatspi {
    /// Open the three libraries, take the functions, and start libatspi.
    ///
    /// # Errors
    /// Which library or function was missing, or that `atspi_init` refused.
    pub(crate) fn started() -> Result<Self, String> {
        let atspi = opened("libatspi.so.0")?;
        let glib = opened("libglib-2.0.so.0")?;
        let gobject = opened("libgobject-2.0.so.0")?;
        let init: Init = taken(&atspi, "atspi_init")?;
        let this = Self {
            desktop: taken(&atspi, "atspi_get_desktop")?,
            child_count: taken(&atspi, "atspi_accessible_get_child_count")?,
            child_at: taken(&atspi, "atspi_accessible_get_child_at_index")?,
            name: taken(&atspi, "atspi_accessible_get_name")?,
            role: taken(&atspi, "atspi_accessible_get_role")?,
            parent: taken(&atspi, "atspi_accessible_get_parent")?,
            index_in_parent: taken(&atspi, "atspi_accessible_get_index_in_parent")?,
            toolkit_name: taken(&atspi, "atspi_accessible_get_toolkit_name")?,
            cache_mask: taken(&atspi, "atspi_accessible_set_cache_mask")?,
            iteration: taken(&glib, "g_main_context_iteration")?,
            free: taken(&glib, "g_free")?,
            error_free: taken(&glib, "g_error_free")?,
            unref: taken(&gobject, "g_object_unref")?,
            _open: [atspi, glib, gobject],
        };
        // SAFETY: `atspi_init` takes nothing; a second call answers 1 and does
        // nothing, so it cannot be called wrongly.
        let started = unsafe { init() };
        match started {
            0 | 1 => Ok(this),
            refused => Err(format!("atspi_init refused: {refused}")),
        }
    }

    /// The desktop: every application the registry has embedded.
    ///
    /// # Errors
    /// When libatspi has no desktop to give, which is no registry.
    pub(crate) fn desktop(&self) -> Result<Thing<'_>, String> {
        // SAFETY: `atspi_get_desktop (0)` takes a plain number and returns a
        // new reference or NULL.
        let at = unsafe { (self.desktop)(0) };
        self.owned(at, "the desktop")
    }

    /// Let GLib's default main loop take in whatever has arrived, once,
    /// without waiting — which is how libatspi's answer to `GetItems` arrives.
    /// Whether anything was.
    pub(crate) fn take_in_what_arrived(&self) -> bool {
        // SAFETY: NULL is GLib's own spelling of the default context, and
        // `FALSE` asks it not to block.
        let took = unsafe { (self.iteration)(ptr::null_mut(), 0) };
        took != 0
    }

    /// `at` as an owned [`Thing`], or what was not there.
    fn owned(&self, at: *mut c_void, what: &str) -> Result<Thing<'_>, String> {
        NonNull::new(at)
            .map(|at| Thing { at, from: self })
            .ok_or_else(|| format!("libatspi gave no {what}"))
    }

    /// What a `GError` said, freed; `Ok` where there was none.
    fn heard(&self, error: *mut GError) -> Result<(), String> {
        if error.is_null() {
            return Ok(());
        }
        // SAFETY: a non-NULL `GError *` from libatspi is a valid `GError`
        // whose `message` is a NUL-terminated string or NULL, and it is ours to
        // free, which is done once, after its message is copied.
        let said = unsafe {
            let message = (*error).message;
            let said = if message.is_null() {
                String::from("an error with no message")
            } else {
                CStr::from_ptr(message).to_string_lossy().into_owned()
            };
            (self.error_free)(error.cast());
            said
        };
        Err(said)
    }

    /// A string libatspi handed over, copied and freed.
    fn copied(&self, string: *mut c_char) -> Option<String> {
        if string.is_null() {
            return None;
        }
        // SAFETY: a non-NULL `gchar *` from a `transfer full` getter is a
        // NUL-terminated string this process owns; it is copied, then freed
        // once.
        unsafe {
            let copied = CStr::from_ptr(string).to_string_lossy().into_owned();
            (self.free)(string.cast());
            Some(copied)
        }
    }
}

impl Thing<'_> {
    /// A number from `asked`, or the error libatspi gave.
    fn number(&self, asked: ANumber) -> Result<i32, String> {
        let mut error: *mut GError = ptr::null_mut();
        // SAFETY: `self.at` is a live `AtspiAccessible *` this owns a reference
        // to, and `error` is a valid place for a `GError *`.
        let answer = unsafe { asked(self.at.as_ptr(), &raw mut error) };
        self.from.heard(error).map(|()| answer)
    }

    /// A string from `asked`, or the error libatspi gave.
    fn string(&self, asked: AString, what: &str) -> Result<String, String> {
        let mut error: *mut GError = ptr::null_mut();
        // SAFETY: as for `number`.
        let answer = unsafe { asked(self.at.as_ptr(), &raw mut error) };
        self.from.heard(error)?;
        self.from
            .copied(answer)
            .ok_or_else(|| format!("libatspi gave no {what}"))
    }

    /// How many children libatspi says this has.
    pub(crate) fn child_count(&self) -> Result<i32, String> {
        self.number(self.from.child_count)
    }

    /// Its role, as `AtspiRole` numbers it.
    pub(crate) fn role(&self) -> Result<i32, String> {
        self.number(self.from.role)
    }

    /// Where libatspi says it is among its parent's children.
    pub(crate) fn index_in_parent(&self) -> Result<i32, String> {
        self.number(self.from.index_in_parent)
    }

    /// Its name.
    pub(crate) fn name(&self) -> Result<String, String> {
        self.string(self.from.name, "name")
    }

    /// The toolkit its application says drew it.
    pub(crate) fn toolkit_name(&self) -> Result<String, String> {
        self.string(self.from.toolkit_name, "toolkit name")
    }

    /// One of its children, which libatspi may answer with nothing.
    pub(crate) fn child_at(&self, index: i32) -> Result<Thing<'_>, String> {
        let mut error: *mut GError = ptr::null_mut();
        // SAFETY: as for `number`; `index` is a plain number, and libatspi
        // returns a new reference or NULL.
        let child = unsafe { (self.from.child_at)(self.at.as_ptr(), index, &raw mut error) };
        self.from.heard(error)?;
        self.from.owned(child, "child")
    }

    /// What it hangs under, where libatspi says it hangs under anything.
    pub(crate) fn parent(&self) -> Result<Option<Thing<'_>>, String> {
        let mut error: *mut GError = ptr::null_mut();
        // SAFETY: as for `child_at`.
        let parent = unsafe { (self.from.parent)(self.at.as_ptr(), &raw mut error) };
        self.from.heard(error)?;
        Ok(self.from.owned(parent, "parent").ok())
    }

    /// Whether this and `other` are the one object libatspi holds — which is
    /// how libatspi says two references name the same thing, since it keeps
    /// one object per path.
    pub(crate) fn is(&self, other: &Thing<'_>) -> bool {
        self.at == other.at
    }

    /// Turn libatspi's cache on for this application, which must be an
    /// application's own root: from here on what `GetItems` said is read
    /// rather than asked again.
    pub(crate) fn cache_everything_it_can(&self) {
        // SAFETY: `self.at` is live; libatspi checks that it is an
        // application's root and logs, rather than misbehaves, if it is not.
        unsafe { (self.from.cache_mask)(self.at.as_ptr(), CACHE_DEFAULT) }
    }
}

impl Drop for Thing<'_> {
    fn drop(&mut self) {
        // SAFETY: this owns exactly one reference to `self.at`, and gives it
        // back exactly once.
        unsafe { (self.from.unref)(self.at.as_ptr()) }
    }
}
