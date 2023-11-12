mod imp;

use glib::Object;
use glib::WeakRef;
use glib::closure_local;
use glib::subclass::types::ObjectSubclassIsExt;
use glib_sys;
use gtk::Accessible;
use gtk::AlertDialog;
use gtk::Buildable;
use gtk::ConstraintTarget;
use gtk::EventControllerMotion;
use gtk::GestureClick;
use gtk::SingleSelection;
use gtk::TreeListRow;
use gtk::Window;
use gtk::cairo::Antialias;
use gtk::cairo::Context;
use gtk::cairo::FontOptions;
use gtk::cairo::Format;
use gtk::cairo::ImageSurface;
use gtk::gdk_pixbuf::InterpType;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::pango::FontDescription;
use gtk::pango::Layout;
use gtk::pango::Style;
use gtk::prelude::*;

use std::cell::Cell;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use crate::drawing_util::util::CursorState;
use crate::drawing_util::util;
use crate::isv2_parameter::Isv2Parameter;
use crate::scenario_node::Item;
use crate::scenario_node::LabelType;
use crate::scenario_node::ScenarioNode;
use crate::scenario_node_object::ScenarioNodeObject;


glib::wrapper! {
    pub struct PreviewWindow(ObjectSubclass<imp::PreviewWindow>)
        @extends gtk::DrawingArea, gtk::Widget,
        @implements Accessible, Buildable, ConstraintTarget;
}

