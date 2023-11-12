mod imp;

use std::cell::Cell;

use serde::{Deserialize, Serialize};

use gtk::glib;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct Isv2Parameter(ObjectSubclass<imp::Isv2Parameter>);
}

#[derive(Serialize, Deserialize)]
pub struct Isv2ParameterSerde{
    pub param: imp::Isv2Parameter,
}
impl From<&Isv2Parameter> for Isv2ParameterSerde {
    fn from(src: &Isv2Parameter) -> Self {
        let param = imp::Isv2Parameter{
            target_width       : Cell::new(src.imp().target_width.get()),
            target_height      : Cell::new(src.imp().target_height.get()),
            project_dir        : src.imp().project_dir.clone(),
            project_file_name  : src.imp().project_file_name.clone(),
            export_dir         : src.imp().export_dir.clone(),
            bgimg_en           : Cell::new(src.imp().bgimg_en.get()),
        };
        Self{
            param
        }
    }
}
impl From<&Isv2ParameterSerde> for Isv2Parameter{
    fn from(src: &Isv2ParameterSerde) -> Self{
        let obj = glib::Object::new::<Isv2Parameter>();
        obj.imp().target_width.set( src.param.target_width.get() );
        obj.imp().target_height.set( src.param.target_height.get() );
        *obj.imp().project_dir.borrow_mut()        = (*src.param.project_dir.borrow()).to_path_buf();
        *obj.imp().project_file_name.borrow_mut()  = (*src.param.project_file_name.borrow()).clone();
        *obj.imp().export_dir.borrow_mut()         = (*src.param.export_dir.borrow()).clone();
        obj.imp().bgimg_en.set( src.param.bgimg_en.get() );
        obj
    }
}

impl Isv2Parameter{
    // new /////////////////////////////////////////////////
    pub fn new() -> Self {
        let obj = glib::Object::new::<Isv2Parameter>();
        obj
    }
    // copy_from_serde
    pub fn copy_from_serde(&self, src: &Isv2ParameterSerde){
        self.imp().target_width.set( src.param.target_width.get() );
        self.imp().target_height.set( src.param.target_height.get() );
        // note: project_dir and project_file_name are updated when the file is opened
        *self.imp().export_dir.borrow_mut() = (*src.param.export_dir.borrow()).clone();
        self.imp().bgimg_en.set( src.param.bgimg_en.get() );
    }
}
