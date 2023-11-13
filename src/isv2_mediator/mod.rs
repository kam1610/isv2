mod imp;
use glib::closure_local;
use gtk::glib;
use gtk::SingleSelection;
use gtk::ListView;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::scenario_node_object::ScenarioNodeObject;
use crate::sno_list::get_belong_model;

glib::wrapper! {
    pub struct Isv2Mediator(ObjectSubclass<imp::Isv2Mediator>);
}

impl Isv2Mediator{
    pub fn new() -> Self {
        let obj = glib::Object::new::<Isv2Mediator>();
        // sno-selected ////////////////////////////////////
         obj.connect_closure(
             "sno-selected",
             false,
             closure_local!(|mediator: Self, s: SingleSelection| {
                 mediator.imp().scenario_text_view.borrow().emit_by_name::<()>("sno-selected", &[&s]);
                 mediator.imp().attr_box.borrow().emit_by_name::<()>("sno-selected", &[&s]);
                 mediator.imp().preview_window.borrow().emit_by_name::<()>("sno-selected", &[&s]);
                 mediator.imp().node_add_box.borrow().emit_by_name::<()>("sno-selected", &[&s]);
             })
         );
        // mat-attribute-changed ///////////////////////////
        obj.connect_closure(
            "mat-attribute-changed",
            false,
            closure_local!(|mediator: Self, s: ScenarioNodeObject| {
                mediator.imp().preview_window.borrow().emit_by_name::<()>("mat-attribute-changed", &[&s]);
            }));
        // sno-move-resize /////////////////////////////////
        obj.connect_closure(
            "sno-move-resize",
            false,
            closure_local!(|mediator: Self, s: ScenarioNodeObject| {
                let lv = mediator.imp().list_view.borrow().clone().downcast::<ListView>().expect("listview");
                let (model, pos) = get_belong_model( lv, s.get_node(), false );
                if let Some(m) = model {
                    m.items_changed(pos, 1, 1);
                }
                mediator.imp().attr_box.borrow().emit_by_name::<()>("sno-move-resize", &[&s]);
            }));
        // scene-attribute-changed /////////////////////////
        obj.connect_closure(
            "scene-attribute-changed",
            false,
            closure_local!(|mediator: Self, s: ScenarioNodeObject| {
                mediator.imp().preview_window.borrow().emit_by_name::<()>("scene-attribute-changed", &[&s]);
            }));
        // unset-sno ///////////////////////////////////////
        obj.connect_closure(
            "unset-sno",
            false,
            closure_local!(|mediator: Self, s: ScenarioNodeObject| {
                mediator.imp().attr_box.borrow().emit_by_name::<()>("unset-sno", &[&s]);
                mediator.imp().preview_window.borrow().emit_by_name::<()>("unset-sno", &[&s]);
                mediator.imp().node_add_box.borrow().emit_by_name::<()>("unset-sno", &[&s]);
            }));
        ////////////////////////////////////////////////////
        obj
    }
}
