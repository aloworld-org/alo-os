//! Full-frame painting of views derived from live client layout transactions.
use alo_appearance::Scheme;
use alo_shell::{RowOrder, WindowControlLayout, readback_xrgb};
use drm::buffer::DrmFourcc;
use smithay::{
    backend::renderer::{
        Bind, Color32F, Frame, Offscreen, Renderer,
        gles::{GlesRenderbuffer, GlesRenderer},
    },
    utils::{Rectangle, Transform},
};

/// Compare each live snapshot to independent glyph masks in both color schemes.
pub fn paint(
    renderer: &mut GlesRenderer,
    layout: &WindowControlLayout,
    restoring: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer: GlesRenderbuffer =
        renderer.create_buffer(DrmFourcc::Abgr8888, (120, 48).into())?;
    let mut target = renderer.bind(&mut buffer)?;
    for scheme in [Scheme::Light, Scheme::Dark] {
        let mut frame = renderer.render(&mut target, (120, 48).into(), Transform::Normal)?;
        frame.clear(
            Color32F::new(0.0, 0.0, 0.0, 1.0),
            &[Rectangle::from_size((120, 48).into())],
        )?;
        layout.paint(&mut frame, scheme)?;
        let _sync = frame.finish()?;
        let pixels = readback_xrgb(renderer, &target, RowOrder::TopToBottom)?;
        assert_eq!(pixels.size(), (120, 48));
        assert_eq!(pixels.pixels().len(), 120 * 48 * 4);
        let (rows, remainder) = pixels.pixels().as_chunks::<4>();
        assert!(remainder.is_empty());
        for (index, pixel) in rows.iter().enumerate() {
            let x = i32::try_from(index % 120)?;
            let y = i32::try_from(index / 120)?;
            let expected = crate::window_controls_pixels::expected(
                (x, y),
                (3, 4),
                scheme,
                [true; 3],
                restoring,
            );
            assert_eq!(*pixel, expected, "live control snapshot pixel {x},{y}");
            assert_eq!(
                layout.hit(f64::from(x) + 0.5, f64::from(y) + 0.5).is_some(),
                expected != [0; 4]
            );
        }
    }
    println!("Live control snapshot: two complete 5,760-pixel light/dark GLES frames passed");
    Ok(())
}
