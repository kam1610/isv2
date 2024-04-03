use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib;
use gtk::glib::clone;
use gtk::SingleSelection;
use gtk::subclass::prelude::*;
use gtk::prelude::*;
use glib::subclass::Signal;

use once_cell::sync::Lazy;

use crate::scenario_node;
use crate::isv2_button::Isv2Button;
use crate::scenario_node_object::ScenarioNodeObject;
use crate::operation_history::OperationHistory;
use crate::tree_util::tree_manipulate;

// ScenarioNodeButtonBox ///////////////////////////////////
#[derive(Debug)]
pub struct ScenarioNodeButtonBox {
    pub(super) buttons : RefCell<Vec<Rc<AddNodeButton>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ScenarioNodeButtonBox {
    const NAME: &'static str = "ScenarioNodeButtonBox";
    type Type = super::ScenarioNodeButtonBox;
    type ParentType = gtk::Box;
}

impl ObjectImpl for ScenarioNodeButtonBox {
    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder("sno-selected")
                .param_types([SingleSelection::static_type()])
                .build(),
                 Signal::builder("unset-sno")
                .param_types([ScenarioNodeObject::static_type()])
                .build()
            ]
        });
        SIGNALS.as_ref()
    }
}

impl WidgetImpl for ScenarioNodeButtonBox {}

impl BoxImpl for ScenarioNodeButtonBox {}

impl Default for ScenarioNodeButtonBox{
    fn default() -> Self{
        Self{
            buttons : RefCell::new(Vec::new())
        }
    }
}

// AddNodeButton ///////////////////////////////////////////

#[derive(Debug)]
pub struct AddNodeButton { // parts of AddNodePopButton
    pub button       : Isv2Button,
    pub button_state : Cell<bool>,
    pub node_type    : scenario_node::Item,
}

impl AddNodeButton {
    // build //////////////////////////////////////////////
    pub fn build(label           : &str,
                 single_selection: SingleSelection,
                 history         : Rc<OperationHistory>,
                 node_type       : scenario_node::Item) -> Rc<Self>{
        let add_node_button = AddNodeButton{
            button: Isv2Button::with_label_selection_history("",
                                                             single_selection.clone(),
                                                             history.clone()),
            button_state : Cell::new(true),
            node_type,
        };
        let add_node_button = Rc::new(add_node_button);

        add_node_button.button.set_label(label);
        add_node_button.button.connect_clicked(clone!(@strong add_node_button=> move |btn| {
                let act_arg = match &add_node_button.node_type {
                    scenario_node::Item::Group    => tree_manipulate::ActTreeNodeAddCmd::Group,
                    scenario_node::Item::Scene(_) => tree_manipulate::ActTreeNodeAddCmd::Scene,
                    scenario_node::Item::Page(_)  => tree_manipulate::ActTreeNodeAddCmd::Page,
                    scenario_node::Item::Mat(_)   => tree_manipulate::ActTreeNodeAddCmd::Mat,
                    scenario_node::Item::Ovimg(_) => tree_manipulate::ActTreeNodeAddCmd::Ovimg,
                    scenario_node::Item::Pmat(_)  => tree_manipulate::ActTreeNodeAddCmd::Pmat,
                };
                btn.activate_action( &("win.".to_string() +
                                       tree_manipulate::ACT_TREE_NODE_ADD),
                                        Some( &(act_arg as i32).to_variant() ) ).expect("(AddNodeButton) invalid action");
            }));

        add_node_button
    }
    // set_state ///////////////////////////////////////////
    pub fn set_state(&self, s: bool){
        self.button_state.set(s);
        if s {
            self.button.first_child().unwrap().remove_css_class("label_ref_gray_out");
        } else {
            self.button.first_child().unwrap().add_css_class("label_ref_gray_out");
        }
    }
}
