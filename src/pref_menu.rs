pub mod pref_actions{

    use std::rc::Rc;
    use std::path::Prefix::Verbatim;
    use std::ffi::OsStr;

    use gtk::prelude::*;
    use gtk::glib::clone;
    use gtk::gio::SimpleAction;
    use gtk::EventControllerKey;
    use gtk::Orientation;
    use gtk::SingleSelection;
    use gtk::Label;
    use gtk::Window;
    use gtk::Box;
    use gtk::Button;
    use gtk::Grid;
    use gtk::Align;
    use gtk::Entry;
    use gtk::glib::signal::Propagation;

    use crate::isv2_mediator::Isv2Mediator;
    use crate::isv2_parameter::Isv2Parameter;
    use crate::sno_list::selection_to_sno;

    pub const ACT_EDIT_PREF   : &str = "edit_pref";

    // valid_export_dir_name ///////////////////////////////
    pub fn valid_export_dir_name(path_str: &str) -> bool {
        if (path_str == "") ||
            (path_str == ".") || (path_str == ".."){
                return false; }

        if !Verbatim(OsStr::new(path_str)).is_verbatim() ||
            path_str.contains(std::path::MAIN_SEPARATOR) {
            return false;
        } else {
            return true;
        }
    }

    // PrefEditWin /////////////////////////////////////////
    struct PrefEditWin{
        win             : Window,
        vbox            : Box,
        grid            : Grid,
        target_width    : Entry,
        target_height   : Entry,
        export_dir      : Entry,
        button_box      : Box,
        ok_button       : Button,
        cancel_button   : Button,
        param           : Isv2Parameter,
        mediator        : Isv2Mediator,
        selection       : SingleSelection,
    }
    impl PrefEditWin{
        // pref_editor_key_ctrl ////////////////////////////
        fn pref_editor_key_ctrl(obj: Rc<Self>) -> EventControllerKey {
            let kctrl = EventControllerKey::new();
            kctrl.connect_key_pressed(
                move|_ctrl, key, _code, _state|{
                    println!("key={}", key.name().unwrap().as_str());
                    let mut prop = Propagation::Stop;
                    match key.name().unwrap().as_str() {
                        "Escape" => { obj.win.close(); },
                        "Return" => { obj.apply_prefs(); },
                        _        => { prop = Propagation::Proceed; }
                    }
                    prop
                });
            kctrl
        }
        // appry_prefs /////////////////////////////////////
        fn apply_prefs(&self) {
            let target_width = {
                if let Ok(w) = self.target_width.buffer().text().parse::<i32>(){
                    w }
                else {
                    return; }
            };
            if (target_width < 1) || (9999 < target_width) {
                return; }

            let target_height = {
                if let Ok(h) = self.target_height.buffer().text().parse::<i32>(){
                    h }
                else {
                    return; }
            };
            if (target_height < 1) || (9999 < target_height) {
                return; }

            let export_dir = self.export_dir.buffer().text();
            if !valid_export_dir_name(&export_dir){
                return; }

            self.param.set_property("target_width",  target_width);
            self.param.set_property("target_height", target_height);
            self.param.set_property("export_dir",    export_dir);

            if let Some((sno,_store)) = selection_to_sno(&self.selection) {
                self.mediator.emit_by_name::<()>("scene-attribute-changed", &[&sno]);
            }

            self.win.close();
        }
        // build ///////////////////////////////////////////
        fn build(param    : Isv2Parameter,
                 mediator : Isv2Mediator,
                 selection: SingleSelection) -> Rc<Self> {
            let win           = Window::builder().title( String::from("preferences") ).modal(true).build();
            let vbox          = Box::builder().orientation(Orientation::Vertical).build();
            let grid          = Grid::builder().build();
            let target_width  = Entry::new();
            let target_height = Entry::new();
            let export_dir    = Entry::new();
            let button_box    = Box::builder().orientation(Orientation::Horizontal).build();
            let ok_button     = Button::builder().css_classes(vec!["isv2_button"]).build();
            let cancel_button = Button::builder().css_classes(vec!["isv2_button"]).build();

            // properties //////////////////////////////////
            let obj = Self{
                win, vbox, grid,
                target_width, target_height, export_dir,
                button_box, ok_button, cancel_button,
                param, mediator, selection};
            let obj = Rc::new(obj);

            let target_width_label =
                Label::builder().label("target width[1--9999]").halign(Align::End).build();
            obj.grid.attach(&target_width_label, 0, 0, 1, 1);
            obj.grid.attach(&obj.target_width, 1, 0, 1, 1);
            obj.target_width.buffer().set_text( &(obj.param.property::<i32>("target_width").to_string()) );

            let target_height_label =
                Label::builder().label("target height[1--9999]").halign(Align::End).build();
            obj.grid.attach(&target_height_label, 0, 1, 1, 1);
            obj.grid.attach(&obj.target_height, 1, 1, 1, 1);
            obj.target_height.buffer().set_text( &(obj.param.property::<i32>("target_height").to_string()) );

            let export_dir_label =
                Label::builder().label("export dir name").halign(Align::End).build();
            obj.grid.attach(&export_dir_label, 0, 2, 1, 1);
            obj.grid.attach(&obj.export_dir, 1, 2, 1, 1);
            obj.export_dir.buffer().set_text( &(obj.param.property::<String>("export_dir")) );

            // buttons /////////////////////////////////////
            obj.button_box.set_halign(Align::End);
            obj.button_box.set_homogeneous(true);

            obj.ok_button.set_label("ok");
            obj.ok_button.set_hexpand(true);
            obj.ok_button.connect_clicked(
                clone!(@strong obj => move|_b|{
                    obj.apply_prefs();
                }));

            obj.cancel_button.set_label("cancel");
            obj.cancel_button.set_hexpand(true);
            obj.cancel_button.connect_clicked(
                clone!(@strong obj => move|_b|{ obj.win.close(); }));

            obj.button_box.append(&obj.ok_button);
            obj.button_box.append(&obj.cancel_button);

            obj.vbox.append(&obj.grid);
            obj.vbox.append(&obj.button_box);

            // keycontroller ///////////////////////////////
            let kctrl_for_entry = clone!(@strong obj => move|_e:&Entry|{obj.apply_prefs();});
            obj.target_width.connect_activate(kctrl_for_entry.clone());
            obj.target_height.connect_activate(kctrl_for_entry.clone());
            obj.export_dir.connect_activate(kctrl_for_entry.clone());

            obj.win.add_controller(Self::pref_editor_key_ctrl(obj.clone()));

            obj.win.set_child(Some(&obj.vbox));
            obj.win.present();

            obj
        }
    }
    // act_open_pref_menu //////////////////////////////////
    pub fn act_open_pref_menu(param    : Isv2Parameter,
                              mediator : Isv2Mediator,
                              selection: SingleSelection) -> SimpleAction {
        let act_edit_pref = SimpleAction::new(ACT_EDIT_PREF, None);
        act_edit_pref.connect_activate(move|_act, _val|{
            PrefEditWin::build(param.clone(), mediator.clone(), selection.clone());
        });
        act_edit_pref
    }

}

#[cfg(test)]
mod tests {
    use crate::pref_actions::valid_export_dir_name;

    #[test]
    fn test_valid_export_dir_name() {
        assert_eq!(false, valid_export_dir_name(""));
        assert_eq!(true,  valid_export_dir_name("a"));
        assert_eq!(false, valid_export_dir_name("."));
        assert_eq!(false, valid_export_dir_name("/"));
        assert_eq!(false, valid_export_dir_name("a/b"));
    }
}
