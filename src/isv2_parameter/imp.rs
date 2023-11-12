use std::cell::{Cell, RefCell};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use glib::Properties;
use gtk::glib::prelude::*;
use gtk::glib;
use gtk::subclass::prelude::*;

// Object holding the state
#[derive(Debug, Properties, Serialize, Deserialize)]
#[properties(wrapper_type = super::Isv2Parameter)]
pub struct Isv2Parameter {
    #[property(get, set)]
    pub(super) target_width       : Cell<i32>,
    #[property(get, set)]
    pub(super) target_height      : Cell<i32>,
    #[property(get, set)]
    pub(super) project_dir        : RefCell<PathBuf>,
    #[property(get, set)]
    pub(super) project_file_name  : RefCell<String>,
    #[property(get, set)]
    pub(super) export_dir         : RefCell<String>,
    #[property(get, set)]
    pub(super) bgimg_en           : Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for Isv2Parameter {
    const NAME: &'static str = "Isv2Parameter";
    type Type = super::Isv2Parameter;
    type ParentType = glib::Object;
}

impl ObjectImpl for Isv2Parameter {
    // properties //////////////////////////////////////////
    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }
    fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        self.derived_set_property(id, value, pspec)
    }
    fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        self.derived_property(id, pspec)
    }
}

impl Default for Isv2Parameter{
    fn default() -> Self{
        let path = {
            if let Ok(p) = std::env::current_dir() { p }
            else { PathBuf::new() }};
        Self{
            target_width       : Cell::new(0),
            target_height      : Cell::new(0),
            project_dir        : RefCell::new(path),
            project_file_name  : RefCell::new(String::from("project.json")),
            export_dir         : RefCell::new(String::from("rel")),
            bgimg_en           : Cell::new(true),
        }
    }
}
