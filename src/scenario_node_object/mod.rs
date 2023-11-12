pub(crate) mod imp;

use std::rc::Rc;

use glib::Object;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::Cast;
use gtk::prelude::ListModelExt;
use gtk::{gio, glib, TreeListRow};

use crate::scenario_node::BranchType;
use crate::scenario_node::Item;
use crate::scenario_node::ScenarioNode;

glib::wrapper! {
    pub struct ScenarioNodeObject(ObjectSubclass<imp::ScenarioNodeObject>);
}

impl ScenarioNodeObject {
    pub fn new() -> Self {
        let obj: ScenarioNodeObject= Object::builder().build();
        let node= ScenarioNode::new();
        obj.set_node(Rc::new(node));
        obj
    }
    pub fn new_with_seq(seq: i32) -> Self {
        let obj= ScenarioNodeObject::new();
        obj.imp().seq.set(seq);
        obj
    }
    pub fn new_with_seq_id(seq: i32, id: i32) -> Self {
        let obj= ScenarioNodeObject::new_with_seq(seq);
        obj.set_id(id);
        obj
    }
    pub fn new_from(r: Rc<ScenarioNode>) -> Self{
        let obj: ScenarioNodeObject= Object::builder().build();
        obj.set_node(r);
        obj
    }

    pub fn get_node     (&self) -> Rc<ScenarioNode>  { self.imp().node.borrow().clone() }
    pub fn set_node     (&self, r: Rc<ScenarioNode>) { *self.imp().node.borrow_mut()= r; }
    pub fn set_vaue     (&self, v:Item)              { self.imp().node.borrow().set_value(v); }
    pub fn set_child    (&self, c: Rc<ScenarioNode>) { self.imp().node.borrow().set_child(c); }
    pub fn set_neighbor (&self, n: Rc<ScenarioNode>) { self.imp().node.borrow().set_neighbor(n); }
    pub fn set_parent   (&self, p: Rc<ScenarioNode>) { self.imp().node.borrow().set_parent(Rc::downgrade(&p)); }
    // set_seq is implemented by derive(Properties)
    pub fn get_seq      (&self) -> i32               { self.imp().seq.get() }
    pub fn set_id       (&self, id: i32)             { self.imp().node.borrow().id.set(id); }
    pub fn get_id       (&self) -> i32               { self.imp().node.borrow().id.get() }
    pub fn set_bt       (&self, b: BranchType)       { self.imp().node.borrow().set_bt(b); }
    pub fn get_bt       (&self) -> BranchType        { self.imp().node.borrow().bt.get() }
}

// adj_seq ///////////////////////////////////////////////
pub fn adj_seq (model: &gio::ListStore, greater_than: i32, i: i32) {
    for scn in model {
        let exp= scn.expect("scenario node");
        let scn_object= exp.downcast_ref::<ScenarioNodeObject>().unwrap();
        if scn_object.get_seq() >= greater_than {
            scn_object.set_seq( scn_object.get_seq() + i );
        }
    }
}
// add_neighbor ////////////////////////////////////////////
pub fn add_neighbor(dest_sno   : &ScenarioNodeObject,
                    new_node   : &ScenarioNodeObject,
                    dest_store : &gio::ListStore){

    ScenarioNode::mv_to_neighbor( dest_sno.get_node(), new_node.get_node() );

    new_node.set_seq( dest_sno.get_seq() + 1 );
    adj_seq( dest_store, dest_sno.get_seq() + 1, 1 );
    if dest_store.n_items() >= ((dest_sno.get_seq() + 1) as u32) {
        dest_store.insert( (dest_sno.get_seq() as u32) + 1, new_node ); }

}
// add_child ///////////////////////////////////////////////
pub fn add_child(dest_sno   : &ScenarioNodeObject,
                 new_node   : &ScenarioNodeObject,
                 dest_row   : &TreeListRow,
                 dest_store : &gio::ListStore){
    ScenarioNode::mv_to_child(dest_sno.get_node(), new_node.get_node());
    new_node.set_seq( 0 );

    if let Some(m) = dest_row.children(){
        let s= m.downcast::<gio::ListStore>().expect("ListStore");
        adj_seq( &s, 0, 1 );
        s.insert( 0, new_node );
    } else {
        if dest_store.n_items() >= (dest_sno.get_seq() as u32) {
            let list_model= dest_store.clone().upcast::<gio::ListModel>();
            list_model.items_changed(dest_sno.get_seq() as u32, 1, 1);
        }
    }
}
// remove_node /////////////////////////////////////////////
pub fn remove_node(dest_store: &gio::ListStore,
                   dest_sno  : &ScenarioNodeObject){
    adj_seq(dest_store, dest_sno.get_seq() + 1, -1);
    dest_sno.get_node().remove();
    dest_store.remove( dest_sno.get_seq() as u32 );
}
