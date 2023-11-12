use std::cell::{RefCell,Cell};
use std::rc::Rc;

use glib::{ParamSpec, Properties, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::scenario_node::ScenarioNode;

// Object holding the node
#[derive(Properties, Default)]
#[properties(wrapper_type = super::ScenarioNodeObject)]
pub struct ScenarioNodeObject {
    pub(super) node: RefCell<Rc<ScenarioNode>>,
    #[property(get, set)]
    pub seq: Cell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for ScenarioNodeObject {
    const NAME: &'static str = "MyGtkAppScenarioNodeObject";
    type Type = super::ScenarioNodeObject;
}

// Trait shared by all GObjects
impl ObjectImpl for ScenarioNodeObject {
    fn properties() -> &'static [ParamSpec] {
        Self::derived_properties()
    }
    fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
        self.derived_set_property(id, value, pspec)
    }
    fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
        self.derived_property(id, pspec)
    }
}


//// debug
// impl Drop for ScenarioNodeObject {
//     fn drop(&mut self) {
//         println!("> Dropping {}", self.node.borrow());
//     }
// }
