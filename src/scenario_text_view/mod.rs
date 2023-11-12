mod imp;

use glib::Object;
use glib::WeakRef;
use glib::clone;
use glib::closure_local;
use gtk::SingleSelection;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::sno_list::selection_to_sno;

glib::wrapper! {
    pub struct ScenarioTextView(ObjectSubclass<imp::ScenarioTextView>)
        @extends gtk::TextView, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}
// ScenarioTextView ////////////////////////////////////////
impl ScenarioTextView {
    pub fn set_mediator(&self, m: WeakRef<Object>){ *self.imp().mediator.borrow_mut() = m; }
    // sno_selected ////////////////////////////////////////
    pub fn sno_selected(&self, s: SingleSelection){
        // update view
        let (sno, _store) =
            if let Some((a,b)) = selection_to_sno(s) { (a,b) } else { return; /* todo: no item */ };
        // set sno
        *self.imp().sno.borrow_mut() = Some( sno.clone() );

        // update buffer
        let t =
            if let Some(t) = sno.get_node().get_mat_text() { t } else {
                self.buffer().set_text( "" );
                return; /* not mat/pmat */ };
        self.buffer().set_text( &t );
    }

    // new /////////////////////////////////////////////////
    pub fn new() -> Self {
        let obj:ScenarioTextView = Object::builder().build();

        obj.connect_closure(
            "sno-selected",
            false,
            closure_local!(|t: Self, s: SingleSelection| {
                t.sno_selected(s);
            }),
        );

        obj.buffer().connect_text_notify( clone!( @strong obj => move |s|{
            let sno = if let Some( sno ) = obj.imp().sno.borrow().as_ref() { sno.clone() } else { return; };
            sno.get_node().set_mat_text( &s.text( &s.start_iter(), &s.end_iter(), true ).to_string() )  ;
            obj.imp().mediator.borrow().upgrade().expect("mediator").emit_by_name::<()>("mat-attribute-changed", &[&sno]);

        }) );


        obj
    }
}
