//! The image carries the approved picture that appearance actually selects.

#[test]
fn the_default_wallpaper_is_installed_readably_under_its_shipped_name()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = "docs/artwork/wallpapers/alo-quiet-horizon.png";
    let picture = std::fs::read(root.join(source))?;
    assert_eq!(picture.get(..8), Some(b"\x89PNG\r\n\x1a\n".as_slice()));
    assert_eq!(picture.get(12..16), Some(b"IHDR".as_slice()));
    assert!(
        picture.len() > 24,
        "a header alone is not an installed picture"
    );
    let recipe = std::fs::read_to_string(root.join("image/Containerfile"))?;
    let installed = format!(
        "/usr/share/alo/wallpapers/{}.png",
        alo_appearance::THE_WALLPAPER
    );
    let copy = format!("COPY --chmod=0644 {source} {installed}");
    assert_eq!(recipe.lines().filter(|line| *line == copy).count(), 1);
    let contract = std::fs::read_to_string(root.join("docs/contracts/shipped-wallpapers.md"))?;
    assert!(contract.contains(&installed));
    Ok(())
}
