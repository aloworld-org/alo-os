//! Independent SHM pixels and file lifetime for graphics integration fixtures.
use super::*;

impl Application {
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
