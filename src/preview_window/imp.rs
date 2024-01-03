use glib::Object;
use glib::WeakRef;
use glib::subclass::Signal;
use gtk::SingleSelection;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::StaticType;
use gtk::subclass::prelude::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;

use once_cell::sync::Lazy;

use crate::drawing_util::util;
use crate::isv2_parameter::Isv2Parameter;
use crate::scenario_node::ScenarioNode;
use crate::scenario_node_object::ScenarioNodeObject;
use crate::status_bar::StatusBar;

// Object holding the state
pub struct PreviewWindow {
    pub(super) buf                  : RefCell<Option<Pixbuf>>,
    pub(super) scale_crop_buf       : RefCell<Option<Pixbuf>>,
    pub(super) area                 : RefCell<Vec<(Rc<ScenarioNode>, Option<Rc<ScenarioNode>>)>>, // mat, and reference label
    pub(super) is_area_transforming : Cell<bool>,
    pub(super) sno                  : RefCell<Option<ScenarioNodeObject>>,
    pub(super) area_state           : Cell<util::CursorState>,
    pub(super) target_sn            : RefCell<Option<Rc<ScenarioNode>>>,
    pub(super) begin_point          : Cell<(i32, i32)>,
    pub(super) mat_orig_point       : Cell<(i32, i32)>,
    pub(super) mediator             : RefCell<WeakRef<Object>>,
    pub(super) parameter            : RefCell<WeakRef<Isv2Parameter>>,
    pub(super) tgt_to_pwin_scale    : Cell<f64>,
    pub(super) status_bar           : RefCell<Option<Rc<StatusBar>>>,
    pub(super) img_mat_buf          : RefCell<HashMap<usize, Pixbuf>>,
    pub(super) hasher               : DefaultHasher,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for PreviewWindow {
    const NAME: &'static str = "PreviewWindow";
    type Type       = super::PreviewWindow;
    type ParentType = gtk::DrawingArea;
}

// Trait shared by all GObjects
impl ObjectImpl for PreviewWindow {
    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder("sno-selected")
                .param_types([SingleSelection::static_type()])
                .build(),
                 Signal::builder("mat-attribute-changed")
                .param_types([ScenarioNodeObject::static_type()])
                .build(),
                 Signal::builder("scene-attribute-changed")
                .param_types([ScenarioNodeObject::static_type()])
                .build(),
                 Signal::builder("unset-sno")
                .param_types([ScenarioNodeObject::static_type()])
                .build()
            ]
        });
        SIGNALS.as_ref()
    }
}

impl WidgetImpl for PreviewWindow {}

impl DrawingAreaImpl for PreviewWindow {}

impl Default for PreviewWindow {
    fn default() -> Self {
        Self{
            buf                  : RefCell::new(None),
            scale_crop_buf       : RefCell::new(Default::default()),
            area                 : RefCell::new(Vec::new()),
            is_area_transforming : Cell::new(false),
            sno                  : None.into(),
            area_state           : util::CursorState::None.into(),
            target_sn            : RefCell::new(None),
            begin_point          : Cell::new((0,0)),
            mat_orig_point       : Cell::new((0,0)),
            mediator             : RefCell::new(WeakRef::new()),
            parameter            : RefCell::new(WeakRef::new()),
            tgt_to_pwin_scale    : Cell::new(1.0),
            status_bar           : RefCell::new(None),
            img_mat_buf          : RefCell::new(HashMap::new()),
            hasher               : DefaultHasher::new(),
        }
    }
}
