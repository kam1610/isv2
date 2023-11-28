use std::rc::Rc;
use gtk::prelude::*;
use gtk::Entry;
use gtk::EntryBuffer;

// StatusBar ///////////////////////////////////////////////
pub struct StatusBar{
    pub entry : Entry,
}
impl StatusBar{
    // build ///////////////////////////////////////////////
    pub fn build() -> Rc<Self>{
        let entry = Entry::builder().buffer(&EntryBuffer::new(Some("init..."))).build();
        entry.set_editable(false);
        let sbar = StatusBar{ entry };
        Rc::new(sbar)
    }
    // set_status //////////////////////////////////////////
    pub fn set_status(&self, s: &str){
        println!("{}", s);
        self.entry.buffer().set_text(s);
    }
}
