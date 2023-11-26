pub mod pref_actions{

    use std::rc::Rc;

    use gtk::prelude::*;
    use gtk::glib::clone;
    use gtk::gio::SimpleAction;
    use gtk::Orientation;
    use gtk::SingleSelection;
    use gtk::Label;
    use gtk::Window;
    use gtk::Box;
    use gtk::Button;
    use gtk::Grid;
    use gtk::Align;
    use gtk::Entry;

    use crate::isv2_mediator::Isv2Mediator;
    use crate::isv2_parameter::Isv2Parameter;

    pub const ACT_EDIT_PREF   : &str = "edit_pref";

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
        mediator        : Isv2Mediator,
        selection       : SingleSelection,
    }
    impl PrefEditWin{
        // appry_prefs /////////////////////////////////////
        fn apply_prefs(&self) {

        }
        // build ///////////////////////////////////////////
        fn build(param    : Isv2Parameter,
                 mediator : Isv2Mediator,
                 selection: SingleSelection) -> Rc<Self> {
            let win           = Window::builder().title( String::from("preference") ).modal(true).build();
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
                mediator, selection};
            let obj = Rc::new(obj);

            let target_width_label =
                Label::builder().label("target width[1--9999]").halign(Align::End).build();
            obj.grid.attach(&target_width_label, 0, 0, 1, 1);
            obj.grid.attach(&obj.target_width, 1, 0, 1, 1);
            obj.target_width.buffer().set_text( &(param.property::<i32>("target_width").to_string()) );

            let target_height_label =
                Label::builder().label("target height[1--9999]").halign(Align::End).build();
            obj.grid.attach(&target_height_label, 0, 1, 1, 1);
            obj.grid.attach(&obj.target_height, 1, 1, 1, 1);
            obj.target_height.buffer().set_text( &(param.property::<i32>("target_height").to_string()) );

            let export_dir_label =
                Label::builder().label("export dir name").halign(Align::End).build();
            obj.grid.attach(&export_dir_label, 0, 2, 1, 1);
            obj.grid.attach(&obj.export_dir, 1, 2, 1, 1);
            obj.export_dir.buffer().set_text( &(param.property::<String>("export_dir")) );

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
