//! What a document holds that a copy might not carry, and what the copy holds.
//!
//! ADR 0039 §4: **the original is inventoried before conversion and the copy
//! after, and the difference is reported by name.** Found from the documents,
//! never from the engine's log — a log says what the engine chose to mention,
//! and a document says what is there.
//!
//! | | |
//! |---|---|
//! | [`original`] | The original, read by format: [`word`], [`excel`], [`powerpoint`] |
//! | [`theme`], [`linked`] | What the three formats share: theme fonts and relationships |
//! | [`copy`] | The PDF, read for the fonts it contains |
//! | [`difference`] | The two inventories, compared into a [`crate::Carried`] |
//!
//! # Set on text, not merely declared
//!
//! A font counts when text is set in it. A document's font table and its theme
//! name fonts nothing uses, and counting those would report losses that are not
//! there — which is the same lie as not reporting one that is, told the other
//! way. So each format's file follows a run of text to the font it is shown
//! in, as far as the format says, and each says where it stops following.

pub mod copy;
pub mod difference;
pub mod excel;
pub mod linked;
pub mod original;
pub mod powerpoint;
pub mod theme;
pub mod word;

pub use copy::Copy;
pub use difference::carried;
pub use original::Original;