// PreviewWindow ///////////////////////////////////////////
impl PreviewWindow {
    pub fn set_mediator(&self, m: WeakRef<Object>){ *self.imp().mediator.borrow_mut() = m; }
    pub fn set_parameter(&self, p: WeakRef<Isv2Parameter>){ *self.imp().parameter.borrow_mut() = p; }
    // draw_func_for_scene /////////////////////////////////
    fn draw_func_for_scene(sn         : &Rc<ScenarioNode>,
                           pbuf       : &Option<Pixbuf>,
                           scale_pbuf : &Option<Pixbuf>,
                           target_w   : i32,
                           target_h   : i32,
                           cr         : &gtk::cairo::Context,
                           param      : Option<Isv2Parameter>
    ){
        if let Some(s) = ScenarioNode::get_belong_scene(sn){
            // resolve label
            let label_sno = {
                if let Some(s_resolved) = ScenarioNode::search_def_label(s.clone()) { s_resolved }
                else { s }}; // resolve ref label

            // fill background
            let c: Vec<_> =
                label_sno.get_scene_bgcol().unwrap().iter().map(|c|{ (*c as f64) / 255.0 }).collect();
            cr.set_source_rgb( c[0], c[1], c[2] );
            cr.rectangle(0.0, 0.0, target_w as f64, target_h as f64);
            cr.fill().expect("fill background on PreviewWindow");

            // draw image
            if !label_sno.get_scene_bg_en().unwrap() { return; }
            if let Some(param) = param {
                if !param.property::<bool>("bgimg_en") { return; } }
            if let Some(pbuf) = pbuf {
                let (_, _, mut crop_w, mut crop_h, crop_en) = label_sno.get_scene_crop_pos_dim().unwrap();
                if !crop_en {
                    crop_w = pbuf.width();
                    crop_h = pbuf.height();
                }
                let (_, crop_target_w, crop_target_h, crop_target_ofst_x, crop_target_ofst_y) =
                    util::get_scale_offset(crop_w, crop_h, target_w, target_h);
                let scale_crop_pixbuf = {
                    if let Some(ref p) = scale_pbuf { p.clone() }
                    else { return; }};

                cr.set_source_pixbuf(&scale_crop_pixbuf,
                                     crop_target_ofst_x as f64, crop_target_ofst_y as f64);
                cr.rectangle(crop_target_ofst_x as f64, crop_target_ofst_y as f64,
                             crop_target_w      as f64, crop_target_h      as f64);
                cr.fill().expect("draw image on PreviewWindow");
            } else {
                println!("pbuf for background has not been prepared!");
            }
        } else {
            println!("belong scene is not found for {:?}!", sn);
        }

    }
    // new /////////////////////////////////////////////////
    pub fn new() -> Self {
        let obj: PreviewWindow = Object::builder().build();
        obj.set_hexpand(true);
        obj.set_vexpand(true);
        // obj.set_content_height(1);
        // obj.set_content_width(1);

        // Gesture /////////////////////////////////////////
        let gesture_ctrl = GestureClick::new();
        gesture_ctrl.connect_begin(|gesture, _|{
            let pwin = gesture.widget().downcast::<PreviewWindow>().expect("preview window");
            if pwin.imp().area_state.get() != CursorState::None {
                pwin.set_transforming(true);
                // save current mouse position
                let (px,py) = gesture.point(None).unwrap();
                let scale = 1.0 / pwin.imp().tgt_to_pwin_scale.get();
                let (px,py) = ((px * scale), (py * scale));
                pwin.imp().begin_point.set((px as i32, py as i32));
                // save original mat's position
                let sn = pwin.imp().target_sn.borrow().as_ref().unwrap().clone();
                if let Some((x, y, _, _)) = sn.get_mat_pos_dim(){
                    pwin.imp().mat_orig_point.set( (x, y) );
                }

                // TODO: prepare history
            }
        });
        gesture_ctrl.connect_update(|gesture, _|{
            let pwin = gesture.widget().downcast::<PreviewWindow>().expect("preview window");
            let scale = 1.0 / pwin.imp().tgt_to_pwin_scale.get();
            let (px,py) = gesture.point(None).unwrap();
            let (px,py) = ((px * scale), (py * scale));
            let px = px as i32;
            let py = py as i32;

            if pwin.get_transforming() {
                let sn = pwin.imp().target_sn.borrow().as_ref().unwrap().clone();
                if let Some((x, y, w, h)) = sn.get_mat_pos_dim(){
                    let (begin_x, begin_y) = pwin.imp().begin_point.get();
                    let (mat_orig_x, mat_orig_y) = pwin.imp().mat_orig_point.get();
                    let (new_x, new_y, new_w, new_h) =
                        util::gesture_update_rect(x, y, w, h,
                                                  begin_x, begin_y,
                                                  mat_orig_x, mat_orig_y,
                                                  px, py,
                                                  pwin.imp().area_state.get());
                    sn.set_mat_pos_dim(new_x, new_y, new_w, new_h);
                    pwin.queue_draw();
                }
            }
        });
        gesture_ctrl.connect_end(|gesture, _|{

            let pwin = gesture.widget().downcast::<PreviewWindow>().expect("preview window");

            if pwin.get_transforming() {

                let sno = ScenarioNodeObject::new_from( pwin.imp().target_sn.borrow().as_ref().unwrap().clone() );
                pwin.imp().mediator.borrow().upgrade().unwrap().emit_by_name::<()>("sno-move-resize", &[&sno]);

                // TODO: history

                pwin.set_transforming(false);
                *pwin.imp().target_sn.borrow_mut() = None;

                // after gesture, if gesture starts before the motion,
                // unwrap target_sn will fail which is set in detect_edge_of_area() called by the motion.
                // therefore those states are reset to prevent condition in connect_begin()
                pwin.imp().area_state.set(CursorState::None);
                pwin.set_cursor_from_name( None );
            }
        });
        obj.add_controller(gesture_ctrl);

        // motion controller ///////////////////////////////
        let motion_ctrl = EventControllerMotion::new();
        motion_ctrl.connect_motion(|a,x,y|{
            let pwin = a.widget().downcast::<PreviewWindow>().expect("preview window");
            pwin.detect_edge_of_area(x as i32, y as i32);
        });
        obj.add_controller(motion_ctrl);

        // resize //////////////////////////////////////////
        obj.connect_resize(|pwin, _w, _h|{
            if let Some(ref sno) = &*pwin.imp().sno.borrow() {

                let scene_node =
                    if let Some(s) = ScenarioNode::get_belong_scene(&sno.get_node()){ s }
                else { println!("(resize) the scene to which this node belongs was not found!"); return; };
                let scene_node = {
                    if let Some(s) = ScenarioNode::search_def_label(scene_node.clone()) { s }
                    else { scene_node }}; // resolve ref label

                pwin.prepare_scale_crop_buf(scene_node);
            }
        });

        // draw func ///////////////////////////////////////
        obj.set_draw_func(move |pwin, cr, _w, _h|{
            let pwin = pwin.clone().downcast::<PreviewWindow>().expect("preview window");

            let param = pwin.imp().parameter.borrow().upgrade().unwrap();
            let target_w = param.property::<i32>("target_width");
            let target_h = param.property::<i32>("target_height");

            // scale from target to pwin
            let (tgt_to_pwin_scale, _, _, _, _) =
               util::get_scale_offset(target_w, target_h, pwin.width(), pwin.height());
            cr.scale(tgt_to_pwin_scale, tgt_to_pwin_scale);
            pwin.imp().tgt_to_pwin_scale.set(tgt_to_pwin_scale);

            let pixbuf = pwin.get_buf();

            if let Some(sn) = pwin.imp().sno.borrow().as_ref() {
                Self::draw_func_for_scene(&sn.get_node(),
                                          &pixbuf,
                                          &*pwin.imp().scale_crop_buf.borrow(),
                                          target_w,
                                          target_h,
                                          cr,
                                          Some(param.clone())
                );
            }
            pwin.draw_mats(cr, _w, _h);
            cr.fill().expect("fill in draw_func on PreviewWindow");
        });

        // signal handler(sno-selected) ////////////////////
        obj.connect_closure(
            "sno-selected",
            false,
            closure_local!(|w: Self, s: SingleSelection| {
                if let Some(row) = s.selected_item() {
                    let tree_list_row = row.downcast::<TreeListRow>().expect("expect row");
                    let sno= tree_list_row.item().unwrap().downcast::<ScenarioNodeObject>().expect("sno");
                    w.set_sno(sno);
                } else {
                    w.unset_sno();
                }
                w.queue_draw();
            }),
        );
        // signal handler(mat-attribute-changed) ///////////
        obj.connect_closure(
            "mat-attribute-changed",
            false,
            closure_local!(|w: Self, sno: ScenarioNodeObject| {
                w.update_mat(sno.clone(), true); // force update
                w.queue_draw();
            }),
        );
        // signal handler(scene-attribute-changed) /////////
        obj.connect_closure(
            "scene-attribute-changed",
            false,
            closure_local!(|w: Self, sno: ScenarioNodeObject| {
                w.update_pixbuf(sno.clone(), true); // force update
                w.queue_draw();
            }),
        );
        // signal handler(unset-sno) ///////////////////////
        obj.connect_closure(
            "unset-sno",
            false,
            closure_local!(|w: Self, _sno: ScenarioNodeObject| {
                w.unset_sno();
                w.queue_draw();
            }),
        );

        obj
    }
    pub fn get_buf(&self) -> Option<Pixbuf>{
        if let Some(ref p) = &*self.imp().buf.borrow() {
            Some( p.clone() )
        } else {
            None
        }
    }
    pub fn set_buf_from_path(&self, path: &PathBuf) {
        println!("(PreviewWindow) loading new pixbuf file: {}", path.to_str().unwrap());
        *self.imp().buf.borrow_mut() = if let Ok(p) = Pixbuf::from_file( path ){
            Some(p)
        } else {
            None
        }
    }
    // prepare_scale_crop_buf //////////////////////////////
    fn prepare_scale_crop_buf_sub(param: &Isv2Parameter,
                                  scene: &Rc<ScenarioNode>,
                                  pbuf : &Pixbuf) -> Option<Pixbuf>{
        let target_w = param.property::<i32>("target_width");
        let target_h = param.property::<i32>("target_height");

        let (mut crop_x, mut crop_y, mut crop_w, mut crop_h, crop_en)
            = scene.get_scene_crop_pos_dim().unwrap();
        if !crop_en {
            crop_x = 0; crop_y = 0;
            crop_w = pbuf.width();
            crop_h = pbuf.height();
        }
        let crop_pbuf = {
            if let Some (p) = Pixbuf::new( pbuf.colorspace(),
                                           true,
                                           pbuf.bits_per_sample(),
                                           crop_w,
                                           crop_h ) { p }
            else { return None; } };
        pbuf.copy_area( crop_x, crop_y, crop_w, crop_h, &crop_pbuf, 0, 0 );
        let (_, crop_target_w, crop_target_h, _, _) =
            util::get_scale_offset(crop_w, crop_h, target_w, target_h);
        let crop_pbuf = crop_pbuf.scale_simple( crop_target_w, crop_target_h, InterpType::Bilinear ).unwrap();

        crop_pbuf.copy()
    }
    pub fn prepare_scale_crop_buf(&self, scene: Rc<ScenarioNode>){
        let pbuf = {
            if let Some(ref p) = *self.imp().buf.borrow() { p.clone() }
            else { return; }};
        *self.imp().scale_crop_buf.borrow_mut() = Self::prepare_scale_crop_buf_sub(
            &(self.imp().parameter.borrow().upgrade().unwrap()),
            &scene,
            &pbuf
        );
        //println!("(prepare_scale_crop_buf) scaled buffer is prepared");
    }
    pub fn set_transforming(&self, t: bool) { self.imp().is_area_transforming.set(t); }
    pub fn get_transforming(&self) -> bool { self.imp().is_area_transforming.get() }
    // confirm_exists_dir
    async fn confirm_exists_dir(root       : impl IsA<Window>,
                                overwrite  : Rc<Cell<bool>>,
                                path_buf   : PathBuf
    ) {
        let dialog = AlertDialog::builder().modal(true).build();
        dialog.set_buttons(&["Overwrite", "Cancel exporting"]);
        dialog.set_message(
            &format!("\"{}\" already exists. If a exporting file exists in the directory, it will be overwritten.",
                     path_buf.to_str().unwrap()));
        dialog.set_default_button(0);
        dialog.set_cancel_button(1);
        let result = dialog.choose_future(Some(&root)).await;
        if let Ok(result) = result {
            if result == 0 {
                overwrite.set(true); }
            else {
                overwrite.set(false); }
        }
    }
    // export_images ///////////////////////////////////////
    pub fn export_images(&self,
                         n     : &Rc<ScenarioNode>,
                         param : &Isv2Parameter,
                         root  : &impl IsA<Window>){

        let target_w = param.property::<i32>("target_width");
        let target_h = param.property::<i32>("target_height");

        // find root
        let mut p = n.clone();
        loop{
            let pp = p.parent.borrow().upgrade();
            if pp.is_none() {
                break; }
            p = pp.unwrap();
        }
        // traverse
        let mut img_seq    = 0;
        let mut pbuf       = None::<Pixbuf>;
        let mut scale_pbuf = None::<Pixbuf>;
        let mut vec        = vec![p.clone()]; // for ScenarioNdoe traversal
        let mut area: Vec<(Rc<ScenarioNode>, Option<Rc<ScenarioNode>>)> = Vec::new();

        // check export dir
        let mut path_buf = param.property::<PathBuf>("project_dir");
        path_buf.push( param.property::<String>("export_dir") );
        // file exists -> show message and abort
        if path_buf.exists() && path_buf.is_file() {
            let dialog = AlertDialog::builder().modal(true).build();
            dialog.set_buttons(&["Cancel exporting"]);
            dialog.set_message(
                &format!("\"{}\" is already exists as file. Please move or remove the file.",
                         path_buf.to_str().unwrap()));
            dialog.show(Some(root));
            println!("(export_images) abort: target dir already exists as file");
            return;
        }
        // dir exists -> show confirm message
        if path_buf.exists() && path_buf.is_dir() {
            let overwrite: Rc<Cell<bool>> = Rc::new(Cell::new(false));
            gtk::glib::MainContext::default().block_on(
                Self::confirm_exists_dir(root.clone(), overwrite.clone(), path_buf.clone()) );
            if !(overwrite.get()) {
                println!("(export_images) abort: target dir already exists");
                return;
            }
        }
        if !path_buf.exists(){
            let mut path_buf = param.property::<PathBuf>("project_dir");
            path_buf.push( param.property::<String>("export_dir") );
            std::fs::create_dir(path_buf).expect("create_dir in export_images");
        }

        loop{
            if let Some(sn) = ScenarioNode::traverse(&mut vec){
                match &*sn.value.borrow() {
                    Item::Scene(_) => { // prepare scaled image
                        let sn_ref = {
                            if let Some(s) = ScenarioNode::search_def_label(sn.clone()) { s }
                            else { sn.clone() }};
                        let bgimg = {
                            if let Some(b) = sn_ref.get_scene_bgimg() { b }
                            else { println!("the node is not scene"); PathBuf::new() } };
                        let mut img_path = param.property::<PathBuf>("project_dir");
                        img_path.push( bgimg );
                        if let Ok(p) = Pixbuf::from_file( img_path ) {
                            pbuf = Some(p); }
                        else {
                            pbuf = None; }
                        if pbuf.is_some(){
                            scale_pbuf = Self::prepare_scale_crop_buf_sub(param,
                                                                          &sn_ref,
                                                                          &pbuf.clone().unwrap()) };
                    },
                    Item::Page(_) | Item::Pmat(_) => {
                        area.clear();
                        Self::collect_mats(&sn, &mut area);
                        // 0. prepare surface
                        let surface = {
                            if let Ok(sf) = ImageSurface::create(Format::ARgb32, target_w, target_h) { sf }
                            else { println!("(export_images) creating curface failed"); return; } };
                        let cr = {
                            if let Ok(ctx) = Context::new(&surface) { ctx }
                            else { println!("(export_images) creating context failed"); return; } };
                        // 1. draw scene
                        Self::draw_func_for_scene(&sn, &pbuf.clone(), &scale_pbuf, target_w, target_h, &cr, None);
                        // 2. draw mats
                        Self::draw_mats_sub(&area, &self.pango_context(), &cr, 0/* w */, 0/* h */);

                        // TODO:ディレクトリがなければ作成する
                        let mut path_buf = param.property::<PathBuf>("project_dir");
                        path_buf.push( param.property::<String>("export_dir") );
                        path_buf.push( format!("{:04}.png", img_seq) );

                        let mut out_file  = {
                            if let Ok(f) = OpenOptions::new()
                                .read(false)
                                .write(true)
                                .create(true)
                                .open(&path_buf) { f }
                            else { println!("(export_images) can not open: {}",
                                            path_buf.to_str().unwrap()); return; } };

                        surface.write_to_png(&mut out_file).expect("write_to_png in export_images");
                        println!("(export_images) {} is written", path_buf.to_str().unwrap());

                        img_seq+= 1;
                    },
                    _ => (),
                }
            } else {
                break;
            }
        }
    }
    // detect_edge_of_area /////////////////////////////////
    pub fn detect_edge_of_area(&self, px: i32, py: i32) {

        let scale = self.imp().tgt_to_pwin_scale.get();
        let px = ((px as f64) * (1.0 / scale)) as i32;
        let py = ((py as f64) * (1.0 / scale)) as i32;

        for area_item in &*self.imp().area.borrow() {

            if self.imp().is_area_transforming.get() {
                return; }

            let (sn_source, sn_ref) = area_item;
            let mut sn = if let Some(ref_target) = sn_ref { ref_target } else { sn_source };
            if sn_source.get_label_type() == Some(LabelType::RefNoRect){ sn = sn_source; }

            if let Some((ax, ay, aw, ah)) = sn.get_mat_pos_dim(){
                let (result, cursor, ope) = util::detect_edge(px, py, ax, ay, aw, ah, scale);
                if result {
                    *self.imp().target_sn.borrow_mut() = Some(sn.clone());
                } else {
                    *self.imp().target_sn.borrow_mut() = None;
                    continue;
                }
                self.imp().area_state.set(cursor);
                self.set_cursor_from_name( Some(&ope.as_ref().unwrap().clone()) );
                return;
            }
        }
        // other
        self.imp().area_state.set(CursorState::None);
        self.set_cursor_from_name( None );
    }
    // draw_mats ///////////////////////////////////////////
    fn draw_mats_sub(area: &Vec<(Rc<ScenarioNode>, Option<Rc<ScenarioNode>>)>,
                     pc  : &gtk::pango::Context,
                     cr  : &Context,
                     _w: i32, _h: i32){
        for area_item in area {
            // mat /////////////////////////////////////////
            let (sn_source, sn_ref) = area_item;
            let sn = if let Some(ref_target) = sn_ref { ref_target } else { sn_source };

            let (mut x, y, w, h) = {
                if sn_source.get_label_type() == Some(LabelType::RefNoRect) {
                    if let Some( tuple )  = sn_source.get_mat_pos_dim_f64() { tuple } else { return; }
                } else {
                    if let Some( tuple )  = sn.get_mat_pos_dim_f64() { tuple } else { return; }
                }
            };

            let (r, g, b, a) =
                if let Some( tuple )  = sn.get_mat_rgba_tuple_f64() { tuple } else { return; };
            cr.set_line_width(2.0); // TODO parameterize
            cr.set_source_rgba( r, g, b, a );

            let round = sn.get_mat_r().unwrap();
            if round <= 0 {
                cr.rectangle(x, y, w, h);
            } else {
                let round = round as f64;
                let m_pi = glib_sys::G_PI;
                cr.move_to(x-round,   y);
                cr.arc    (x,     y,        round,  1.0*m_pi,  1.5*m_pi);
                cr.line_to(x+w,   y-round);
                cr.arc    (x+w,   y,        round,  1.5*m_pi,  2.0*m_pi);
                cr.line_to(x+w+round, y+h+r);
                cr.arc    (x+w,   y+h,      round,  0.0*m_pi,  0.5*m_pi);
                cr.line_to(x,     y+h+round);
                cr.arc    (x,     y+h,      round,  0.5*m_pi,  1.0*m_pi);
                cr.close_path();
            }

            cr.fill().expect("fill draw_mats_sub");
            // text ////////////////////////////////////////
            let layout = Layout::new(pc);

            if sn.get_mat_vertical().unwrap() {
                layout.set_width ((h as i32) * pango::SCALE);     // swap when
                layout.set_height((w as i32) * 3 * pango::SCALE); // vertical writing
            } else {
                layout.set_width ((w as i32) * pango::SCALE);
                layout.set_height((h as i32) * 3 * pango::SCALE);
            }
            // note: Expand the height (width in vertical writing)
            // to avoid unintentional omission of display.
            // This is useful when line-spacing is set to 1 or less.
            // The expanded width (x3) is provisional.

            layout.set_text(&sn_source.get_mat_text().unwrap());

            // line spacing
            layout.set_line_spacing(sn.get_mat_line_spacing().unwrap());

            cr.set_source_rgba( r, g, b, a );
            let mut fopt = FontOptions::new().expect("fopt");
            fopt.set_antialias(Antialias::Good);
            pangocairo::context_set_font_options(pc, Some(&fopt));

            let mut font_desc= FontDescription::new();
            font_desc.set_family( &sn.get_mat_font_family().unwrap() );
            font_desc.set_style( Style::Normal );
            font_desc.set_size( sn.get_mat_font_size().unwrap() * pango::SCALE);
            if sn.get_mat_vertical().unwrap() {
                font_desc.set_gravity(gtk::pango::Gravity::East); }

            let w1 = util::StrumWeight::from_str( &(sn.get_mat_font_weight().unwrap()) ).expect("weight expression");
            font_desc.set_weight(w1.0);

            layout.set_font_description(Some(&font_desc));

            // text mat
            if sn.get_mat_vertical().unwrap() {
                x = x + w; }
            cr.set_line_join(cairo::LineJoin::Round);

            cr.save().expect("save context before stroke text");
            cr.move_to(x, y);
            if sn.get_mat_vertical().unwrap() {
                cr.rotate(0.5 * glib_sys::G_PI); }
            pangocairo::layout_path(cr, &layout);
            let (r, g, b, a) =
                if let Some( tuple )  = sn.get_mat_font_rgba_tuple_f64_2() { tuple } else { return; };
            cr.set_source_rgba( r, g, b, a );
            let font_outl_2 =
                if let Some( w )  = sn.get_mat_font_outl_2() { w } else { return; };
            cr.set_line_width(font_outl_2);
            cr.stroke().expect("stroke text");
            cr.restore().expect("restore context");

            // text foreground
            cr.move_to(x, y);
            if sn.get_mat_vertical().unwrap() {
                cr.rotate(0.5 * glib_sys::G_PI); }
            let (r, g, b, a) =
                if let Some( tuple )  = sn.get_mat_font_rgba_tuple_f64() { tuple } else { return; };
            cr.set_source_rgba( r, g, b, a );
            pangocairo::show_layout(cr, &layout);
        }

    }
    pub fn draw_mats(&self, cr: &Context, _w: i32, _h: i32){
        Self::draw_mats_sub(&*self.imp().area.borrow(),
                            &self.pango_context(),
                            cr, _w, _h);
    }
    // update_pixbuf ///////////////////////////////////////
    fn update_pixbuf(&self, sno: ScenarioNodeObject, force_update: bool) -> bool{
        let scene_node =
            if let Some(s) = ScenarioNode::get_belong_scene(&sno.get_node()){ s }
            else { println!("the scene to which this node belongs was not found!"); return false; };
        let scene_node = {
            if let Some(s) = ScenarioNode::search_def_label(scene_node.clone()) { s }
            else { scene_node }}; // resolve ref label

        if let Some(current_sno) = self.imp().sno.borrow().as_ref() {
            let current_scene_sn =
                if let Some(c) = ScenarioNode::get_belong_scene(&current_sno.get_node()){ c }
            else { println!("the scene to which this node belongs was not found!"); return false; };

            let current_scene_sn = {
                if let Some(s) = ScenarioNode::search_def_label(current_scene_sn.clone()) { s }
                else { current_scene_sn }};

            if Rc::ptr_eq( &current_scene_sn, &scene_node ) && !force_update{
                println!("the same scene is detected");
                return false;
            }
        }
        // handles bgimage /////////////////////////////////
        let mut prj_path = (&*self.imp().parameter.borrow()).upgrade().unwrap().property::<PathBuf>("project_dir");
        let bgimg = {
            if let Some(b) = scene_node.get_scene_bgimg() { b }
            else { println!("the node is not scene");
                   *self.imp().buf.borrow_mut() = None;
                   return true; } };
        prj_path.push(&bgimg);
        self.set_buf_from_path(&prj_path);
        self.prepare_scale_crop_buf(scene_node.clone());
        true
    }
    // update_mat //////////////////////////////////////////
    fn collect_mats(page_node : &Rc<ScenarioNode>,
                    area      : &mut Vec::<(Rc<ScenarioNode>, Option<Rc<ScenarioNode>>)>) {
        let mut p = page_node.clone();
        loop {
            // - it collects mat or pmat
            // - While collecting mat, pmat will not be mixed
            //    (guaranteed by tree manipulation constraints)
            let p1;
            match *p.value.borrow() {
                Item::Page(_) => {
                    p1 = p.child.borrow().clone();
                }
                Item::Mat(_) => {
                    let lbl_ref_node = ScenarioNode::search_def_label(p.clone()); // reference if it has label-ref
                    area.push( (p.clone(), lbl_ref_node) );
                    p1 = p.neighbor.borrow().clone();
                }
                Item::Pmat(_) => {
                    let lbl_ref_node = ScenarioNode::search_def_label(p.clone()); // reference if it has label-ref
                    area.push( (p.clone(), lbl_ref_node) );
                    p1 = None; /* exit when Pmat */
                }
                _ => { p1 = None; }
            }
            if p1.is_some() { /* child or neighbor is set if exists */
                p = p1.unwrap();
            } else {
                break;
            }
        }
    }
    pub fn update_mat(&self, sno: ScenarioNodeObject, force_update: bool) -> bool{
        let page_node =
            if let Some(p) = ScenarioNode::get_belong_page(&sno.get_node()) { p } // detects page or pmat
            else { println!("the page to which new node belongs was not found!"); return false; };

        if let Some(current_page) = self.imp().sno.borrow().as_ref(){
            if let Some(current_page_sn) = ScenarioNode::get_belong_page(&current_page.get_node()){
                if Rc::ptr_eq( &current_page_sn, &page_node) && !force_update {
                    return false;
                }
            }
        }
        // handles mat /////////////////////////////////////
        self.imp().area.borrow_mut().clear();
        Self::collect_mats(&page_node, &mut(*self.imp().area.borrow_mut()));
        true
    }
    // clear_mat_on_scene_node /////////////////////////////
    fn clear_mat_on_scene_node(&self, sno: ScenarioNodeObject) -> bool{
        if let Item::Scene(_) = *sno.get_node().value.borrow(){
            self.imp().area.borrow_mut().clear();
            true
        } else {
            false
        }
    }
    // set_sno /////////////////////////////////////////////
    pub fn set_sno(&self, sno: ScenarioNodeObject){
        let res1 = self.update_pixbuf(sno.clone(), false);
        let res2 = self.update_mat(sno.clone(), false);
        let res3 = self.clear_mat_on_scene_node(sno.clone());
        if  res1 || res2 || res3{
            self.queue_draw(); }
        *self.imp().sno.borrow_mut() = Some(sno);
    }
    // unset_sno ///////////////////////////////////////////
    pub fn unset_sno(&self){
        self.imp().area.borrow_mut().clear();
        *self.imp().buf.borrow_mut() = None;
        *self.imp().scale_crop_buf.borrow_mut() = None;
    }
}
