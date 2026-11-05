//! Direct, keyboard-accessible choices without a separate popup surface.

use std::{cell::Cell, rc::Rc};

use crate::words::t;
use gtk::prelude::*;

#[derive(Clone)]
pub struct Choices {
    pub widget: gtk::Box,
    selected: Rc<Cell<u32>>,
}

impl Choices {
    pub fn new(title: &str, options: &[(&str, &str)]) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        widget.set_accessible_role(gtk::AccessibleRole::Group);
        widget.update_property(&[gtk::accessible::Property::Label(&t(title))]);
        let label = gtk::Label::new(Some(&t(title)));
        label.add_css_class("setting-label");
        widget.append(&label);
        let selected = Rc::new(Cell::new(0));
        let mut first = None;
        for (index, (label, description)) in options.iter().enumerate() {
            let button = gtk::ToggleButton::with_mnemonic(&t(label));
            button.add_css_class("setting-choice");
            button.set_tooltip_text(Some(&t(description)));
            button.update_property(&[gtk::accessible::Property::Label(&t(description))]);
            button.set_group(first.as_ref());
            let selected = selected.clone();
            button.connect_toggled(move |button| {
                if button.is_active() {
                    selected.set(index as u32);
                }
            });
            if index == 0 {
                button.set_active(true);
                first = Some(button.clone());
            }
            widget.append(&button);
        }
        Self { widget, selected }
    }

    pub fn selected(&self) -> u32 {
        self.selected.get()
    }
}
