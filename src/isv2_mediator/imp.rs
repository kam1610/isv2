use gtk::glib;
use gtk::glib::prelude::*;
use gtk::subclass::prelude::*;
use gtk::SingleSelection;
use glib::object::Object;

use glib::Properties;
use glib::subclass::Signal;

use std::cell::RefCell;

use once_cell::sync::Lazy;

use crate::scenario_node_object::ScenarioNodeObject;

// Object holding the state
#[derive(Debug, Properties)]
#[properties(wrapper_type = super::Isv2Mediator)]
pub struct Isv2Mediator {
    #[property(get, set)]
    pub(super) list_view          : RefCell<Object>,
    #[property(get, set)]
    pub(super) attr_box           : RefCell<Object>,
    #[property(get, set)]
    pub(super) preview_window     : RefCell<Object>,
    #[property(get, set)]
    pub(super) parameter          : RefCell<Object>,
    #[property(get, set)]
    pub(super) scenario_text_view : RefCell<Object>,
    #[property(get, set)]
    pub(super) node_add_box       : RefCell<Object>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Isv2Mediator {
    const NAME: &'static str = "Isv2Mediator";
    type Type = super::Isv2Mediator;
    type ParentType = glib::Object;
}

impl ObjectImpl for Isv2Mediator {
    // properties //////////////////////////////////////////
    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }
    fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        self.derived_set_property(id, value, pspec)
    }
    fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        self.derived_property(id, pspec)
    }
    // signals /////////////////////////////////////////////
    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder("sno-selected")
                .param_types([SingleSelection::static_type()])
                .build(),
                 Signal::builder("mat-attribute-changed")
                .param_types([ScenarioNodeObject::static_type()])
                .build(),
                 Signal::builder("sno-move-resize")
                .param_types([ScenarioNodeObject::static_type()])
                .build(),
                 Signal::builder("scene-attribute-changed")
                .param_types([ScenarioNodeObject::static_type()])
                .build(),
                 Signal::builder("unset-sno")
                .param_types([ScenarioNodeObject::static_type()])
                .build()
            ]
        });
        SIGNALS.as_ref()
    }

}

impl Default for Isv2Mediator{
    fn default() -> Self{
        Self{
            list_view          : RefCell::new(Object::with_type(glib::types::Type::OBJECT)),
            attr_box           : RefCell::new(Object::with_type(glib::types::Type::OBJECT)),
            preview_window     : RefCell::new(Object::with_type(glib::types::Type::OBJECT)),
            parameter          : RefCell::new(Object::with_type(glib::types::Type::OBJECT)),
            scenario_text_view : RefCell::new(Object::with_type(glib::types::Type::OBJECT)),
            node_add_box       : RefCell::new(Object::with_type(glib::types::Type::OBJECT)),
        }
    }
}
