use once_cell::sync::Lazy;

use glib::Object;
use glib::WeakRef;
use glib::subclass::Signal;
use gtk::SingleSelection;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

use crate::scenario_node_object::ScenarioNodeObject;

pub struct ScenarioTextView{
    pub(super) sno      : RefCell<Option<ScenarioNodeObject>>,
    pub(super) mediator : RefCell<WeakRef<Object>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ScenarioTextView {
    const NAME: &'static str = "ScenarioTextView";
    type Type = super::ScenarioTextView;
    type ParentType = gtk::TextView;
}

impl ObjectImpl for ScenarioTextView{
    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder("sno-selected")
                .param_types([SingleSelection::static_type()])
                .build(),
            ]
        });
        SIGNALS.as_ref()
    }
}

impl WidgetImpl for ScenarioTextView {}

impl TextViewImpl for ScenarioTextView {}

impl Default for ScenarioTextView {
    fn default() -> Self{
        Self{
            sno      : None.into(),
            mediator : RefCell::new(WeakRef::new()),
        }
    }
}
