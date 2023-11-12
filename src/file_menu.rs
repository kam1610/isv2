pub mod actions{

    use std::fs::OpenOptions;
    use std::io::BufReader;
    use std::io;
    use std::path::PathBuf;

    use gtk::FileDialog;
    use gtk::FileFilter;
    use gtk::SingleSelection;
    use gtk::Window;
    use gtk::gio;
    use gtk::gio::Cancellable;
    use gtk::gio::File;
    use gtk::gio::ListStore;
    use gtk::gio::SimpleAction;
    use gtk::glib::error::Error;
    use gtk::glib::variant::Variant;
    use gtk::prelude::*;

    use serde::{Deserialize, Serialize};
    use serde_json::ser::Formatter;

    use crate::isv2_mediator::Isv2Mediator;
    use crate::isv2_parameter::Isv2Parameter;
    use crate::isv2_parameter::Isv2ParameterSerde;
    use crate::preview_window::PreviewWindow;
    use crate::scenario_node::ScenarioNode;
    use crate::scenario_node::ScenarioNodeSerde;
    use crate::scenario_node_object::ScenarioNodeObject;
    use crate::tree_util::tree_manipulate;

    pub const ACT_FILE_SAVE_AS    : &str = "file_save_as";
    pub const ACT_FILE_OPEN       : &str = "file_open";
    pub const ACT_FILE_EXPORT_IMG : &str = "file_export_img";

    // formatter ///////////////////////////////////////////
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Isv2FormatterState{ BeginObjectKey, EndObjectKey }
    #[derive(Clone, Debug)]
    pub struct Isv2Formatter {
        state         : Isv2FormatterState,
        key           : String,
    }
    impl Isv2Formatter {
        pub fn new() -> Self {
            Isv2Formatter {
                state         : Isv2FormatterState::EndObjectKey,
                key           : "".to_string(),
            }
        }
    }
    impl Default for Isv2Formatter {
        fn default() -> Self {
            Isv2Formatter::new() } }
    impl Formatter for Isv2Formatter {
        #[inline]
        fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where W: ?Sized + io::Write, {
            if first {
                Ok(()) }
            else {
                writer.write_all(b",\n") }
        }
        #[inline]
        fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where W: ?Sized + io::Write, {
            self.state = Isv2FormatterState::BeginObjectKey;
            if first {
                Ok(()) }
            else if (self.key == "vertical") || (self.key == "text") {
                // this condition strongly depends on the order of the members
                // of scenario_node::Mat.
                // One solution is buffering the output and adjust the line break
                // when the 'text' key appears, but here we'll keep it simple.
                writer.write_all(b",\n ") }
            else {
                writer.write_all(b",") }
        }
        #[inline]
        fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
        where W: ?Sized + io::Write, {
            if self.state == Isv2FormatterState::BeginObjectKey { self.key = fragment.to_string(); }
            writer.write_all(fragment.as_bytes())
        }
        #[inline]
        fn end_object_key<W>(&mut self, _writer: &mut W) -> io::Result<()>
        where W: ?Sized + io::Write, {
            self.state = Isv2FormatterState::EndObjectKey;
            Ok(())
        }
    }
    // ProjectFileSer //////////////////////////////////////
    #[derive(Serialize, Deserialize)]
    struct ProjectFileSerde{
        param_ser : Isv2ParameterSerde,
        sn_ser    : Option<Vec<ScenarioNodeSerde>>
    }
    impl ProjectFileSerde{
        fn from(param_ser : Isv2ParameterSerde,
                sn_ser    : Option<Vec<ScenarioNodeSerde>>) -> Self{
            Self{
                param_ser,
                sn_ser
            }
        }
    }
    // project_file_dialog /////////////////////////////////
    fn project_file_dialog(param: Isv2Parameter) -> FileDialog{
        let file_dialog = FileDialog::builder().modal(true).build();
        if let Ok(cur_dir) = std::env::current_dir() {
            file_dialog.set_initial_folder(Some(&gio::File::for_path(&cur_dir)));
        }
        let param_path = param.property::<PathBuf>("project_dir");
        if param_path.exists() && param_path.is_dir() {
            file_dialog.set_initial_folder(Some(&gio::File::for_path(param_path)));
        }
        file_dialog.set_initial_name(Some(&(param.property::<String>("project_file_name"))));

        let file_filter = FileFilter::new();
        file_filter.add_pattern("*.json");
        file_filter.set_name(Some("json"));
        let model = gio::ListStore::with_type(FileFilter::static_type());
        model.append(&file_filter);
        file_dialog.set_filters(Some(&model));
        file_dialog.set_default_filter(Some(&file_filter));

        file_dialog
    }
    // dialog_save_func ////////////////////////////////////
    fn dialog_save_func(store: ListStore,
                        param: Isv2Parameter) -> Box<dyn FnOnce(Result<File, Error>) + 'static>{
        Box::new( move|result|{
            let     file      = if let Ok(f) = result { f } else { return; };
            let mut path_buf  = if let Some(p) = file.path() { p } else { return; };
            let     file_name = if let Some(f) = path_buf.file_name() { f } else { return; };

            param.set_property("project_file_name", file_name.to_str().unwrap().to_string());
            path_buf.pop();

            // rewrite bgimg path to relative path from project_dir
            let sn= store.item(0);
            let sn_ser;
            if sn.is_some(){ // when skipped if store has no node
                let sn = sn.unwrap().downcast_ref::<ScenarioNodeObject>().expect("sno").get_node();
                ScenarioNode::update_bgimg_relative_path(&sn,
                                                         param.property::<PathBuf>("project_dir"),
                                                         &path_buf);
                sn_ser = Some( ScenarioNodeSerde::from_sn(sn) );
            } else {
                sn_ser = None;
            }
            // update parameter
            param.set_property("project_dir", path_buf);

            // write project.json
            let param_ser = Isv2ParameterSerde::from(&param);
            let prj_ser   = ProjectFileSerde::from(param_ser, sn_ser);
            let out_file  = {
                if let Ok(f) = OpenOptions::new().read(false).write(true).create(true).open(file.path().unwrap()) { f }
                else { return; } };
            let mut isv2ser = serde_json::ser::Serializer::with_formatter(out_file, Isv2Formatter::new());
            if let Err(_) = prj_ser.serialize(&mut isv2ser){
                println!("(dialog_save_func) write failed! {:?}, {:?}", file.path().unwrap(), file);
            }
        })
    }
    // act_save_func ///////////////////////////////////////
    fn act_save_func(store: ListStore,
                     param: Isv2Parameter,
                     pwin : impl IsA<Window>) -> Box<dyn Fn(&SimpleAction, Option<&Variant>) + 'static>{
        Box::new( move|_act, _val|{
            let file_dialog = project_file_dialog(param.clone());
            file_dialog.save(Some(&pwin),
                             None::<Cancellable>.as_ref(),
                             dialog_save_func(store.clone(),
                                              param.clone()));
        })
    }
    // act_file_save ///////////////////////////////////////
    pub fn act_file_save_as(store: ListStore,
                            param: Isv2Parameter,
                            pwin : impl IsA<Window>
    ) -> SimpleAction{
        let act_save_as = SimpleAction::new(ACT_FILE_SAVE_AS, None);
        act_save_as.connect_activate(act_save_func(store.clone(),
                                                   param.clone(),
                                                   pwin.clone()));


        act_save_as
    }
    // dialog_open_func ////////////////////////////////////
    fn dialog_open_func(store     : ListStore,
                        param     : Isv2Parameter,
                        mediator  : Isv2Mediator,
                        selection : SingleSelection
    ) -> Box<dyn FnOnce(Result<File, Error>) + 'static>{
        Box::new( move|result| {
            let     file      = if let Ok(f) = result { f } else { return; };
            let mut path_buf  = if let Some(p) = file.path() { p } else { return; };
            let     file_name = if let Some(f) = path_buf.file_name() { f } else { return; };

            param.set_property("project_file_name", file_name.to_str().unwrap().to_string());
            path_buf.pop();
            param.set_property("project_dir", path_buf);

            let in_file = {
                if let Ok(f) = std::fs::File::open(file.path().unwrap()) { f }
                else { println!("(dialog_open_func) cannnot open file: {:?}", file.path().unwrap()); return; } };
            let reader = BufReader::new(in_file);

            /* ---- serde_json ---- */
            // let prj_ser : ProjectFileSerde = {
            //     if let Ok(p) = serde_json::from_reader(reader) { p }
            //     else { println!("(dialog_open_func) parse failed!"); return; }};
            /* ---- serde_json ---- */

            /* ---- serde_path_to_error ----*/
            let prj_ser = &mut serde_json::Deserializer::from_reader(reader);
            let result: Result<ProjectFileSerde, _> = serde_path_to_error::deserialize(prj_ser);
            if let Err(err) = result {
                let err_path = err.path().to_string();
                println!("(dialog_open_func) parse failed! {:?}", err_path);
                return;
            }
            let prj_ser = result.expect("ProjectFileSerde is expected");
            /* ---------------------------- */

            // deserialize of parameter
            param.copy_from_serde(&prj_ser.param_ser);

            // deserialize of store
            store.remove_all();
            if let Some(sn_tree) = prj_ser.sn_ser {
                if let Some(sn_rev) = ScenarioNode::from_serde(sn_tree){
                    let root_sno = ScenarioNodeObject::new_from(sn_rev);
                    tree_manipulate::append_neighbors( &store, root_sno.get_node(), 0);
                    mediator.emit_by_name::<()>("sno-selected", &[&selection]);
                } else {
                    mediator.emit_by_name::<()>("unset-sno", &[&selection]);
                }
            }
        })
    }
    // act_open_func ///////////////////////////////////////
    fn act_open_func(store     : ListStore,
                     param     : Isv2Parameter,
                     mediator  : Isv2Mediator,
                     selection : SingleSelection,
                     pwin : impl IsA<Window>) -> Box<dyn Fn(&SimpleAction, Option<&Variant>) + 'static>{
        Box::new( move|_act, _val|{
            let file_dialog = project_file_dialog(param.clone());
            file_dialog.open(Some(&pwin),
                             None::<Cancellable>.as_ref(),
                             dialog_open_func(store.clone(),
                                              param.clone(),
                                              mediator.clone(),
                                              selection.clone()));
        })
    }
    // act_file_open ///////////////////////////////////////
    pub fn act_file_open(store     : ListStore,
                         param     : Isv2Parameter,
                         mediator  : Isv2Mediator,
                         selection : SingleSelection,
                         pwin : impl IsA<Window>
    ) -> SimpleAction{
        let act_open = SimpleAction::new(ACT_FILE_OPEN, None);
        act_open.connect_activate(act_open_func(store,
                                                param,
                                                mediator,
                                                selection,
                                                pwin));

        act_open
    }
    // act_export_img //////////////////////////////////////
    pub fn act_export_img(store : ListStore,
                          param : Isv2Parameter,
                          pwin  : PreviewWindow,
                          root  : impl IsA<Window>) -> SimpleAction{
        let act_export_img = SimpleAction::new(ACT_FILE_EXPORT_IMG, None);
        act_export_img.connect_activate(move|_act, _val|{
            let sn = store.item(0);
            if sn.is_some(){
                let sn = sn.unwrap().downcast_ref::<ScenarioNodeObject>().expect("sno").get_node();
                pwin.export_images(&sn, &param, &root);
            } else {
                println!("(act_export_img) store has noitem");
            }

        });
        act_export_img
    }
}
