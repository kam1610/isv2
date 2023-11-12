use gtk::SingleSelection;
use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::operation_history::OperationHistory;

// Object holding the state
#[derive(Default)]
pub struct Isv2Button {
    pub(super) selection: RefCell<Rc<SingleSelection>>,
    pub(super) history  : RefCell<Rc<OperationHistory>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Isv2Button {
    const NAME: &'static str = "Isv2Button";
    type Type = super::Isv2Button;
    type ParentType = gtk::Button;
}

// Trait shared by all GObjects
impl ObjectImpl for Isv2Button {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

// Trait shared by all widgets
impl WidgetImpl for Isv2Button {}

// Trait shared by all buttons
impl ButtonImpl for Isv2Button {}
