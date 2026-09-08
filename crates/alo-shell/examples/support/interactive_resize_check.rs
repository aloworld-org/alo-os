//! Full-frame GLES evidence for real XDG resize commit ordering.
use alo_shell::{Cursor, Server, render_scanout};
use smithay::backend::{
    input::ButtonState::{Pressed, Released},
    renderer::gles::GlesRenderer,
};

/// Drive one synchronized client stage and inspect every framebuffer pixel.
pub fn stage(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    stage: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let root = roots.first().ok_or("interactive resize root missing")?;
    let (origin, size) = match stage {
        13 => {
            server.place_window(root, (20, 20))?;
            server.pointer_motion(24.0, 25.0, 200)?;
            assert!(server.pointer_button(0x110, Pressed, 201)?);
            ((20, 20), (32, 24))
        }
        14 => {
            server.pointer_motion(16.0, 17.0, 202)?;
            assert!(server.pointer_motion(1_000_025.0, 17.0, 203).is_err());
            ((20, 20), (32, 24))
        }
        15 => {
            assert!(!server.pointer_button(0x110, Released, 204)?);
            ((36, 28), (16, 16))
        }
        16 => ((20, 20), (32, 24)),
        _ => return Err("unknown interactive resize stage".into()),
    };
    assert_eq!(
        alo_shell::window_buffer_origin(root),
        (f64::from(origin.0), f64::from(origin.1)).into()
    );
    let prepared = render_scanout(
        renderer,
        (80, 80).into(),
        &roots,
        &server.popup_surfaces(),
        &Cursor::Hidden,
    )?;
    for (index, pixel) in prepared
        .pixels()
        .pixels()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        let x = (index % 80) as i32;
        let y = (index / 80) as i32;
        let expected = if (origin.0..origin.0 + size.0).contains(&x)
            && (origin.1..origin.1 + size.1).contains(&y)
        {
            [255, 255, 255, 0]
        } else {
            [0; 4]
        };
        assert_eq!(
            *pixel, expected,
            "interactive resize stage {stage} pixel {index}"
        );
    }
    println!("Interactive resize stage {stage}: all 6400 GLES pixels passed");
    Ok(())
}
