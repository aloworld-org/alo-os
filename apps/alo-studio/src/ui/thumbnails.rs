//! Load library previews only when shown, without blocking recorder controls.

use gtk::{gio, glib, prelude::*};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(super) fn load_when_shown(picture: &gtk::Picture, path: PathBuf) {
    let pending = Rc::new(RefCell::new(None::<Arc<AtomicBool>>));
    let cancelled = pending.clone();
    picture.connect_unmap(move |_| {
        if let Some(task) = cancelled.borrow_mut().take() {
            task.store(true, Ordering::Relaxed);
        }
    });
    picture.connect_map(move |picture| {
        if picture.paintable().is_some() || pending.borrow().is_some() {
            return;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        *pending.borrow_mut() = Some(cancel.clone());
        let weak = picture.downgrade();
        let waiting = pending.clone();
        let token = cancel.clone();
        let path = path.clone();
        // The stream's async API can still decode PNG chunks on the UI context.
        // Run the complete decoder in GIO's worker pool; only owned pixels return.
        let worker = gio::spawn_blocking(move || decode(path, &cancel));
        glib::MainContext::default().spawn_local_with_priority(glib::Priority::LOW, async move {
            let decoded = worker.await;
            clear(&waiting, &token);
            if token.load(Ordering::Relaxed) {
                return;
            }
            if let (Some(picture), Ok(Some(image))) = (weak.upgrade(), decoded)
                && picture.is_mapped()
            {
                picture.set_paintable(Some(&gtk::gdk::MemoryTexture::new(
                    image.width,
                    image.height,
                    gtk::gdk::MemoryFormat::R8g8b8a8,
                    &glib::Bytes::from_owned(image.pixels),
                    image.width as usize * 4,
                )));
            }
        });
    });
}

fn clear(pending: &RefCell<Option<Arc<AtomicBool>>>, token: &Arc<AtomicBool>) {
    let mut pending = pending.borrow_mut();
    if pending
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(current, token))
    {
        pending.take();
    }
}

struct Thumbnail {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}

fn decode(path: PathBuf, cancelled: &AtomicBool) -> Option<Thumbnail> {
    if cancelled.load(Ordering::Relaxed) || !std::fs::metadata(&path).ok()?.is_file() {
        return None;
    }
    let image = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(path, 100, 60, true).ok()?;
    if cancelled.load(Ordering::Relaxed) {
        return None;
    }
    let (width, height) = (image.width(), image.height());
    let channels = usize::try_from(image.n_channels()).ok()?;
    let stride = usize::try_from(image.rowstride()).ok()?;
    if !(1..=100).contains(&width)
        || !(1..=60).contains(&height)
        || image.bits_per_sample() != 8
        || ![3, 4].contains(&channels)
    {
        return None;
    }
    let source = image.read_pixel_bytes();
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for row in 0..height as usize {
        let start = row.checked_mul(stride)?;
        for pixel in source
            .get(start..start + width as usize * channels)?
            .chunks_exact(channels)
        {
            pixels.extend_from_slice(&pixel[..3]);
            pixels.push(if channels == 4 { pixel[3] } else { 255 });
        }
    }
    Some(Thumbnail {
        width,
        height,
        pixels,
    })
}
