//! Independent SHM pixels and file lifetime for graphics integration fixtures.
use super::*;

impl Application {
    /// Commit a buffer matching the offscreen fixture's complete 33x32 output.
    pub fn attach_maximized(&self) {
        let qh = self.queue.handle();
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(&[0xff; 33 * 32 * 4]).unwrap();
        let pool = self.shm.create_pool(file.as_fd(), 33 * 32 * 4, &qh, ());
        let buffer = pool.create_buffer(0, 33, 32, 33 * 4, wl_shm::Format::Argb8888, &qh, ());
        self.surface.attach(Some(&buffer), 0, 0);
        self.surface.damage_buffer(0, 0, 33, 32);
        self.surface.commit();
        pool.destroy();
    }

    /// Commit an opaque 32x24 buffer for resize negotiation and input geometry.
    pub fn attach_resized(&self) {
        let qh = self.queue.handle();
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(&[0xff; 32 * 24 * 4]).unwrap();
        let pool = self.shm.create_pool(file.as_fd(), 32 * 24 * 4, &qh, ());
        let buffer = pool.create_buffer(0, 32, 24, 32 * 4, wl_shm::Format::Argb8888, &qh, ());
        self.surface.attach(Some(&buffer), 0, 0);
        self.surface.damage_buffer(0, 0, 32, 24);
        self.surface.commit();
        pool.destroy();
    }

    /// Attach a complete 16x16 ARGB buffer; retain the file for corruption tests.
    pub fn attach_pixels(&self, surface: &wl_surface::WlSurface, pixels: &[u8; 1024]) -> File {
        let qh = self.queue.handle();
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(pixels).unwrap();
        let pool = self.shm.create_pool(file.as_fd(), 1024, &qh, ());
        let buffer = pool.create_buffer(0, 16, 16, 64, wl_shm::Format::Argb8888, &qh, ());
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, 16, 16);
        surface.commit();
        pool.destroy();
        file
    }
}
