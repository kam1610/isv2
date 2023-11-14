use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib;
use gtk::glib::clone;
use gtk::TreeListRow;
use gtk::SingleSelection;
use gtk::subclass::prelude::*;
use gtk::prelude::*;
use glib::subclass::Signal;

use once_cell::sync::Lazy;

use crate::scenario_node;
use crate::scenario_node::ScenarioNode;
use crate::isv2_button::Isv2Button;
use crate::scenario_node_object::ScenarioNodeObject;
use crate::operation_history::OperationHistoryItem;
use crate::operation_history::OperationHistory;
use crate::operation_history::Operation;
use crate::operation_history::TreeManipulationHandle;
use crate::tree_util::tree_manipulate;
use crate::scenario_node::{Scene, Page, Mat, Ovimg};
use crate::scenario_node_object::add_child;
use crate::scenario_node_object::add_neighbor;

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
    // button_clicked //////////////////////////////////////
    fn button_clicked(add_node_button: &AddNodeButton,
                      btn            : Isv2Button){

        if !add_node_button.button_state.get() { println!("(button_clicked) button is disabled"); return; }

        let new_node = ScenarioNodeObject::new_with_seq_id(0, tree_manipulate::gen_id());
        *new_node.get_node().value.borrow_mut() = match &add_node_button.node_type {
            scenario_node::Item::Group    => scenario_node::Item::Group,
            scenario_node::Item::Scene(_) => scenario_node::Item::Scene(Scene::default()),
            scenario_node::Item::Page(_)  => scenario_node::Item::Page(Page::default()),
            scenario_node::Item::Mat(_)   => scenario_node::Item::Mat(Mat::default()),
            scenario_node::Item::Ovimg(_) => scenario_node::Item::Ovimg(Ovimg::default()),
            scenario_node::Item::Pmat(_)  => scenario_node::Item::Pmat(Mat::default()),
            _ => scenario_node::Item::Group,
        };
        // confirm empty list
        if btn.get_selection().selected_item().is_none() {
            let root_store= btn.get_store();
            let h= OperationHistoryItem::new_with_root_store(Operation::AddRoot,
                                                             &root_store,
                                                             &new_node);
            add_node_button.button.get_history().push(h);
            tree_manipulate::add_node_to_empty_store(add_node_button.button.clone(),
                                                     &new_node);
            return;
        }

        let obj     = btn.get_selection().selected_item().unwrap();
        let sel_row = obj.downcast_ref::<TreeListRow>().expect("TreeListRow is expected");
        let sel_sno = sel_row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");

        let ope_type;
        // sel_belong_row //////////////////////////////////
        fn sel_belong_row(sel_sno: &ScenarioNodeObject,
                          btn    : &Isv2Button,
                          belong_func: &dyn Fn(&Rc<ScenarioNode>)->Option<Rc<ScenarioNode>> ) -> Result<(),()>{
            let target_s = {
                if let Some(s) = belong_func(&sel_sno.get_node()) {s}
                else {
                    println!("(AddNodeButton)unexpected condition {}:{}", file!(), line!());
                    return Err(()); }};
            let (_s, n) = tree_manipulate::search_row_with_sn_up_in_ssel(&*btn.get_selection(),
                                                                         target_s,
                                                                         btn.get_selection().selected());
            btn.get_selection().set_selected(n); // selection is updated tempolary to create history handle
            Ok(())
        }

        ////////////////////////////////////////////////////
        // ope-sel conditions //////////////////////////////
        match add_node_button.node_type { // ope
            // ope:grp /////////////////////////////////////
            scenario_node::Item::Group => {
                match *sel_sno.get_node().value.borrow() { // sel
                    scenario_node::Item::Group |
                    scenario_node::Item::Scene(_) => { // ope:grp, sel:grp,scn
                        ope_type = Operation::AddNeighbor;
                    },
                    scenario_node::Item::Page(_) |
                    scenario_node::Item::Pmat(_) |
                    scenario_node::Item::Mat(_)  |
                    scenario_node::Item::Ovimg(_) => { // ope:grp, sel:pg,pmt,mt,ovi
                        if let Ok(_) = sel_belong_row(&sel_sno, &btn, &ScenarioNode::get_belong_scene) {
                            ope_type = Operation::AddNeighbor; }
                        else {
                            return; }
                    },
                    _ => { ope_type = Operation::AddChild; }
                };
            },
            // ope:scn /////////////////////////////////////
            scenario_node::Item::Scene(_) => {
                match *sel_sno.get_node().value.borrow() { // sel
                    scenario_node::Item::Group => { // ope:scn, sel:grp
                        ope_type = Operation::AddChild;
                    },
                    scenario_node::Item::Scene(_) => { // ope:scn, sel:scn
                        ope_type = Operation::AddNeighbor;
                    },
                    scenario_node::Item::Page(_) |
                    scenario_node::Item::Pmat(_) |
                    scenario_node::Item::Mat(_)  |
                    scenario_node::Item::Ovimg(_) => { // ope:scn, sel:pg,pmt,mt,ovi
                        let target_s = {
                            if let Some(s) = ScenarioNode::get_belong_scene(&sel_sno.get_node()) {s}
                            else {
                                println!("(AddNodeButton)unexpected condition {}:{}", file!(), line!());
                                return; }};
                        let (_s, n) = tree_manipulate::search_row_with_sn_up_in_ssel(&*btn.get_selection(),
                                                                                     target_s,
                                                                                     btn.get_selection().selected());
                        btn.get_selection().set_selected(n);
                        ope_type = Operation::AddNeighbor;
                    },
                    _ => { ope_type = Operation::AddChild; }
                };
            },
            // ope:pg,pm ///////////////////////////////////
            scenario_node::Item::Page(_) |
            scenario_node::Item::Pmat(_) => {
                match *sel_sno.get_node().value.borrow() { // sel
                    scenario_node::Item::Group => {
                        return;
                    },
                    scenario_node::Item::Scene(_) => {
                        ope_type = Operation::AddChild;
                    },
                    scenario_node::Item::Page(_) |
                    scenario_node::Item::Pmat(_) => {
                        ope_type = Operation::AddNeighbor;
                    },
                    scenario_node::Item::Mat(_) |
                    scenario_node::Item::Ovimg(_) => {
                        if let Ok(_) = sel_belong_row(&sel_sno, &btn, &ScenarioNode::get_belong_page) {
                            ope_type = Operation::AddNeighbor; }
                        else {
                            return; }
                    },
                    _ => { ope_type = Operation::AddChild; }
                };
            },
            // ope:mat,ovi /////////////////////////////////
            scenario_node::Item::Mat(_) |
            scenario_node::Item::Ovimg(_) => {
                match *sel_sno.get_node().value.borrow() { // sel
                    scenario_node::Item::Group |
                    scenario_node::Item::Scene(_) => {
                        return;
                    },
                    scenario_node::Item::Page(_) => {
                        ope_type = Operation::AddChild;
                    },
                    scenario_node::Item::Pmat(_) => {
                        return;
                    },
                    scenario_node::Item::Mat(_) |
                    scenario_node::Item::Ovimg(_) => {
                        ope_type = Operation::AddNeighbor;
                    },
                    _ => { ope_type = Operation::AddChild; }
                };
            },
            _ => { ope_type = Operation::AddChild; }
        };

        // select kind of addition(child or neighbor_ //////
        let add_child_func = std::boxed::Box::new( |h: &TreeManipulationHandle, n: &ScenarioNodeObject|{
            add_child( h.sno.as_ref().unwrap().as_ref(),
                       n,
                       h.row.as_ref().unwrap().as_ref(),
                       h.store.as_ref().unwrap().as_ref()); } );
        let add_neighbor_func = std::boxed::Box::new( |h: &TreeManipulationHandle, n: &ScenarioNodeObject|{
            add_neighbor( h.sno.as_ref().unwrap().as_ref(),
                          n,
                          h.store.as_ref().unwrap().as_ref()); });
        let add_func : std::boxed::Box<dyn Fn(&TreeManipulationHandle, &ScenarioNodeObject)>;

        if ope_type == Operation::AddChild {
            add_func = add_child_func; }
        else {
            add_func = add_neighbor_func; }

        if let Ok(hdl) = tree_manipulate::isv2button_to_dest_member4(&btn){
            add_func(&hdl, &new_node);

            let mut h= OperationHistoryItem::new_from_handle(ope_type, hdl); // error! must chose AddNeighbor or AddChild
            h.new_sno= Some( Rc::new(new_node.clone()) );

            add_node_button.button.get_history().push(h);

            // update selection to select added node
            let (s, n) = tree_manipulate::search_row_with_sn_down_in_ssel(&*btn.get_selection(),
                                                                          new_node.get_node().clone(),
                                                                          btn.get_selection().selected());
            if s.is_some() { btn.get_selection().set_selected(n); }
        } else {
            println!("(add_node_button) unexpected condition!");
        }
    }
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
        add_node_button.button.connect_clicked(clone!(@strong add_node_button => move |_btn| {
            Self::button_clicked(&add_node_button, add_node_button.button.clone());
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
