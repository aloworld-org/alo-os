//! Derive credential carriers from fields, transitively, starting at TypedPassword.
use std::collections::{BTreeMap, BTreeSet};

/// Find every struct or enum containing the password holder, at any depth.
pub fn derived(files: &[(String, Vec<(usize, String)>)]) -> BTreeSet<String> {
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    for (_, lines) in files {
        let mut within = None;
        for (_, line) in lines {
            let trimmed = line.trim();
            if let Some(rest) = [
                "pub struct ",
                "pub(crate) struct ",
                "struct ",
                "pub enum ",
                "pub(crate) enum ",
                "enum ",
            ]
            .iter()
            .find_map(|prefix| trimmed.strip_prefix(prefix))
            {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                fields.entry(name.clone()).or_default().push_str(rest);
                within = (!trimmed.ends_with(';') && !trimmed.ends_with('}')).then_some(name);
            } else if line.starts_with('}') {
                within = None;
            } else if let Some(name) = &within {
                fields
                    .entry(name.clone())
                    .or_default()
                    .push_str(&format!(" {line}"));
            }
        }
    }
    assert!(fields.contains_key("TypedPassword"));
    let mut holders = BTreeSet::from(["TypedPassword".to_owned()]);
    loop {
        let before = holders.len();
        for (name, body) in &fields {
            if body
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|word| holders.contains(word))
            {
                holders.insert(name.clone());
            }
        }
        if holders.len() == before {
            return holders;
        }
    }
}

/// A newly added wrapper must be checked without editing a name list.
#[test]
fn newly_wrapped_passwords_are_discovered_without_a_list_edit() {
    let code = "struct TypedPassword {\n bytes: Vec<u8>,\n}\nstruct NewEntry {\n secret: TypedPassword,\n}\nenum NewResult {\n Waiting(Box<NewEntry>),\n}\nstruct Unrelated {\n value: usize,\n}";
    let files = vec![(
        "fixture.rs".into(),
        code.lines()
            .enumerate()
            .map(|(i, s)| (i, s.into()))
            .collect(),
    )];
    assert_eq!(
        derived(&files),
        BTreeSet::from([
            "TypedPassword".into(),
            "NewEntry".into(),
            "NewResult".into()
        ])
    );
}
