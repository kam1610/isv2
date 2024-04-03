mod imp;

use std::cell::Cell;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::rc::Weak;

use gio::Cancellable;
use glib::GString;
use glib::Object;
use glib::clone;
use glib::closure_local;
use glib::object::WeakRef;
use glib::signal::Propagation;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::Button;
use gtk::CheckButton;
use gtk::ColorDialog;
use gtk::DrawingArea;
use gtk::DropDown;
use gtk::EntryBuffer;
use gtk::EventControllerKey;
use gtk::EventControllerMotion;
use gtk::FileDialog;
use gtk::FileFilter;
use gtk::FontDialog;
use gtk::FontDialogButton;
use gtk::GestureClick;
use gtk::ListItem;
use gtk::Root;
use gtk::SignalListItemFactory;
use gtk::SingleSelection;
use gtk::StringObject;
use gtk::Window;
use gtk::cairo::Context;
use gtk::gdk::RGBA;
use gtk::gdk::Rectangle;
use gtk::gdk_pixbuf::InterpType;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio;
use gtk::glib;
use gtk::pango::FontDescription;
use gtk::pango;
use gtk::prelude::*;
use gtk::{Label, Entry, Box, Widget, Orientation, Align};

use crate::drawing_util::util;
use crate::isv2_parameter::Isv2Parameter;
use crate::scenario_node::LabelType;
use crate::scenario_node;
use crate::scenario_node_object::ScenarioNodeObject;
use crate::sno_list::selection_to_sno;

