mod imp;

use std::rc::Rc;

use glib::Object;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::ListItem;
use gtk::gio;
use gtk::glib;

use crate::operation_history::OperationHistory;

glib::wrapper! {
    pub struct ScenarioItemDragObject(ObjectSubclass<imp::ScenarioItemDragObject>);
        //@extends gtk::,
        //@implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ScenarioItemDragObject {
    pub fn new() -> Self {
        Object::builder().build()
    }
    pub fn set_root_store(&self, s: gio::ListStore){
        *self.imp().root_store.borrow_mut()= Some(s.into());
    }
    pub fn set_history(&self, h: Rc<OperationHistory>){
        *self.imp().history.borrow_mut()= Some(h.into());
    }
    pub fn set_list_item(&self, l: ListItem){
        *self.imp().list_item.borrow_mut()= Some(l.into());
    }

    pub fn get_root_store(&self) -> gio::ListStore{
        self.imp().root_store.borrow().as_ref().unwrap().clone()
    }
    pub fn get_history(&self) -> Rc<OperationHistory>{
        self.imp().history.borrow().as_ref().unwrap().clone()
    }
    pub fn get_list_item(&self) -> ListItem{
        self.imp().list_item.borrow().as_ref().unwrap().clone()
    }
}

impl Default for ScenarioItemDragObject {
    fn default() -> Self {
        Self::new()
    }
}
