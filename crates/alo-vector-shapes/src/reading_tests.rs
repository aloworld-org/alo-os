//! That the design file's own export reads, and that everything this crate
//! cannot draw is refused by name rather than skipped.

use super::{AShape, NotAShape};

/// The design file's Send arrow, exactly as `download_assets` returned it for
/// `427:28090` on 2026-10-30.
const SEND: &str = r##"<svg preserveAspectRatio="none" overflow="visible" style="display: block;" width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
<g id="Send / icon">
<path id="Vector" d="M14 22.1664V5.83311M21 12.8331L14 5.83311L7 12.8331" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
</g>
</svg>
"##;

/// A path in an SVG of `viewBox="0 0 28 28"`, with these attributes.
fn a_path_with(attributes: &str) -> String {
    format!(r#"<svg viewBox="0 0 28 28"><path d="M7 7H21V21H7Z" {attributes}/></svg>"#)
}

#[test]
fn the_design_files_own_export_reads() {
    let shape = AShape::read(SEND);
    assert!(shape.is_ok(), "{shape:?}");
    let shape = shape.unwrap_or_else(|_| unreachable!());
    assert_eq!(shape.its_box(), (28.0, 28.0));
    assert_eq!(shape.how_many_parts(), 1);
    assert_eq!(shape.colours(), [[255, 255, 255]]);
}

#[test]
fn a_path_filled_and_stroked_is_two_parts() {
    let shape = AShape::read(&a_path_with(r##"fill="#102A43" stroke="black""##));
    assert_eq!(
        shape.map(|it| it.colours()),
        Ok(vec![[0x10, 0x2a, 0x43], [0, 0, 0]])
    );
}

#[test]
fn a_transform_is_refused_by_name() {
    let svg = r#"<svg viewBox="0 0 28 28"><g transform="rotate(45)"><path d="M0 0L9 9" stroke="black"/></g></svg>"#;
    assert_eq!(
        AShape::read(svg),
        Err(NotAShape::AnAttributeNotRead {
            element: "g".to_owned(),
            attribute: "transform".to_owned()
        })
    );
}

#[test]
fn an_element_this_crate_does_not_draw_is_refused() {
    let svg = r#"<svg viewBox="0 0 28 28"><circle cx="1" cy="1" r="1"/></svg>"#;
    assert_eq!(
        AShape::read(svg),
        Err(NotAShape::AnElementNotDrawn("circle".to_owned()))
    );
}

#[test]
fn an_attribute_this_crate_does_not_read_is_refused() {
    assert_eq!(
        AShape::read(&a_path_with(r#"stroke="black" opacity="0.5""#)),
        Err(NotAShape::AnAttributeNotRead {
            element: "path".to_owned(),
            attribute: "opacity".to_owned()
        })
    );
}

#[test]
fn a_view_box_not_from_nought_is_refused() {
    let svg = r#"<svg viewBox="2 0 28 28"><path d="M7 7H21" stroke="black"/></svg>"#;
    assert_eq!(
        AShape::read(svg),
        Err(NotAShape::AViewBoxThisCannotDraw("2 0 28 28".to_owned()))
    );
}

#[test]
fn an_svg_with_no_view_box_is_refused() {
    let svg = r#"<svg width="28"><path d="M7 7H21" stroke="black"/></svg>"#;
    assert_eq!(AShape::read(svg), Err(NotAShape::NotOneSvgWithAViewBox));
}

#[test]
fn text_between_tags_is_refused() {
    let svg = r#"<svg viewBox="0 0 28 28">hello<path d="M7 7H21" stroke="black"/></svg>"#;
    assert_eq!(AShape::read(svg), Err(NotAShape::TextOutsideATag));
}

#[test]
fn an_element_left_open_is_refused() {
    let svg = r#"<svg viewBox="0 0 28 28"><g><path d="M7 7H21" stroke="black"/></g>"#;
    assert_eq!(
        AShape::read(svg),
        Err(NotAShape::AnElementNeverClosed("svg".to_owned()))
    );
}

#[test]
fn a_path_neither_filled_nor_stroked_is_refused() {
    assert_eq!(
        AShape::read(&a_path_with(r#"fill="none""#)),
        Err(NotAShape::APathNotPainted)
    );
}

#[test]
fn a_colour_this_crate_does_not_read_is_refused() {
    assert_eq!(
        AShape::read(&a_path_with(r#"fill="url(#gradient)""#)),
        Err(NotAShape::NotAColour("url(#gradient)".to_owned()))
    );
}

#[test]
fn path_data_that_is_not_path_data_is_refused() {
    let svg = r#"<svg viewBox="0 0 28 28"><path d="hello" stroke="black"/></svg>"#;
    assert_eq!(
        AShape::read(svg),
        Err(NotAShape::NotPathData("hello".to_owned()))
    );
}

#[test]
fn path_data_that_draws_nothing_is_refused() {
    // A move and nothing else: valid letters, and no pixel covered.
    let svg = r#"<svg viewBox="0 0 28 28"><path d="M7 7" fill="black"/></svg>"#;
    assert_eq!(
        AShape::read(svg),
        Err(NotAShape::NotPathData("M7 7".to_owned()))
    );
}

#[test]
fn an_svg_that_draws_nothing_is_refused() {
    let svg = r#"<svg viewBox="0 0 28 28"><g id="empty"></g></svg>"#;
    assert_eq!(AShape::read(svg), Err(NotAShape::NothingDrawn));
}