glib::wrapper! {
    pub struct ScenarioNodeAttributeBox(ObjectSubclass<imp::ScenarioNodeAttributeBox>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

// set_css_grayout /////////////////////////////////////////
fn set_css_grayout(list: Vec<impl IsA<Widget>>, enable: bool) {
    if enable{
        let _ = list.iter().map(|w| { w.add_css_class("label_ref_gray_out"); }).collect::<Vec<_>>(); }
    else{
        let _ = list.iter().map(|w| { w.remove_css_class("label_ref_gray_out")}).collect::<Vec<_>>(); }
}
// format_rgba /////////////////////////////////////////////
fn format_rgba_str(rgba: RGBA) -> String{
    let colors:Vec<f32> = vec![ rgba.red(), rgba.green(), rgba.blue(), rgba.alpha() ];
    let colors:Vec<_> = colors.iter().map(|a|{ (a * 255.0) as u32 }).collect();
    let colors:Vec<_> = colors.iter().map(|a|{ format!("{:0>2x?}",a) }).collect();
    let colors = "#".to_owned() + &colors.concat();
    colors
}
// Isv2ColorBox ////////////////////////////////////////////
struct Isv2ColorBox{
    pub color_box   : Box,
    pub color_label : Label,
    pub color_entry : Entry
}
impl Isv2ColorBox {
    // build ///////////////////////////////////////////////
    pub fn build<F> (root           : Root,
                     capt_label     : &str,
                     mediator       : WeakRef<Object>,
                     store          : gio::ListStore,
                     sno            : ScenarioNodeObject,
                     ini_col        : &Vec<u32>,
                     sno_update_func: F)  -> Self
    where F: Fn(ScenarioNodeObject, Vec<u32>) + 'static {
        // color ///////////////////////////////////////////////
        let ini_col: Vec<_> = ini_col.iter().map(|a|{ format!("{:0>2x?}", a) }).collect();
        let ini_col = "#".to_owned() + &ini_col.concat();
        let color_box    = Box::builder().orientation(Orientation::Horizontal).build();
        let color_label  = Label::new(Some(capt_label));
        let color_entry  = Entry::builder()
            .width_chars(9)
            .buffer(&EntryBuffer::new(Some(ini_col.clone())))
            .build();
        let color_button = Button::builder().css_classes(vec!["isv2_button"]).build();
        let color_dialog = ColorDialog::builder()
            .modal(true)
            .title("chose mat color")
            .with_alpha(true)
            .build();
        let color_preview = DrawingArea::builder().build();
        color_preview.set_draw_func(clone!( @strong color_entry,
                                            @strong color_button => move |_p, cr, _w, _h|{
            let pad = 2.0;
            if let Ok(rgba) = RGBA::parse( color_entry.text() ){
                cr.set_source_rgba(rgba.red() as f64, rgba.green() as f64, rgba.blue() as f64, rgba.alpha() as f64);
                cr.rectangle(pad, pad, color_button.height() as f64 - pad, color_button.width() as f64 - pad);
                cr.fill().expect("fill color box button in Isv2ColorBox");
            }
        }));
        color_button.set_child(Some(&color_preview));
        // color button action /////////////////////////////////
        color_button.connect_clicked(
            clone! (@weak   color_entry,
                    @weak   root,
                    @strong color_dialog,
                    @strong color_button => move |_b| {
                let root_win = root.downcast_ref::<Window>().unwrap();
                let rgba = match RGBA::parse(color_entry.text()) {
                    Ok(c) => Some(c),
                    Err(_) => None
                };
                color_dialog.choose_rgba(Some(root_win),
                                         rgba.as_ref(),
                                         None::<&Cancellable>,
                                         clone!(@weak color_entry => move |res|{
                                             match res {
                                                 Ok(r)  => color_entry.set_text( &format_rgba_str(r) ),
                                                 Err(_) => () /* do nothing */
                                             }
                                         }));
            })
        );
        // color entry callback ////////////////////////////////
        color_entry.connect_changed( clone!( @strong sno,
                                             @weak   store,
                                             @weak   color_preview,
                                             @strong mediator => move |ce|{
                                                 if let Ok(rgba) = RGBA::parse( ce.text() ) {
                                                     color_preview.queue_draw();
                                                     let colors:Vec<f32> = vec![ rgba.red(), rgba.green(), rgba.blue(), rgba.alpha() ];
                                                     let colors:Vec<_> = colors.iter().map(|a|{ (a * 255.0) as u32 }).collect();
                                                     sno_update_func(sno.clone(), colors); // call closure!
                                                     store.items_changed(sno.get_seq() as u32, 1, 1);
                                                     mediator.upgrade().unwrap().emit_by_name::<()>("mat-attribute-changed", &[&sno]);
                                                 }
                                             } ) );

        color_box.append(&color_label);
        color_box.append(&color_entry);
        color_box.append(&color_button);

        let obj = Self{ color_box,
                        color_label,
                        color_entry };

        if sno.get_node().get_label_type() == Some(LabelType::Ref) ||
           sno.get_node().get_label_type() == Some(LabelType::RefNoRect) {
            set_css_grayout(vec![obj.color_label.clone().upcast::<Widget>(),
                                 obj.color_entry.clone().upcast::<Widget>() ],
                            true);
        }

        obj
    }
    pub fn get_box(&self) -> Box { self.color_box.clone() }
}
// dropdown_key_controller /////////////////////////////////
fn dropdown_key_controller(dropdown: DropDown) -> EventControllerKey{
    let kctrl = EventControllerKey::new();
    kctrl.connect_key_pressed(
        clone!(@strong dropdown => move|_ctrl, key, _code, _state|{
            let selected_pos = dropdown.selected() as i32;
            let mut prop = Propagation::Stop;
            let selected_pos_new:i32 = match key.name().unwrap().as_str() {
                "Up"   => selected_pos - 1,
                "Down" => selected_pos + 1,
                "Home" => 0,
                "End"  => (dropdown.model().unwrap().n_items() as i32) - 1,
                _      => {prop = Propagation::Proceed;  -1}
            };
            if (0 <= selected_pos_new) && (selected_pos_new < (dropdown.model().unwrap().n_items() as i32)){
                dropdown.set_selected( selected_pos_new as u32);
            }
            prop
        }));
    kctrl
}
// weight_select_box ///////////////////////////////////////
fn font_weight_select_box<F>(sno            : ScenarioNodeObject,
                             caption        : String,
                             store          : gio::ListStore,
                             mediator       : WeakRef<Object>,
                             mediator_msg   : String,
                             sno_update_func: F) -> Box
    where F: Fn(ScenarioNodeObject, String) + 'static {
    let weight_box        = Box::builder().orientation(Orientation::Horizontal).build();
    let weight_dd_factory = SignalListItemFactory::new();
    let weight_type_store = gio::ListStore::with_type( StringObject::static_type() );
    let weight_label      = Label::new(Some(&caption));

    let mut i = 0;
    for w_s_tpl in util::StrumWeight::variants(){
        weight_type_store.insert(i, &StringObject::new( w_s_tpl.1 )); // w.0:enum, w.1:String
        i+= 1;
    }
    let weight_dd = DropDown::builder().model(&weight_type_store).factory(&weight_dd_factory).build();
    weight_dd_factory.connect_setup(move |_, list_item|{
        let label = Label::builder().halign(Align::Start).build();
        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&label));
    });
    weight_dd_factory.connect_bind(move |_, list_item| {
        let list_item = list_item.downcast_ref::<ListItem>().expect("list item");
        let label     = list_item.child().and_downcast::<Label>().expect("label");
        let item      = list_item.item().and_downcast::<StringObject>().expect("string object");
        label.set_label(&item.string());
    });
    weight_dd.set_factory(Some(&weight_dd_factory));

    // select default label type by sno
    for i in 0..weight_type_store.n_items() {
        let s = weight_type_store.item(i).and_downcast::<StringObject>().expect("string object");
        if s.string().as_str() == sno.get_node().get_mat_font_weight().unwrap() { // todo: parameterise get_weight_type()
            weight_dd.set_selected(i);
        }
    }
    // selected_notify
    weight_dd.connect_selected_notify( clone!(@weak   sno,
                                              @weak   store,
                                              @strong mediator,
                                              @strong mediator_msg => move |dd|{
                                                  let item = dd.selected_item().unwrap();
                                                  let s = item.downcast_ref::<StringObject>().expect("string_object");
                                                  sno_update_func(sno.clone(), s.string().as_str().to_string()); // call closure
                                                  store.items_changed(sno.get_seq() as u32, 1, 1);
                                                  mediator.upgrade().unwrap().emit_by_name::<()>(&mediator_msg, &[&sno]);
                                              }));
    // key controller
    let kctrl = dropdown_key_controller(weight_dd.clone());
    weight_dd.add_controller(kctrl);

    weight_box.append(&weight_label);
    weight_box.append(&weight_dd);

    if sno.get_node().get_label_type() == Some(LabelType::Ref) ||
       sno.get_node().get_label_type() == Some(LabelType::RefNoRect) {
           set_css_grayout(vec![weight_label.clone().upcast::<Widget>(),
                                weight_dd.first_child().unwrap().clone().upcast::<Widget>() ],
                           true);
       }

    weight_box
}
// label_type_select_box ///////////////////////////////////
fn label_type_select_box(sno              : ScenarioNodeObject,
                         gray_list        : Vec<impl IsA<Widget>>,
                         gray_list_posdim : Vec<impl IsA<Widget>>,
                         store            : gio::ListStore,
                         mediator         : WeakRef<Object>,
                         mediator_msg     : String) -> (Box, Widget) {
    let label_box        = Box::builder().orientation(Orientation::Horizontal).build();
    let label_dd_factory = SignalListItemFactory::new();
    let label_type_store = gio::ListStore::with_type( StringObject::static_type() );
    let label_label      = Label::new(Some("label type"));
    label_type_store.insert(0, &StringObject::new( LabelType::None.pretty_format()      ));
    label_type_store.insert(1, &StringObject::new( LabelType::Def.pretty_format()       ));
    label_type_store.insert(2, &StringObject::new( LabelType::Ref.pretty_format()       ));
    label_type_store.insert(3, &StringObject::new( LabelType::RefNoRect.pretty_format() ));
    let label_dd = DropDown::builder().model(&label_type_store).factory(&label_dd_factory).build();
    label_dd_factory.connect_setup(move |_, list_item|{
        let label = Label::new(None);
        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&label));
    });
    label_dd_factory.connect_bind(move |_, list_item| {
        let list_item = list_item.downcast_ref::<ListItem>().expect("list item");
        let label     = list_item.child().and_downcast::<Label>().expect("label");
        let item      = list_item.item().and_downcast::<StringObject>().expect("string object");
        label.set_label(&item.string());
    });
    label_dd.set_factory(Some(&label_dd_factory));

    // select default label type by sno
    for i in 0..label_type_store.n_items() {
        let s = label_type_store.item(i).and_downcast::<StringObject>().expect("string object");
        if &s.string() == sno.get_node().get_label_type().unwrap().pretty_format() {
            label_dd.set_selected(i);
        }
    }
    // selected_notify
    let gray_list = Rc::new(gray_list);
    label_dd.connect_selected_notify( clone!( @strong gray_list,
                                              @weak   sno,
                                              @weak   store,
                                              @strong mediator,
                                              @strong mediator_msg => move |dd|{
                                                  let item = dd.selected_item().unwrap();
                                                  let s = item.downcast_ref::<StringObject>().expect("string_object");
                                                  if LabelType::from( &s.string() ) == LabelType::Ref {
                                                      set_css_grayout(gray_list_posdim.to_vec(), true);
                                                      set_css_grayout(gray_list.to_vec(),        true); }
                                                  else if LabelType::from( &s.string() ) == LabelType::RefNoRect{
                                                      set_css_grayout(gray_list_posdim.to_vec(), false);
                                                      set_css_grayout(gray_list.to_vec(),        true); }
                                                  else {
                                                      set_css_grayout(gray_list_posdim.to_vec(), false);
                                                      set_css_grayout(gray_list.to_vec(),        false);
                                                  }
                                                  sno.get_node().set_label_type( LabelType::from( &s.string() ) );
                                                  store.items_changed(sno.get_seq() as u32, 1, 1);
                                                  mediator.upgrade().unwrap().emit_by_name::<()>(&mediator_msg, &[&sno]);
                                              }));
    // key controller
    let kctrl = dropdown_key_controller(label_dd.clone());
    label_dd.add_controller(kctrl);

    let label_entry  = Entry::builder()
         .buffer(&EntryBuffer::new( sno.get_node().get_label() ))
         .build();
    label_entry.connect_changed( clone!( @weak sno,
                                         @weak store,
                                         @strong mediator => move |entry|{
                                             sno.get_node().set_label( Some( entry.text().to_string() ) );
                                             store.items_changed(sno.get_seq() as u32, 1, 1);
                                             mediator.upgrade().unwrap().emit_by_name::<()>(&mediator_msg, &[&sno]);
                                         }));

    label_box.append(&label_label);
    label_box.append(&label_dd);
    label_box.append(&label_entry);

    (label_box, label_entry.upcast::<Widget>())
}
// mat_pos_dim_box /////////////////////////////////////////
struct Isv2PosDimBox{
    pub posdim_box   : Box,
    pub posdim_label : Label,
    pub posdim_entry : Entry
}
impl Isv2PosDimBox{
    pub fn build(b        : &ScenarioNodeAttributeBox,
                 sno      : ScenarioNodeObject,
                 store    : gio::ListStore,
                 mediator : WeakRef<Object>) -> Self{
        let posdim_box = Box::builder().orientation(Orientation::Horizontal).build();
        let posdim_label = Label::new(Some("x,y,w,h"));
        let (x, y, w, h) = sno.get_node().get_mat_pos_dim().unwrap();
        let posdim_str = format!("{},{},{},{}", x, y, w, h);
        let posdim_entry = Entry::builder().buffer(&EntryBuffer::new(Some(posdim_str))).build();
        posdim_entry.connect_changed( clone!( @strong sno,
                                               @weak store,
                                               @strong mediator => move |pe| {
                                                   let t = pe.text();
                                                   let v:Vec<&str> = t.split(",").collect();
                                                   if v.len() != 4 { return; }
                                                   if let ( Ok(x), Ok(y), Ok(w), Ok(h) ) =
                                                       (i32::from_str_radix(v[0], 10),
                                                        i32::from_str_radix(v[1], 10),
                                                        i32::from_str_radix(v[2], 10),
                                                        i32::from_str_radix(v[3], 10)) {
                                                           sno.get_node().set_mat_pos_dim(x, y, w, h);
                                                           store.items_changed(sno.get_seq() as u32, 1, 1);
                                                           mediator.upgrade().unwrap()
                                                               .emit_by_name::<()>("mat-attribute-changed", &[&sno]);
                                                       }
                                               }));
        b.imp().mat_posdim_entry.set(Some(&posdim_entry));

        posdim_box.append(&posdim_label);
        posdim_box.append(&posdim_entry);

        if sno.get_node().get_label_type() == Some(LabelType::Ref){
            set_css_grayout(vec![posdim_label.clone().upcast::<Widget>(),
                                 posdim_entry.clone().upcast::<Widget>() ],
                            true);
        }

        Self{ posdim_box,
              posdim_label,
              posdim_entry }
    }
    pub fn get_box(&self) -> Box { self.posdim_box.clone() }
}
struct Isv2PosBox{
    pub pos_box   : Box,
    pub pos_label : Label,
    pub pos_entry : Entry
}
impl Isv2PosBox{
    pub fn build(sno      : ScenarioNodeObject,
                 store    : gio::ListStore,
                 mediator : WeakRef<Object>) -> Self{
        let pos_box = Box::builder().orientation(Orientation::Horizontal).build();
        let pos_label = Label::new(Some("text pos from top left(x,y)"));
        let (x, y) = sno.get_node().get_mat_text_pos().unwrap();
        let pos_str = format!("{},{}", x, y);
        let pos_entry = Entry::builder().buffer(&EntryBuffer::new(Some(pos_str))).build();
        pos_entry.connect_changed( clone!( @strong  sno,
                                            @strong store,
                                            @strong mediator => move |pe| {
                                                let t = pe.text();
                                                let v:Vec<&str> = t.split(",").collect();
                                                if v.len() != 2 { return; }
                                                if let ( Ok(x), Ok(y)) =
                                                    (i32::from_str_radix(v[0], 10),
                                                     i32::from_str_radix(v[1], 10)) {
                                                        sno.get_node().set_mat_text_pos(x, y);
                                                        store.items_changed(sno.get_seq() as u32, 1, 1);
                                                        mediator.upgrade().unwrap().emit_by_name::<()>("mat-attribute-changed", &[&sno]);
                                                    }
                                            }));
        pos_box.append(&pos_label);
        pos_box.append(&pos_entry);

        if sno.get_node().get_label_type() == Some(LabelType::Ref){
            set_css_grayout(vec![pos_label.clone().upcast::<Widget>(),
                                 pos_entry.clone().upcast::<Widget>() ],
                            true);
        }
        Self{ pos_box, pos_label, pos_entry }
    }
}
// float_entry_box /////////////////////////////////////////
struct Isv2FloatEntryBox {
    pub hbox  : Box,
    pub label : Label,
    pub entry : Entry,
}
impl Isv2FloatEntryBox{
    pub fn build <F1, F2, T>(sno          : ScenarioNodeObject,
                             mediator     : WeakRef<Object>,
                             store        : gio::ListStore,
                             caption      : &str,
                             get_func     : F1,
                             set_func     : F2,
                             mediator_msg : String) -> Self
    where F1: Fn(ScenarioNodeObject) -> T + 'static,
          F2: Fn(ScenarioNodeObject, T) + 'static,
    T : std::fmt::Display + std::str::FromStr
    {
        let hbox  = Box::builder().orientation(Orientation::Horizontal).build();
        let label = Label::new(Some(caption));
        let entry = Entry::builder().build();
        entry.set_text( &(format!("{:.3}", get_func(sno.clone()))) );
        hbox.append(&label);
        hbox.append(&entry);
        entry.connect_changed(clone!(@strong sno, @strong mediator, @weak store, @strong mediator_msg => move|entry|{
            if let Ok(val) = entry.text().parse::<T>() {
                set_func(sno.clone(), val);
                store.items_changed(sno.get_seq() as u32, 1, 1);
                mediator.upgrade().unwrap().emit_by_name::<()>(&mediator_msg, &[&sno]);
            }
        }));

        // initial gray out state
        if sno.get_node().get_label_type() == Some(LabelType::Ref) ||
           sno.get_node().get_label_type() == Some(LabelType::RefNoRect){
            set_css_grayout(vec![label.upcast_ref::<Widget>().clone(),
                                 entry.upcast_ref::<Widget>().clone()], true);
        }
        Self{hbox, label, entry}
    }
}
// build_mat_attribute_box ////////////////////////////////
fn build_mat_attribute_box(b         : &ScenarioNodeAttributeBox,
                           root      : Root,
                           mediator  : WeakRef<Object>,
                           store     : gio::ListStore,
                           sno       : ScenarioNodeObject,
                           temp_box  : &Box,
                           parameter : Isv2Parameter) -> Widget{
    // name ////////////////////////////////////////////////
    /*
    let name_box = Box::builder().orientation(Orientation::Horizontal).build();
    let name_label = Label::new(Some("name"));
    let name_entry = Entry::builder().build();
    name_box.append(&name_label);
    name_box.append(&name_entry);
    */

    // bgimg ///////////////////////////////////////////////
    let bgimg_box = Isv2FileDialogBox::build(root.clone(),
                                             sno.clone(),
                                             mediator.clone(),
                                             parameter.clone(),
                                             "mat-attribute-changed".to_string());

    // color_box ///////////////////////////////////////////
    let color_box = Rc::new(Isv2ColorBox::build( root.clone(),
                                                 "mat color",
                                                 mediator.clone(),
                                                 store.clone(),
                                                 sno.clone(),
                                                 &sno.get_node().get_mat_rgba().unwrap(),
                                                 |s, c| { s.get_node().set_mat_rgba(c) } ));
    // posdim //////////////////////////////////////////////
    let posdim_box = Isv2PosDimBox::build(b, sno.clone(), store.clone(), mediator.clone());
    // text pos ///////////////////////////////////////////
    let text_pos_box = Isv2PosBox::build(sno.clone(), store.clone(), mediator.clone());

    // round ///////////////////////////////////////////////
    let round_box   = Box::builder().orientation(Orientation::Horizontal).build();
    let round_label = Label::new(Some("round"));
    let round_entry = Entry::builder().build();
    round_entry.set_text(&format!("{}", sno.get_node().get_mat_r().unwrap()));
    round_entry.connect_changed(clone!(
        @strong mediator,
        @strong sno,
        @strong store => move |re|{
            if let Ok(r) = re.text().parse::<i32>(){
                sno.get_node().set_mat_r(r);
                store.items_changed(sno.get_seq() as u32, 1, 1);
                mediator.upgrade().unwrap()
                    .emit_by_name::<()>("mat-attribute-changed", &[&sno]);
            }
        }));
    round_box.append(&round_label);
    round_box.append(&round_entry);
    if sno.get_node().get_label_type() == Some(LabelType::Ref) ||
       sno.get_node().get_label_type() == Some(LabelType::RefNoRect){
        set_css_grayout(vec![round_label.upcast_ref::<Widget>().clone(),
                             round_entry.upcast_ref::<Widget>().clone()], true);
    }

    // font family, size ///////////////////////////////////
    let font_dialog = FontDialog::builder().modal(true).build();
    let mut font_desc = FontDescription::new();
    font_desc.set_family( &sno.get_node().get_mat_font_family().unwrap() );
    font_desc.set_size( sno.get_node().get_mat_font_size().unwrap() * pango::SCALE);
    let font_dialog_button = FontDialogButton::builder()
        .font_desc(&font_desc).dialog(&font_dialog).build();
    if sno.get_node().get_label_type() == Some(LabelType::Ref) ||
       sno.get_node().get_label_type() == Some(LabelType::RefNoRect){
        font_dialog_button.first_child().unwrap().add_css_class("label_ref_gray_out"); }
    font_dialog_button.connect_font_desc_notify(clone! (@strong mediator, @strong sno => move |b|{
        let desc = b.font_desc().unwrap();
        sno.get_node().set_mat_font_family( &desc.family().unwrap() );
        sno.get_node().set_mat_font_size( desc.size() / pango::SCALE );
        mediator.upgrade().unwrap()
            .emit_by_name::<()>("mat-attribute-changed", &[&sno]);
    }));

    // font color //////////////////////////////////////////
    let font_color_box = Rc::new(Isv2ColorBox::build( root.clone(),
                                                      "font color",
                                                      mediator.clone(),
                                                      store.clone(),
                                                      sno.clone(),
                                                      &sno.get_node().get_mat_font_rgba().unwrap(),
                                                      |s, c| { s.get_node().set_mat_font_rgba(c) } ));

    // weight list 1 ///////////////////////////////////////
    let font_weight_box = font_weight_select_box(sno.clone(),
                                                 "font weight".to_string(),
                                                 store.clone(),
                                                 mediator.clone(),
                                                 "mat-attribute-changed".to_string(),
                                                 |s, w| { s.get_node().set_mat_font_weight(w); });

    // font color 2 ////////////////////////////////////////
    let font_color_box_2 = Rc::new(Isv2ColorBox::build( root.clone(),
                                                        "font stroke",
                                                        mediator.clone(),
                                                        store.clone(),
                                                        sno.clone(),
                                                        &sno.get_node().get_mat_font_rgba_2().unwrap(),
                                                        |s, c| { s.get_node().set_mat_font_rgba_2(c) } ));
    // font_outl ///////////////////////////////////////////
    let outl_box = Isv2FloatEntryBox::build(sno.clone(),
                                            mediator.clone(),
                                            store.clone(),
                                            "font ountline",
                                            |sno|{ sno.get_node().get_mat_font_outl_2().unwrap() }, // f32
                                            |sno, val|{ sno.get_node().set_mat_font_outl_2(val); },
                                            "mat-attribute-changed".to_string());
    // line_spacing ////////////////////////////////////////
    let lspacing_box = Isv2FloatEntryBox::build(sno.clone(),
                                                mediator.clone(),
                                                store.clone(),
                                                "line spacing",
                                                |sno|{ sno.get_node().get_mat_line_spacing().unwrap() }, // f32
                                                |sno, val|{ sno.get_node().set_mat_line_spacing(val); },
                                                "mat-attribute-changed".to_string());
    // vertical writing ////////////////////////////////////
    let vertical_box   = Box::builder().orientation(Orientation::Horizontal).build();
    let vertical_label = Label::new(Some("vertical writing"));
    let vertical_check = CheckButton::builder().active(sno.get_node().get_mat_vertical().unwrap()).build();
    vertical_box.append(&vertical_label);
    vertical_box.append(&vertical_check);
    vertical_check.connect_toggled(
        clone!(@strong mediator,
               @strong sno => move|vc|{
                   sno.get_node().set_mat_vertical(vc.is_active());
                   mediator.upgrade().unwrap()
                       .emit_by_name::<()>("scene-attribute-changed", &[&sno]);
               }));
    if sno.get_node().get_label_type() == Some(LabelType::Ref) ||
       sno.get_node().get_label_type() == Some(LabelType::RefNoRect) {
        set_css_grayout(vec![vertical_label.upcast_ref::<Widget>().clone()], true);
    }
    // label ///////////////////////////////////////////////
    let gray_list = vec![bgimg_box.file_dialog_label.upcast_ref::<Widget>().clone(),
                         bgimg_box.file_dialog_entry.upcast_ref::<Widget>().clone(),
                         bgimg_box.enable_check.upcast_ref::<Widget>().clone(),
                         round_label.upcast_ref::<Widget>().clone(),
                         round_entry.upcast_ref::<Widget>().clone(),
                         color_box.color_label.upcast_ref::<Widget>().clone(),
                         color_box.color_entry.upcast_ref::<Widget>().clone(),
                         font_dialog_button.first_child().unwrap().upcast_ref::<Widget>().clone(),
                         font_color_box.color_label.upcast_ref::<Widget>().clone(),
                         font_color_box.color_entry.upcast_ref::<Widget>().clone(),
                         font_color_box_2.color_label.upcast_ref::<Widget>().clone(),
                         font_color_box_2.color_entry.upcast_ref::<Widget>().clone(),
                         outl_box.label.upcast_ref::<Widget>().clone(),
                         outl_box.entry.upcast_ref::<Widget>().clone(),
                         lspacing_box.label.upcast_ref::<Widget>().clone(),
                         lspacing_box.entry.upcast_ref::<Widget>().clone(),
                         vertical_label.upcast_ref::<Widget>().clone(),
                         text_pos_box.pos_label.upcast_ref::<Widget>().clone(),
                         text_pos_box.pos_entry.upcast_ref::<Widget>().clone()];
    let gray_list_posdim = vec![posdim_box.posdim_label.upcast_ref::<Widget>().clone(),
                                posdim_box.posdim_entry.upcast_ref::<Widget>().clone()];

    let (label_box, focus_tag) = label_type_select_box(sno.clone(),
                                                       gray_list,
                                                       gray_list_posdim,
                                                       store.clone(),
                                                       mediator.clone(),
                                                       "mat-attribute-changed".to_string());

    // apply to temp_box
    //temp_box.append(&name_box);
    temp_box.append(&bgimg_box.file_dialog_box);
    temp_box.append(&color_box.get_box());
    temp_box.append(&label_box);
    temp_box.append(&posdim_box.get_box());
    temp_box.append(&round_box);
    temp_box.append(&text_pos_box.pos_box);
    temp_box.append(&font_dialog_button);
    temp_box.append(&font_color_box.get_box());
    temp_box.append(&font_weight_box);
    temp_box.append(&font_color_box_2.get_box());
    temp_box.append(&outl_box.hbox);
    temp_box.append(&lspacing_box.hbox);
    temp_box.append(&vertical_box);

    focus_tag
}
// Isv2FileDialogBox ///////////////////////////////////////
struct Isv2FileDialogBox{
    pub file_dialog_box    : Box,
    pub file_dialog_label  : Label,
    pub file_dialog_entry  : Entry,
    pub enable_check       : CheckButton,
}
impl Isv2FileDialogBox {
    pub fn build(root         : Root,
                 sno          : ScenarioNodeObject,
                 mediator     : WeakRef<Object>,
                 parameter    : Isv2Parameter,
                 mediator_msg : String
    ) -> Self{
        let file_dialog_box = Box::builder().orientation(Orientation::Horizontal).build();
        let file_dialog_label = Label::new(Some("bg img"));
        let file_dialog = FileDialog::builder().modal(true).build();
        set_file_dialog_initial_path(&file_dialog, &sno, &parameter);
        let file_filter = FileFilter::new();
        file_filter.add_pixbuf_formats();
        file_filter.set_name(Some("image"));
        let model = gio::ListStore::with_type(FileFilter::static_type());
        model.append(&file_filter);
        file_dialog.set_filters(Some(&model));
        file_dialog.set_default_filter(Some(&file_filter)); // https://gitlab.gnome.org/GNOME/gtk/-/issues/6071

        let bgimg = sno.get_node().get_bgimg();
        let file_entry_str =
            if let Some(ref p) = bgimg{
                p.to_str().unwrap().clone()
            } else {
                ""
            };
        let file_dialog_entry = Entry::builder()
            .editable(false)
            .buffer(&EntryBuffer::new(Some(file_entry_str)))
            .build();
        file_dialog_entry.add_css_class("uneditable");
        file_dialog_entry.set_hexpand(true);

        let file_dialog_button = Button::with_label("...");
        file_dialog_button.add_css_class("isv2_button");
        file_dialog_button.connect_clicked(
            clone!(@weak   root,
                   @strong file_dialog,
                   @strong file_dialog_entry,
                   @weak   sno,
                   @strong mediator,
                   @strong mediator_msg => move |_b|{
                       let root_win = root.downcast_ref::<Window>().unwrap();
                       file_dialog.open(
                           Some(root_win),
                           None::<&Cancellable>,
                           clone!(@strong file_dialog_entry,
                                  @strong sno,
                                  @strong mediator,
                                  @strong mediator_msg => move|r|{
                                      let f = if let Ok(f) = r { f } else { return; };
                                      if sno.get_node().is_scene() {
                                          sno.get_node().set_scene_bgimg(
                                              Some(f.path().unwrap()))
                                      } else if sno.get_node().is_mat() || sno.get_node().is_pmat(){
                                          sno.get_node().set_mat_bgimg(
                                              Some(f.path().unwrap()))
                                      } else {
                                          println!("(Isv2FileDialogBox) unsupported node!");
                                          return;
                                      }
                                      file_dialog_entry.set_text(f.path().unwrap().to_str().unwrap());
                                      mediator.upgrade().unwrap()
                                          .emit_by_name::<()>(&mediator_msg, &[&sno]);
                                  }));
                   }));

        //let enable_check = CheckButton::builder().active(sno.get_node().get_scene_bg_en().unwrap()).build();
        let enable_check = CheckButton::new();
        if (sno.get_node().is_scene() &&
            sno.get_node().get_scene_bg_en().unwrap()) ||
            ((sno.get_node().is_mat() || sno.get_node().is_pmat()) &&
             sno.get_node().get_mat_bg_en().unwrap()) {
                enable_check.set_active(true);
            } else {
                enable_check.set_active(false);
            }

        enable_check.connect_toggled(
            clone!(@strong mediator,
                   @strong sno => move|chk|{
                       if sno.get_node().is_scene() {
                           sno.get_node().set_scene_bg_en(chk.is_active());
                       } else if sno.get_node().is_mat() || sno.get_node().is_pmat(){
                           sno.get_node().set_mat_bg_en(chk.is_active());
                       } else {
                           println!("(Isv2FileDialogBox) unsupported node!");
                       }
                       mediator.upgrade().unwrap()
                           .emit_by_name::<()>(&mediator_msg, &[&sno]);
                   }));

        file_dialog_box.append(&file_dialog_label);
        file_dialog_box.append(&enable_check);
        file_dialog_box.append(&file_dialog_entry);
        file_dialog_box.append(&file_dialog_button);

        Self{
            file_dialog_box,
            file_dialog_label,
            file_dialog_entry,
            enable_check,
        }
    }
}
// Isv2CropWindow //////////////////////////////////////////
struct Isv2CropWindow{
    pub editor         : Weak<Isv2CropEditor>,
    pub win            : Window,
    pub vbox           : Box,
    pub button_box     : Box,
    pub ok_button      : Button,
    pub cancel_button  : Button,
    pub cursor         : Rc<RefCell<util::CursorState>>,
    pub crop_temp_rect : Rc<RefCell<Rectangle>>,
    pub pbuf           : RefCell<Option<Pixbuf>>,
    pub scale_pbuf     : RefCell<Option<Pixbuf>>,
    pub area           : DrawingArea,
    pub sno            : ScenarioNodeObject,
    pub transforming   : Cell<bool>,
    pub crop_begin_pos : Cell<(i32, i32)>,
    pub crop_orig_pos  : Cell<(i32, i32)>,
}
impl Isv2CropWindow {
    // crop_win_area_draw ///////////////////////////////
    fn crop_win_area_draw(rect: Rc<RefCell<Rectangle>>,
                          pbuf: Option<Pixbuf>) -> impl FnMut(&DrawingArea, &Context, i32, i32) + 'static{
        move |area, cr, _w, _h|{
            // draw pixbuf
            let pbuf = if let Some(ref p) = pbuf { p.clone() } else { return; };
            let (scale, dst_w, dst_h, ofst_x, ofst_y) =
                util::get_scale_offset(pbuf.width(), pbuf.height(),
                                       area.width(), area.height());
            cr.scale(scale, scale);
            cr.set_source_pixbuf(&pbuf,
                                 ofst_x as f64 / scale, ofst_y as f64 / scale);
            cr.rectangle(ofst_x as f64 / scale, ofst_y as f64 / scale,
                         dst_w  as f64 / scale, dst_h  as f64 / scale);

            cr.fill().expect("fill pixbuf in Isv2CropWindow");
            // draw crop area
            let r = &*rect.borrow();
            let (x, y, w, h) = (r.x(), r.y(), r.width(), r.height());
            let crop_area_width = 1.0 / scale; // TODO parameterize

            cr.set_line_width(crop_area_width);
            cr.set_source_rgba( 0.0, 0.0, 0.0, 1.0 );
            cr.rectangle(x as f64 + (ofst_x as f64) / scale,
                         y as f64 + (ofst_y as f64) / scale,
                         w as f64,
                         h as f64);
            cr.stroke().expect("stroke crop are in Isv2CropWindow");

        }
    }
    // crop_win_motion //////////////////////////////////
    fn crop_win_motion(obj: Rc<Self>) -> EventControllerMotion{
        let motion_ctrl = EventControllerMotion::new();
        motion_ctrl.connect_motion(
            clone!(@strong obj=> move|_e, px, py|{
                       if obj.transforming.get() { return; }

                       let pbuf = if let Some(ref p) = &*obj.pbuf.borrow() { p.clone() } else { return; };
                       let (scale, _, _, ofst_x, ofst_y) =
                           util::get_scale_offset(pbuf.width(), pbuf.height(),
                                                  obj.area.width(), obj.area.height());
                       let px = ((px as f64) / scale) as i32;
                       let py = ((py as f64) / scale) as i32;

                       let r = obj.crop_temp_rect.borrow().clone();
                       let (x, y, w, h) = (r.x(), r.y(), r.width(), r.height());
                       let ax = x as f64 + (ofst_x as f64) / scale;
                       let ay = y as f64 + (ofst_y as f64) / scale;

                       let (_result, cursor, ope) = util::detect_edge(px, py, ax as i32, ay as i32, w, h, scale);
                       *obj.cursor.borrow_mut() = cursor;
                       obj.area.set_cursor_from_name( Some(&ope.as_ref().unwrap().clone()) );
                       return;
                   }));
        motion_ctrl
    }
    // crop_win_gesture ////////////////////////////////////
    fn crop_win_gesture(obj: Rc<Self>) -> GestureClick{
        let gesture_ctrl = GestureClick::new();
        // begin ///////////////////////////////////////////
        gesture_ctrl.connect_begin(
            clone!(@strong obj => move|g, _es|{
                if *obj.cursor.borrow() != util::CursorState::None {
                    let pbuf = if let Some(ref p) = &*obj.pbuf.borrow() { p.clone() } else { return; };
                    obj.transforming.set(true);
                    // save current mouse position
                    let (scale, _,_, ofst_x, ofst_y) =
                        util::get_scale_offset(pbuf.width(), pbuf.height(),
                                               obj.area.width(), obj.area.height());
                    let (px,py) = g.point(None).unwrap();
                    let (px,py) = ((px / scale), (py / scale)); // pre-scale coordinates
                    obj.crop_begin_pos.set((px as i32, py as i32));
                    // save original mat's position in pre-scale coordinates
                    let r = obj.crop_temp_rect.borrow().clone();
                    obj.crop_orig_pos.set( (r.x() + (((ofst_x as f64) / scale) as i32),
                                            r.y() + (((ofst_y as f64) / scale) as i32)) );
                }
            }));
        // update //////////////////////////////////////////
        gesture_ctrl.connect_update(
            clone!(@strong obj => move|g, _es|{
                let pbuf = if let Some(ref p) = &*obj.pbuf.borrow() { p.clone() } else { return; };
                let (scale, _,_, ofst_x, ofst_y) =
                    util::get_scale_offset(pbuf.width(), pbuf.height(),
                                           obj.area.width(), obj.area.height());
                let (px,py) = g.point(None).unwrap();
                let (px,py) = ((px / scale) as i32, (py / scale) as i32); // pre-scale coordinates
                let (ofst_x, ofst_y) = ((ofst_x as f64/ scale) as i32, (ofst_y as f64/ scale) as i32);
                if obj.transforming.get() {
                    let (begin_x, begin_y) = obj.crop_begin_pos.get();
                    let (orig_x,  orig_y ) = obj.crop_orig_pos.get();
                    let r = obj.crop_temp_rect.borrow().clone();
                    let (new_x, new_y, new_w, new_h) =
                        util::gesture_update_rect(r.x() + ofst_x, r.y() + ofst_y,
                                                  r.width(),      r.height(),
                                                  begin_x,        begin_y,
                                                  orig_x,         orig_y,
                                                  px,             py,
                                                  *obj.cursor.borrow());
                    (*obj.crop_temp_rect.borrow_mut()).set_x(new_x - ofst_x);
                    (*obj.crop_temp_rect.borrow_mut()).set_y(new_y - ofst_y);
                    (*obj.crop_temp_rect.borrow_mut()).set_width(new_w);
                    (*obj.crop_temp_rect.borrow_mut()).set_height(new_h);
                    obj.area.queue_draw();
                }
            }));
        // end /////////////////////////////////////////////
        gesture_ctrl.connect_end(
            clone!(@strong obj => move|_g, _es|{
                if obj.transforming.get() {
                    obj.transforming.set(false);
                    *obj.cursor.borrow_mut() = util::CursorState::None;
                    obj.area.set_cursor_from_name( None );
                }
            })
        );

        gesture_ctrl
    }
    // build ///////////////////////////////////////////////
    pub fn build(editor    : Rc<Isv2CropEditor>,
                 sno       : ScenarioNodeObject,
                 parameter : Isv2Parameter) -> Rc<Self>{
        let obj = Rc::new(Self{
            editor         : Rc::downgrade(&editor),
            win            : Window::builder().title( String::from("crop") ).modal(true).build(),
            vbox           : Box::builder().orientation(Orientation::Vertical).build(),
            button_box     : Box::builder().orientation(Orientation::Horizontal).build(),
            ok_button      : Button::builder().css_classes(vec!["isv2_button"]).build(),
            cancel_button  : Button::builder().css_classes(vec!["isv2_button"]).build(),
            cursor         : Rc::new(RefCell::new(util::CursorState::None)),
            crop_temp_rect : Rc::new(RefCell::new(Rectangle::new(0, 0, 0, 0))),
            pbuf           : RefCell::new(None),
            scale_pbuf     : RefCell::new(None),
            area           : DrawingArea::builder().hexpand(true).vexpand(true).build(),
            sno            : sno.clone(),
            transforming   : Cell::new(false),
            crop_begin_pos : Cell::new((0,0)),
            crop_orig_pos  : Cell::new((0,0)),
        });

        obj.button_box.set_halign(Align::End);
        obj.button_box.set_homogeneous(true);

        obj.ok_button.set_label("ok");
        obj.ok_button.set_hexpand(true);
        obj.ok_button.connect_clicked(
            clone!(@weak obj => move|_b|{
                let r = obj.crop_temp_rect.borrow().clone();
                let crop_str = format!("{},{},{},{}", r.x(), r.y(), r.width(), r.height());
                obj.editor.upgrade().unwrap().crop_entry.set_text(&crop_str);
                obj.sno.get_node().set_scene_crop_en(true);
                obj.editor.upgrade().unwrap().crop_check.set_active(true);
                obj.win.close();
            }));

        obj.cancel_button.set_label("cancel");
        obj.cancel_button.set_hexpand(true);
        obj.cancel_button.connect_clicked(
            clone!(@strong obj => move|_b|{ obj.win.close(); }));

        let (sel_x,sel_y,sel_w,sel_h,_) = obj.sno.get_node().get_scene_crop_pos_dim().unwrap();
        *obj.crop_temp_rect.borrow_mut() = // without scale, pixbuf coordinates (without offset)
            Rectangle::new(sel_x,sel_y,sel_w,sel_h);

        obj.button_box.append(&obj.ok_button);
        obj.button_box.append(&obj.cancel_button);

        let mut prj_path = parameter.property::<PathBuf>("project_dir");
        if let Some(bgimg_path) = sno.get_node().get_scene_bgimg(){
            prj_path.push(&bgimg_path);
            if let Ok(p) = Pixbuf::from_file( prj_path ){
                *obj.pbuf.borrow_mut() = Some(p); }
            else {
                *obj.pbuf.borrow_mut() = None; }
        } else {
            *obj.pbuf.borrow_mut() = None;
        }

        obj.area.set_height_request(480);
        obj.area.set_width_request(480);
        // resize //////////////////////////////////////////
        obj.area.connect_resize(clone!(@strong obj => move|area, _w, _h|{
            let pbuf = if let Some(ref p) = &*obj.pbuf.borrow() { p.clone() } else { return; };
            let (_, dst_w, dst_h, _, _) =
                util::get_scale_offset(pbuf.width(), pbuf.height(), area.width(), area.height());
            *obj.scale_pbuf.borrow_mut() = pbuf.scale_simple(dst_w, dst_h, InterpType::Bilinear);
        }));
        // draw_func ///////////////////////////////////////
        obj.area.set_draw_func(
            Self::crop_win_area_draw(obj.crop_temp_rect.clone(),
                                     obj.pbuf.borrow().clone())
        );
        // motion //////////////////////////////////////////
        let motion_ctrl = Self::crop_win_motion(obj.clone());
        obj.area.add_controller(motion_ctrl);
        // gesture /////////////////////////////////////////
        let gesture_ctrl = Self::crop_win_gesture(obj.clone());
        obj.area.add_controller(gesture_ctrl);

        obj.vbox.append(&obj.area);
        obj.vbox.append(&obj.button_box);

        obj.win.set_child(Some(&obj.vbox));
        obj.win.present();

        obj
    }
}
// Isv2CropEditor //////////////////////////////////////////
struct Isv2CropEditor{
    pub crop_box     : Box,
    pub crop_label   : Label,
    pub crop_check   : CheckButton,
    pub crop_entry   : Entry,
    pub crop_button  : Button,
}
impl Isv2CropEditor {
    // build ///////////////////////////////////////////////
    pub fn build(mediator : WeakRef<Object>,
                 sno      : ScenarioNodeObject,
                 parameter: Isv2Parameter) -> Rc<Isv2CropEditor>{

        let (x, y, w, h, _en) = sno.get_node().get_scene_crop_pos_dim().unwrap();
        let crop_str = format!("{},{},{},{}", x, y, w, h);
        let crop_entry = Entry::builder().buffer(&EntryBuffer::new(Some(crop_str))).build();
        crop_entry.connect_changed(
            clone!(@strong mediator,
                   @strong sno => move |ce|{
                       let t = ce.text();
                       let items_s = t.split(',').collect::<Vec<_>>();
                       let pos = items_s.iter().filter_map(|s| s.parse::<i32>().ok()).collect::<Vec<i32>>();
                       if pos.len() != 4 {
                           return; }
                       sno.get_node().set_scene_crop_pos_dim(pos[0], pos[1], pos[2], pos[3]);
                       mediator.upgrade().unwrap()
                           .emit_by_name::<()>("scene-attribute-changed", &[&sno]);
                   }));
        let obj = Rc::new(Self{
            crop_box    : Box::builder().orientation(Orientation::Horizontal).build(),
            crop_label  : Label::new(Some("crop")),
            crop_check  : CheckButton::builder().active(sno.get_node().get_scene_crop_en().unwrap()).build(),
            crop_entry,
            crop_button : Button::builder().css_classes(vec!["isv2_button"]).icon_name("window-new").build(),
        });

        obj.crop_check.connect_toggled(
            clone!(@strong mediator,
                   @strong sno => move|cc|{
                       sno.get_node().set_scene_crop_en(cc.is_active());
                       mediator.upgrade().unwrap()
                           .emit_by_name::<()>("scene-attribute-changed", &[&sno]);
                   }));


        obj.crop_box.append(&obj.crop_label);
        obj.crop_box.append(&obj.crop_check);
        obj.crop_box.append(&obj.crop_entry);
        obj.crop_box.append(&obj.crop_button);

        obj.crop_button.connect_clicked( clone!(@strong obj => move |_b| {
            Isv2CropWindow::build(obj.clone(), sno.clone(), parameter.clone());
        }));

        obj
    }
}
fn set_file_dialog_initial_path(file_dialog: &FileDialog,
                                sno        : &ScenarioNodeObject,
                                parameter  : &Isv2Parameter){
    if let Ok(cur_dir) = std::env::current_dir() {
        file_dialog.set_initial_folder(Some(&gio::File::for_path(&cur_dir)));
    }
    let mut prj_path = parameter.property::<PathBuf>("project_dir");

    if let Some(p) = sno.get_node().get_bgimg() {
        prj_path.push(&p);
    } else {
        println!("(set_file_dialog_initial_path) scene_bgimg is not set yet");
    }

    if let Ok(m) = std::fs::metadata(prj_path.clone()){
        if m.is_file() {
            file_dialog.set_initial_file(Some(&gio::File::for_path(prj_path.clone())));
        } else if m.is_dir() {
            file_dialog.set_initial_folder(Some(&gio::File::for_path(prj_path.clone())));
        }
    }
}
fn build_scene_attribute_box(sno      : ScenarioNodeObject,
                             root     : Root,
                             mediator : WeakRef<Object>,
                             store    : gio::ListStore,
                             temp_box : &Box,
                             parameter: Isv2Parameter) -> Widget{
    // bgimg ///////////////////////////////////////////////
    let bgimg_box = Isv2FileDialogBox::build(root.clone(),
                                             sno.clone(),
                                             mediator.clone(),
                                             parameter.clone(),
                                             "scene-attribute-changed".to_string());

    // bg color //////////////////////////////////////////
    // Isv2ColorBox will emits "mat-attribute-changed" to mediator,
    // but in this case "scene-attribute-changed" should be emitted.
    // Although, "fill background" process is executed in a path of
    // "mat-attribute-changed" -> queue_draw -> set_draw_func,
    // then the background color is updated expectedly.
    // TODO: add an argument of message type to Isv2ColorBox::build
    let bg_color_box = Rc::new(Isv2ColorBox::build( root.clone(),
                                                    "bg color",
                                                    mediator.clone(),
                                                    store.clone(),
                                                    sno.clone(),
                                                    &sno.get_node().get_scene_bg_rgba().unwrap(),
                                                    |s, c| { s.get_node().set_scene_bg_rgba(c) } ));
    // crop editor /////////////////////////////////////////
    let crop_editor = Isv2CropEditor::build( mediator.clone(),
                                             sno.clone(),
                                             parameter.clone());
    // label ///////////////////////////////////////////////
    let gray_list = vec![bgimg_box.file_dialog_label.upcast_ref::<Widget>().clone(),
                         bgimg_box.file_dialog_entry.upcast_ref::<Widget>().clone(),
                         bgimg_box.enable_check.upcast_ref::<Widget>().clone(),
                         bg_color_box.color_label.upcast_ref::<Widget>().clone(),
                         bg_color_box.color_entry.upcast_ref::<Widget>().clone(),
                         crop_editor.crop_label.upcast_ref::<Widget>().clone(),
                         crop_editor.crop_entry.upcast_ref::<Widget>().clone()];
    let (label_box, focus_tag) = label_type_select_box(sno.clone(),
                                                       gray_list.clone(),
                                                       Vec::<Widget>::new(),
                                                       store.clone(),
                                                       mediator.clone(),
                                                       "scene-attribute-changed".to_string());


    // initial gray out accoding to label
    if sno.get_node().get_label_type() == Some(LabelType::Ref) {
        set_css_grayout(gray_list, true);
    }

    // apply to temp_box
    temp_box.append(&label_box);
    temp_box.append(&bgimg_box.file_dialog_box);
    temp_box.append(&bg_color_box.get_box());
    temp_box.append(&crop_editor.crop_box);

    focus_tag
}
// build_page_attribute_box ////////////////////////////////
fn build_page_attribute_box (p       : &scenario_node::Page,
                             store   : gio::ListStore,
                             sno     : ScenarioNodeObject,
                             temp_box: &Box){
    let page_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .build();
    let page_label = Label::new(Some("page"));
    let page_entry = Entry::builder()
        .vexpand(false)
        .valign(Align::Start)
        .build();

    page_entry.set_buffer( &EntryBuffer::builder()
                            .text(GString::from_string_checked(p.name.clone()).expect("page name is expected") ).build() );

    page_entry.connect_changed(glib::clone!(@strong store, @strong sno => move |e| {
        *sno.get_node().value.borrow_mut() =
            scenario_node::Item::Page( scenario_node::Page{ name: String::from(e.buffer().text().as_str()) });
        store.items_changed(sno.get_seq() as u32, 1, 1);
    }));

    page_box.append(&page_label);
    page_box.append(&page_entry);

    temp_box.append(&page_box);
}
// change_item_type /////////////////////////////////////////
fn change_item_type(b: ScenarioNodeAttributeBox, s: SingleSelection){
    if b.imp().contents_box.borrow().is_some() {
        b.remove( &b.imp().contents_box.borrow_mut().clone().unwrap() );
    }

    let temp_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    let (sno, store) =
        if let Some((a,b)) = selection_to_sno(&s) { (a,b) } else {
            let label = Label::new(Some("no item is selected"));
            temp_box.append(&label);
            b.append(&temp_box);
            *b.imp().contents_box.borrow_mut() = Some(temp_box.clone());
            return;};

    temp_box.add_css_class("narrow_box"); // for debug

    *b.imp().sno.borrow_mut() = Some( sno.clone() );

    match *sno.get_node().value.borrow(){
        scenario_node::Item::Page(ref p) => {
            build_page_attribute_box(p, store, sno, &temp_box);
            *b.imp().focus_tag.borrow_mut() = None;
        },
        scenario_node::Item::Mat(ref _m) | scenario_node::Item::Pmat(ref _m)=> {
            let focus_tag = build_mat_attribute_box(&b,
                                                    b.root().unwrap(),
                                                    b.imp().mediator.borrow().clone(),
                                                    store,
                                                    sno,
                                                    &temp_box,
                                                    b.imp().parameter.borrow().clone().unwrap());
            *b.imp().focus_tag.borrow_mut() = Some(focus_tag);
        },
        scenario_node::Item::Scene(_) => {
            let focus_tag = build_scene_attribute_box(sno,
                                                      b.root().unwrap(),
                                                      b.imp().mediator.borrow().clone(),
                                                      store,
                                                      &temp_box,
                                                      b.imp().parameter.borrow().clone().unwrap());
            *b.imp().focus_tag.borrow_mut() = Some(focus_tag);
        }
        _ => ()
    }

    b.append(&temp_box);
    *b.imp().contents_box.borrow_mut() = Some(temp_box.clone());
}
// ScenarioNodeAttributeBox ////////////////////////////////
impl ScenarioNodeAttributeBox {
    // set_parameter ///////////////////////////////////////
    pub fn set_parameter(&self, p: Option<Isv2Parameter>){ *self.imp().parameter.borrow_mut() = p; }
    // new /////////////////////////////////////////////////
    pub fn new() -> Self {
        let obj: ScenarioNodeAttributeBox = Object::builder().build();
        // sno-selected ////////////////////////////////////
        obj.connect_closure(
            "sno-selected",
            false,
            closure_local!(|b: Self, s: SingleSelection| {
                change_item_type(b, s);
            }),
        );
        // unset-sno ///////////////////////////////////////
        obj.connect_closure(
            "unset-sno",
            false,
            closure_local!(|b: Self, s: SingleSelection| {
                change_item_type(b, s);
            }),
        );
        // sno-move-resize /////////////////////////////////
        obj.connect_closure(
            "sno-move-resize",
            false,
            closure_local!(|b: Self, s: ScenarioNodeObject| {
                // "mat-attribute-changed" message will be sent to the mediator
                // by the changed-closure of posdim_entry is not necessary,
                // but there is no disadvantage other than performance, so this version allows it.
                if Rc::ptr_eq( &s.get_node(), &b.imp().sno.borrow().as_ref().unwrap().get_node()  ) {
                    if let Some(entry) = b.imp().mat_posdim_entry.upgrade() {
                        let (x,y,w,h) = s.get_node().get_mat_pos_dim().unwrap();
                        let posdim_str = format!("{},{},{},{}", x, y, w, h);
                        entry.set_text( &posdim_str );
                    }
                }
            }));

        obj
    }
    pub fn update_item_type(&self, s: SingleSelection){
        change_item_type(self.clone(), s);
    }
    pub fn set_mediator(&self, m: WeakRef<Object>){ *self.imp().mediator.borrow_mut() = m; }
    pub fn get_focus_tag(&self)->Option<Widget>{
        if let Some(w) = self.imp().focus_tag.borrow().as_ref(){
            Some(w.clone())
        } else {
            None
        }
    }
}

impl Default for ScenarioNodeAttributeBox {
    fn default() -> Self {
        Self::new()
    }
}
