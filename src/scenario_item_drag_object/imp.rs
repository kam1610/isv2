use std::cell::RefCell;
use std::rc::Rc;

use gtk::ListItem;
use gtk::gio;
use gtk::glib;
use gtk::subclass::prelude::*;

use crate::operation_history::OperationHistory;

// Object holding the state
pub struct ScenarioItemDragObject {
    pub(super) root_store: RefCell<Option<gio::ListStore>>,
    pub(super) history   : RefCell<Option<Rc<OperationHistory>>>,
    pub(super) list_item : RefCell<Option<ListItem>>
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ScenarioItemDragObject {
    const NAME: &'static str = "ScenarioItemDragObject";
    type Type = super::ScenarioItemDragObject;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for ScenarioItemDragObject {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

// Trait shared by all widgets
impl WidgetImpl for ScenarioItemDragObject {}

// Trait shared by all buttons
impl ButtonImpl for ScenarioItemDragObject {}

impl Default for ScenarioItemDragObject {
    fn default() -> Self{
        ScenarioItemDragObject{
            root_store: RefCell::new(None),
            history   : RefCell::new(None),
            list_item : RefCell::new(None),
        }
    }
}
