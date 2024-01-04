use std::cell::RefCell;
use std::rc::Rc;

use once_cell::sync::Lazy;

use glib::Object;
use glib::object::WeakRef;
use glib::subclass::Signal;
use gtk::Entry;
use gtk::SingleSelection;
use gtk::gio;
use gtk::glib;
use gtk::prelude::StaticType;
use gtk::subclass::prelude::*;
use gtk::Widget;

use crate::isv2_parameter::Isv2Parameter;
use crate::operation_history::OperationHistory;
use crate::scenario_node_object::ScenarioNodeObject;

// Object holding the state
// #[derive(Debug, Properties)]
// #[properties(wrapper=super::ScenarioNodeAttributeBox)]
pub struct ScenarioNodeAttributeBox {
    pub(super) sno              : RefCell<Option<ScenarioNodeObject>>,
    pub(super) _history         : RefCell<Option<Rc<OperationHistory>>>,
    pub(super) _root_store      : RefCell<Option<gio::ListStore>>,
    pub(super) contents_box     : RefCell<Option<gtk::Box>>,
    pub(super) mediator         : RefCell<WeakRef<Object>>,
    pub(super) parameter        : RefCell<Option<Isv2Parameter>>,
    pub(super) mat_posdim_entry : WeakRef<Entry>,
    pub(super) focus_tag        : RefCell<Option<Widget>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ScenarioNodeAttributeBox {
    const NAME: &'static str = "ScenarioNodeAttributeBox";
    type Type = super::ScenarioNodeAttributeBox;
    type ParentType = gtk::Box;
}

// Trait shared by all GObjects
impl ObjectImpl for ScenarioNodeAttributeBox {
    fn constructed(&self) {
        self.parent_constructed();
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder("sno-selected")
                .param_types([SingleSelection::static_type()])
                .build(),
                 Signal::builder("sno-move-resize")
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

// Trait shared by all widgets
impl WidgetImpl for ScenarioNodeAttributeBox {}

// Trait shared by all buttons
impl BoxImpl for ScenarioNodeAttributeBox {}

impl Default for ScenarioNodeAttributeBox {
    fn default() -> Self{
        ScenarioNodeAttributeBox{
            sno              : RefCell::new(None),
            _history         : RefCell::new(None),
            _root_store      : RefCell::new(None),
            contents_box     : RefCell::new(None),
            mediator         : RefCell::new(WeakRef::new()),
            parameter        : RefCell::new(None),
            mat_posdim_entry : WeakRef::new(),
            focus_tag        : RefCell::new(None),
        }
    }
}
