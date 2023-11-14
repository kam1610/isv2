mod imp;

use std::rc::Rc;

use gtk::gio;
use gtk::glib;
use gtk::SingleSelection;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use glib::closure_local;
use glib::object::Object;

use crate::operation_history::OperationHistory;
use crate::scenario_node::ScenarioNode;
use crate::scenario_node::Item;
use crate::scenario_node::{Scene, Page, Mat, Ovimg};
use crate::scenario_node;
use crate::scenario_node_object::ScenarioNodeObject;
use crate::scenario_node_button_box::imp::AddNodeButton;
use crate::sno_list::selection_to_sno;

glib::wrapper! {
    pub struct ScenarioNodeButtonBox(ObjectSubclass<imp::ScenarioNodeButtonBox>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ScenarioNodeButtonBox{
    //
    fn sno_selected(snbbox: ScenarioNodeButtonBox, s: SingleSelection){
        //let snbbox = w.downcast_ref::<ScenarioNodeButtonBox>().unwrap();
        let (sno, _) =
            if let Some((a,b)) = selection_to_sno(s) {
                (a,b)
            } else {
                println!("no node is selected");
                let n = ScenarioNode::new(); // dummy node to indicate Group when empty list
                n.set_value(Item::Group);    // empty list is treated as a group
                let sno = ScenarioNodeObject::new_from( Rc::new(n) );
                (sno, gio::ListStore::with_type(ScenarioNodeObject::static_type()))
            };

        let sno_value = sno.get_node();
        let sno_value = &*sno_value.value.borrow();
        let _ = (*snbbox.imp().buttons.borrow()).iter().map(|b|{
            if ScenarioNode::can_be_neighbor_or_child_auto(sno_value,
                                                           &b.node_type){
                b.set_state(true);
            } else {
                b.set_state(false);
            }
        }).collect::<Vec<_>>();
    }

    // new /////////////////////////////////////////////////
    pub fn new(single_selection: SingleSelection,
               history         : Rc<OperationHistory>) -> Self {
        let obj: ScenarioNodeButtonBox = Object::builder().build();

        // build buttons ///////////////////////////////////
        // make sure that names and items has the same number of items
        let mut names = vec!["Pm","i","M","P","S","G"];
        let mut items = vec![
            scenario_node::Item::Pmat(Mat::default()),
            scenario_node::Item::Ovimg(Ovimg::default()),
            scenario_node::Item::Mat(Mat::default()),
            scenario_node::Item::Page(Page::default()),
            scenario_node::Item::Scene(Scene::default()),
            scenario_node::Item::Group,
        ];
        let names_len = names.len();
        for _i in 0..names_len {
            let button = AddNodeButton::build(names.pop().unwrap(),
                                              single_selection.clone(),
                                              history.clone(),
                                              items.pop().unwrap());
            obj.append(&button.button);
            obj.imp().buttons.borrow_mut().push(button);
        }
        // sno-selected ////////////////////////////////////
        obj.connect_closure(
            "sno-selected",
            false,

            closure_local!(|w: Self, s: SingleSelection| {
                Self::sno_selected(w, s);
            }),
        );
        // sno-selected ////////////////////////////////////
        obj.connect_closure(
            "unset-sno",
            false,
            closure_local!(|w: Self, s: SingleSelection| {
                Self::sno_selected(w, s);
            }),
        );

        obj
    }

}
