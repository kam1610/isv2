mod drawing_util;
mod file_menu;
mod pref_menu;
mod isv2_button;
mod isv2_mediator;
mod isv2_parameter;
mod operation_history;
mod preview_window;
mod scenario_item_drag_object;
mod scenario_node;
mod scenario_node_attribute_box;
mod scenario_node_button_box;
mod scenario_node_object;
mod scenario_text_view;
mod sno_list;
mod tree_util;
mod view_menu;
mod status_bar;
mod text_edit_util;
mod keybind;

use std::path::PathBuf;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::env;
use std::rc::Rc;

use gtk::Application;
use gtk::ApplicationWindow;
use gtk::Box;
use gtk::Button;
use gtk::CssProvider;
use gtk::DropTarget;
use gtk::ListItem;
use gtk::ListView;
use gtk::Orientation;
use gtk::Paned;
use gtk::PolicyType;
use gtk::ScrolledWindow;
use gtk::SingleSelection;
use gtk::TreeListModel;
use gtk::gdk::Display;
use gtk::gdk::DragAction;
use gtk::gio::ListModel;
use gtk::gio::Menu;
use gtk::gio::MenuItem;
use gtk::gio;
use gtk::glib::object::Object;
use gtk::glib;
use gtk::prelude::*;

use crate::file_menu::actions;
use crate::isv2_button::Isv2Button;
use crate::isv2_mediator::Isv2Mediator;
use crate::isv2_parameter::Isv2Parameter;
use crate::operation_history::Operation;
use crate::operation_history::OperationHistory;
use crate::operation_history::OperationHistoryItem;
use crate::pref_menu::pref_actions;
use crate::preview_window::PreviewWindow;
use crate::scenario_node::BranchType;
use crate::scenario_node::Color;
use crate::scenario_node::Dimension;
use crate::scenario_node::LabelType;
use crate::scenario_node::Position;
use crate::scenario_node::{Scene, Mat};
use crate::scenario_node_attribute_box::ScenarioNodeAttributeBox;
use crate::scenario_node_button_box::ScenarioNodeButtonBox;
use crate::scenario_node_object::ScenarioNodeObject;
use crate::scenario_node_object::remove_node;
use crate::scenario_text_view::ScenarioTextView;
use crate::sno_list::selection_to_sno;
use crate::tree_util::tree_manipulate;
use crate::view_menu::view_actions;
use crate::status_bar::StatusBar;
use crate::text_edit_util::text_edit;
use crate::keybind::KeyBind;

// load_css ////////////////////////////////////////////////
pub fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
// my_creator //////////////////////////////////////////////
fn my_creator(obj: &Object) -> Option<ListModel>{

    let sn= obj.downcast_ref::<ScenarioNodeObject>().expect("ScenarioNodeObject is expected");
    if let Some(c) = (*sn.get_node().child.borrow_mut()).as_ref() {
        let model = gio::ListStore::with_type(ScenarioNodeObject::static_type());

        tree_manipulate::append_neighbors(&model, c.clone(), 0);

        Some(model.into())
    } else {
        None
    }

}

// build_ui ////////////////////////////////////////////////
pub fn build_ui(app: &Application) {

    let mediator = Isv2Mediator::new();

    let param = Isv2Parameter::new();
    param.set_property("target_width",  1024);
    param.set_property("target_height",  768);
    mediator.set_property("parameter", param.clone());

    let model = gio::ListStore::with_type(ScenarioNodeObject::static_type());

    let o_node1 = ScenarioNodeObject::new_with_seq_id(0, 1);
    let scene1  = Scene::default();
    *o_node1.get_node().value.borrow_mut() = scenario_node::Item::Scene(scene1);

    tree_manipulate::append_neighbors( &model, o_node1.get_node(), 0);

    let tree_list_model = TreeListModel::new(model.clone(),
                                             false /* passthrough */,
                                             true  /* auto expand */,
                                             my_creator);
    let tree_list_model_2 = tree_list_model.clone();

    let selection_model = SingleSelection::builder().autoselect(true).model(&tree_list_model_2).build();


    let history = OperationHistory::default();
    let history = Rc::new(history);

    let list_view = sno_list::build_tree_list_view(tree_list_model.clone(),
                                                   selection_model.clone(),
                                                   history.clone());
    mediator.set_property("list_view", list_view.clone());

    history.set_list_view(list_view.clone());

    let drop_target= DropTarget::new( ListItem::static_type(), DragAction::COPY);
    drop_target.connect_drop(|d, v, x, y|{
        println!("dropped! d:{:?}, dv:{:?}, v:{:?}, x:{:?}, y:{:?}",

                 //d.widget().downcast::<ListView>().expect("lview is expected").model().unwrap(),
                 d.widget().downcast::<ListView>().expect("lview is expected").first_child(),

                 d.value(), v, x, y);
        true
    });
    list_view.add_controller(drop_target);

    //list_view.set_enable_rubberband(true);
    list_view.set_show_separators(true);

    let scrolled_window = ScrolledWindow::builder()
        //.min_content_height(480)
        //.min_content_width(20)
        .hscrollbar_policy(PolicyType::Always)
        .vscrollbar_policy(PolicyType::Always)
        .hexpand(true).vexpand(true)
        .overlay_scrolling(false)
        .child(&list_view)
        .build();

    // undo ////////////////////////////////////////////////
    let undo_button = Isv2Button::with_label_selection_history("undo",
                                                               selection_model.clone(),
                                                               history.clone());
    undo_button.connect_clicked(move |a| {
        a.get_history().undo();
    });
    // redo ////////////////////////////////////////////////
    let redo_button = Isv2Button::with_label_selection_history("redo",
                                                               selection_model.clone(),
                                                               history.clone());
    redo_button.connect_clicked(move |a| {
        a.get_history().redo();
    });
    // update //////////////////////////////////////////////
    let update_button = Button::with_label("update");
    update_button.add_css_class("isv2_button");
    update_button.connect_clicked( |button| {
        let list_view= button.parent().unwrap()
            .prev_sibling().unwrap()
            .first_child().unwrap()
            .downcast::<ListView>().expect("ListView");
        let list_model= list_view.model().unwrap() // SelectionModel
            .downcast::<SingleSelection>().expect("SingleSelection")
            .model().unwrap() // TreeListModel
            .downcast::<TreeListModel>().expect("TreeListModel")
            .model();          // ListModel
        for i in 0..list_model.n_items() {
            list_model.items_changed(i, 1, 1);
        }

    });
    // dump ////////////////////////////////////////////////
    let dump_button = Button::with_label("dump"); // just for debug
    dump_button.add_css_class("isv2_button");
    dump_button.connect_clicked( glib::clone!( @strong list_view => move |_| {
        let obj= list_view.model().unwrap() // SelectionModel
            .downcast::<SingleSelection>().expect("SingleSelection")
            .model().unwrap() // TreeListModel
            .downcast::<TreeListModel>().expect("TreeListModel")
            .model()          // ListModel
            .item(0);         // Object<ScenarioNodeObject>
        let sno= obj.unwrap().downcast_ref::<ScenarioNodeObject>().expect("sno").get_node();
        println!("--------------------");
        sno.dump(0);
        list_view.set_model( Some( &list_view.model().unwrap() ) );
        list_view.queue_draw();

        // 再描画のサンプル，性能はわからないがとりあえず期待通り動作する
        // TODO list_model を move せずに， connect_clicked の引数から生成する

        let list_model= list_view.model().unwrap() // SelectionModel
            .downcast::<SingleSelection>().expect("SingleSelection")
            .model().unwrap() // TreeListModel
            .downcast::<TreeListModel>().expect("TreeListModel")
            .model();          // ListModel
        for i in 0..list_model.n_items() {
            list_model.items_changed(i, 1, 1);
        }

    }));

    // node add buttons ////////////////////////////////////
    let scenario_node_button_box = ScenarioNodeButtonBox::new(selection_model.clone(),
                                                              history.clone());
    mediator.set_property("node_add_box", scenario_node_button_box.clone());

    ////////////////////////////////////////////////////////
    let scenario_list_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let attribute_box = ScenarioNodeAttributeBox::new();
    mediator.set_property("attr_box", attribute_box.clone());
    attribute_box.set_parameter(Some(param.clone()));
    attribute_box.set_mediator(mediator.clone().upcast::<Object>().downgrade());

    selection_model.connect_selected_notify(glib::clone!(@strong mediator => move |s| {
        mediator.emit_by_name::<()>("sno-selected", &[&s]);
    }));

    // remove //////////////////////////////////////////////
    let remove_button = Isv2Button::with_label_selection_history("rm",
                                                                 selection_model.clone(),
                                                                 history.clone());
    remove_button.connect_clicked(glib::clone!(@strong mediator,
                                               @strong selection_model => move |a| {
        if let Ok(hdl) = tree_manipulate::isv2button_to_dest_member4(a){
            let h= OperationHistoryItem::new_from_handle(Operation::Remove, hdl);
            a.get_history().push(h.clone());
            remove_node(h.src.store.unwrap().as_ref(), h.src.sno.unwrap().as_ref());
            mediator.emit_by_name::<()>("sno-selected", &[&selection_model]);
        } else {
            println!("empty!");
        }
    }));

    scenario_list_box.append(&scrolled_window);

    let button_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .build();
    button_box.append(&undo_button);
    button_box.append(&redo_button);
    button_box.append(&dump_button);
    button_box.append(&update_button);
    button_box.append(&scenario_node_button_box);
    button_box.append(&remove_button);

    scenario_list_box.append(&button_box);
    scenario_list_box.set_width_request(button_box.width());

    let pane   = Paned::builder().wide_handle(true).orientation(Orientation::Horizontal).build();
    let pane_l = Paned::builder().wide_handle(true).orientation(Orientation::Vertical).build();
    let pane_r = Paned::builder().wide_handle(true).orientation(Orientation::Vertical).build();

    let preview_window = PreviewWindow::new();
    mediator.set_property("preview_window", preview_window.clone());
    preview_window.set_mediator(mediator.clone().upcast::<Object>().downgrade());
    preview_window.set_parameter(param.clone().downgrade());

    let text_view = ScenarioTextView::new();
    let text_scroll_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Always)
        .vscrollbar_policy(PolicyType::Always)
        .hexpand(true).vexpand(true)
        .overlay_scrolling(true)
        .child(&text_view)
        .build();
    text_view.set_mediator(mediator.clone().upcast::<Object>().downgrade());
    preview_window.set_sno( o_node1.clone() );
    mediator.set_property("scenario_text_view", text_view.clone());

    pane_l.set_start_child( Some( &preview_window) );
    pane_l.set_end_child( Some( &text_scroll_window) );

    pane_l.set_resize_start_child(true);
    pane_l.set_shrink_start_child(false);
    pane_l.set_resize_end_child(false);
    pane_l.set_shrink_end_child(false);

    let attribute_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .hexpand(true).vexpand(true)
        .overlay_scrolling(false)
        .child(&attribute_box)
        .build();
    pane_r.set_start_child( Some( &attribute_window) );

    pane_r.set_end_child( Some( &scenario_list_box) );
    pane_r.connect_position_notify(
        glib::clone!(@strong button_box, @strong scrolled_window => move |p|{
            if (p.height() - p.position()) <= (button_box.height() + scrolled_window.height()) {
                p.set_shrink_end_child(false);
            } else {
                p.set_shrink_end_child(true);
            }
        }));

    pane.set_start_child( Some( &pane_l) );
    pane.set_end_child( Some( &pane_r) );

    pane.set_resize_end_child(false);
    pane.connect_position_notify( glib::clone!(@strong pane_r, @strong button_box => move |p|{
        if pane_r.width() <= button_box.width() {
            p.set_shrink_end_child(false);
        } else {
            p.set_shrink_end_child(true);
        }
    }));

    // vbox(pane + statusbar)
    let sbar = StatusBar::build();
    preview_window.set_status_bar(Some(sbar.clone()));

    let vbox = Box::builder().orientation(Orientation::Vertical).build();
    vbox.append(&pane);
    vbox.append(&sbar.entry);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title( String::from("isv2") )
        .default_width(600)
        .default_height(800)
        .child(&vbox)
        .build();

    window.connect_default_width_notify( glib::clone!(@strong button_box,
                                                      @strong pane => move |_w|{
        if pane.width() < button_box.width() {
            pane.set_width_request( button_box.width() );
        }
    }));

    ////////////////////////////////////////////////////////
    // util for assign menu item and shortcut
    fn assign_acti32_and_accelkey(acts       : &Vec<(&str, &str, i32, &str)>,
                                  parent_menu: &Menu,
                                  app        : &Application){
        let _ = acts.iter()
            .map(|act|{
                let menu_act = MenuItem::new(Some(act.0),
                                             Some(&("win.".to_string() +
                                                    act.1 +
                                                    "(" + &(act.2).to_string() + ")")));
                parent_menu.append_item(&menu_act);
                app.set_accels_for_action(&("win.".to_string() + act.1 +
                                            "(" + &(act.2).to_string() + ")"),
                                          &[act.3]);
            }).collect::<Vec<_>>();
    }

    // menu ////////////////////////////////////////////////
    let menu      = Menu::new();

    ////////////////////////////////////////////////////////
    // file menu ///////////////////////////////////////////
    let menu_file = Menu::new();
    menu.append_submenu(Some("_File"), &menu_file);
    // save as /////////////////////////////////////////////
    let act_save_as = actions::act_file_save_as(model.clone(),
                                                param.clone(),
                                                window.clone());
    app.add_action(&act_save_as);
    let menu_item_save_as = MenuItem::new(Some("_Save as"),
                                          Some( &("app.".to_string() + actions::ACT_FILE_SAVE_AS) ));
    menu_file.append_item(&menu_item_save_as);
    // open ////////////////////////////////////////////////
    let act_open = actions::act_file_open(model.clone(),
                                          param.clone(),
                                          mediator.clone(),
                                          selection_model.clone(),
                                          window.clone());
    app.add_action(&act_open);
    let menu_item_open = MenuItem::new(Some("_Open"),
                                       Some( &("app.".to_string() + actions::ACT_FILE_OPEN) ));
    menu_file.append_item(&menu_item_open);
    // export //////////////////////////////////////////////
    let act_export_img = actions::act_export_img(model.clone(),
                                                 param.clone(),
                                                 preview_window.clone(),
                                                 window.clone());
    app.add_action(&act_export_img);
    let menu_item_export_img = MenuItem::new(Some("_Export images"),
                                             Some( &("app.".to_string() + actions::ACT_FILE_EXPORT_IMG) ));
    menu_file.append_item(&menu_item_export_img);
    ////////////////////////////////////////////////////////
    // view menu ///////////////////////////////////////////
    let menu_node_view = Menu::new();
    menu.append_submenu(Some("_View"), &menu_node_view);
    // view comands ////////////////////////////////////////
    let select_tree_node = view_actions::act_tree_node_sel(selection_model.clone());
    window.add_action(&select_tree_node);

    //   {key value in keybind.conf, action name(String), arg(i32)}
    let node_view_acts = vec![
        ("FwdNode",       view_actions::ACT_TREE_NODE_SEL,  Some(view_actions::ActTreeNodeSelCmd::FwdNode  as i32)),
        ("BackNode",      view_actions::ACT_TREE_NODE_SEL,  Some(view_actions::ActTreeNodeSelCmd::BackNode as i32)),
        ("FwdPage",       view_actions::ACT_TREE_NODE_SEL,  Some(view_actions::ActTreeNodeSelCmd::FwdPage  as i32)),
        ("BackPage",      view_actions::ACT_TREE_NODE_SEL,  Some(view_actions::ActTreeNodeSelCmd::BackPage as i32)),
        ("CollapseNode",  view_actions::ACT_TREE_NODE_SEL,  Some(view_actions::ActTreeNodeSelCmd::Collapse as i32)),
        ("ExpandNode",    view_actions::ACT_TREE_NODE_SEL,  Some(view_actions::ActTreeNodeSelCmd::Expand   as i32)),];
    let keybind_conf = KeyBind::init();
    keybind_conf.assign_acti32_and_accelkey(&node_view_acts,
                                            &menu_node_view,
                                            &app,
                                            "win.");

    let act_close_all_page = view_actions::act_close_all_page(selection_model.clone());
    window.add_action(&act_close_all_page);
    let act_close_all_scene = view_actions::act_close_all_scene(selection_model.clone());
    window.add_action(&act_close_all_scene);

    let node_view_acts2 = vec![
        ("CloseAllPage",   view_actions::ACT_CLOSE_ALL_PAGE,  None),
        ("CloseAllScene",  view_actions::ACT_CLOSE_ALL_SCENE, None)];
    keybind_conf.assign_acti32_and_accelkey(&node_view_acts2,
                                            &menu_node_view,
                                            &app,
                                            "win.");

    // toggle_bgimg ////////////////////////////////////
    let act_toggle_bgimg = view_actions::act_toggle_bgimg(param.clone(), mediator.clone(), selection_model.clone());
    window.add_action(&act_toggle_bgimg);
    keybind_conf.assign_acti32_and_accelkey(&vec![("ToggleBgimg", view_actions::ACT_TOGGLE_BGIMG, None)],
                                            &menu_node_view,
                                            &app,
                                            "win.");

    ////////////////////////////////////////////////////////
    // full screen
    let act_full_screen_preview = view_actions::act_preview(mediator.clone(),
                                                            param.clone(),
                                                            selection_model.clone());
    app.add_action(&act_full_screen_preview);
    let full_screen_item = MenuItem::new(Some("full screen preview"),
                                         Some( &("app.".to_string() + view_actions::ACT_PREVIEW)));
    menu_node_view.append_item(&full_screen_item);

    ////////////////////////////////////////////////////////
    // preferences menu ////////////////////////////////////
    let menu_pref = Menu::new();
    menu.append_submenu(Some("Preferences"), &menu_pref);
    // edit_preference /////////////////////////////////////
    let act_edit_pref = pref_actions::act_open_pref_menu(param.clone(),
                                                         mediator.clone(),
                                                         selection_model.clone());
    app.add_action(&act_edit_pref);
    let menu_item_edit_pref = MenuItem::new(Some("_EditPreferences"),
                                            Some( &("app.".to_string() + pref_actions::ACT_EDIT_PREF) ));
    menu_pref.append_item(&menu_item_edit_pref);
    ////////////////////////////////////////////////////////
    // set menubar /////////////////////////////////////////
    app.set_menubar(Some(&menu));

    ////////////////////////////////////////////////////////
    // menu tree edit //////////////////////////////////////
    let menu_tree_edit = Menu::new();
    menu.append_submenu(Some("Edit"), &menu_tree_edit);
    // add_tree_node ///////////////////////////////////////
    let add_tree_node = tree_manipulate::act_tree_node_add(selection_model.clone(),
                                                           history.clone());
    window.add_action(&add_tree_node);

    let tree_manipulate_acts = vec![
        ("AddTreeNodeGroup", tree_manipulate::ACT_TREE_NODE_ADD, Some(tree_manipulate::ActTreeNodeAddCmd::Group as i32)),
        ("AddTreeNodeScene", tree_manipulate::ACT_TREE_NODE_ADD, Some(tree_manipulate::ActTreeNodeAddCmd::Scene as i32)),
        ("AddTreeNodePage",  tree_manipulate::ACT_TREE_NODE_ADD, Some(tree_manipulate::ActTreeNodeAddCmd::Page  as i32)),
        ("AddTreeNodeMat",   tree_manipulate::ACT_TREE_NODE_ADD, Some(tree_manipulate::ActTreeNodeAddCmd::Mat   as i32)),
        ("AddTreeNodePmat",  tree_manipulate::ACT_TREE_NODE_ADD, Some(tree_manipulate::ActTreeNodeAddCmd::Pmat  as i32)),];
    keybind_conf.assign_acti32_and_accelkey(&tree_manipulate_acts,
                                            &menu_tree_edit,
                                            &app,
                                            "win.");

    // rm_tree_node ////////////////////////////////////////
    let rm_tree_node = tree_manipulate::act_tree_node_rm(selection_model.clone(),
                                                         history.clone(),
                                                         mediator.clone());
    window.add_action(&rm_tree_node);
    keybind_conf.assign_acti32_and_accelkey(&vec![("RemoveTreeNode", tree_manipulate::ACT_TREE_NODE_RM, None)],
                                            &menu_tree_edit,
                                            &app,
                                            "win.");

    ////////////////////////////////////////////////////////
    // menu text edit //////////////////////////////////////
    let menu_text_edit = Menu::new();
    menu.append_submenu(Some("TextView"), &menu_text_edit);
    // text view commands //////////////////////////////////
    let text_move_cursor   = text_edit::act_cursor_move(window.clone());
    window.add_action(&text_move_cursor);

    let text_delete_action = text_edit::act_delete_text(window.clone());
    window.add_action(&text_delete_action);

    let text_insert_action = text_edit::act_insert_text(window.clone());
    window.add_action(&text_insert_action);

    let text_c_n_p_action  = text_edit::act_c_n_p_text(window.clone());
    window.add_action(&text_c_n_p_action);

    let text_edit_acts = vec![
        ("forward char",         text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::FwdChar       as i32, "<Alt>semicolon"),
        ("backward char",        text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::BackChar      as i32, "<Alt>J"),
        ("forward word",         text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::FwdWord       as i32, "<Alt>o"),
        ("backword word",        text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::BackWord      as i32, "<Alt>i"),
        ("next line",            text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::NextLine      as i32, "<Alt>k"),
        ("prev line",            text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::PrevLine      as i32, "<Alt>l"),
        ("next line 3",          text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::NextLine3     as i32, "<Alt>9"),
        ("prev line 3",          text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::PrevLine3     as i32, "<Alt>0"),
        ("beginning line",       text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::BegLine       as i32, "<Ctrl>a"),
        ("end line",             text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::EndLine       as i32, "<Ctrl>e"),
        ("beginning buffer",     text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::BegBuff       as i32, "<Alt>less"),
        ("end buffer",           text_edit::ACT_CURSOR_MOVE, text_edit::ActCursorCmd::EndBuff       as i32, "<Alt>greater"),
        ("delete backward char", text_edit::ACT_DEL_TEXT,    text_edit::ActDelTextCmd::DelBackChar  as i32, "<Ctrl>h"),
        ("delete char",          text_edit::ACT_DEL_TEXT,    text_edit::ActDelTextCmd::DelChar      as i32, "<Ctrl>d"),
        ("kill line",            text_edit::ACT_DEL_TEXT,    text_edit::ActDelTextCmd::KillLine     as i32, "<Ctrl>k"),
        ("backward kill word",   text_edit::ACT_DEL_TEXT,    text_edit::ActDelTextCmd::BackKillWord as i32, "<Alt>h" ),
        ("kill word",            text_edit::ACT_DEL_TEXT,    text_edit::ActDelTextCmd::KillWord     as i32, "<Alt>d" ),
        ("new line",             text_edit::ACT_INS_TEXT,    text_edit::ActInsTextCmd::NewLine      as i32, "<Ctrl>m"),
        ("open line",            text_edit::ACT_INS_TEXT,    text_edit::ActInsTextCmd::OpenLine     as i32, "<Ctrl>o"),
        ("dakuten",              text_edit::ACT_INS_TEXT,    text_edit::ActInsTextCmd::Dakuten      as i32, "<Ctrl>quoteright"),
        ("copy text",            text_edit::ACT_C_N_P_TEXT,  text_edit::ActCnPTextCmd::Copy         as i32, "<Alt>w"),
        ("cut text",             text_edit::ACT_C_N_P_TEXT,  text_edit::ActCnPTextCmd::Cut          as i32, "<Ctrl>w"),
        ("paste text",           text_edit::ACT_C_N_P_TEXT,  text_edit::ActCnPTextCmd::Paste        as i32, "<Ctrl>y"),];
    assign_acti32_and_accelkey(&text_edit_acts, &menu_text_edit, &app);

    let focus_view_action = view_actions::act_focus_view(text_view.clone(),
                                                         list_view.clone());
    window.add_action(&focus_view_action);

    let focus_attr_box_action = view_actions::act_focus_attrbox(attribute_box.clone());
    window.add_action(&focus_attr_box_action);

    ////////////////////////////////////////////////////////
    // shortcut ////////////////////////////////////////////
    app.set_accels_for_action(&("win.".to_string() + view_actions::ACT_FOCUS_VIEW +
                                "(" + &(view_actions::ActFocusViewCmd::TextView as i32).to_string() + ")"), &["F2"]);
    app.set_accels_for_action(&("win.".to_string() + view_actions::ACT_FOCUS_VIEW +
                                "(" + &(view_actions::ActFocusViewCmd::TreeView as i32).to_string() + ")"), &["F3"]);
    app.set_accels_for_action(&("win.".to_string() + view_actions::ACT_FOCUS_ATTRBOX)                     , &["F4"]);

    app.set_accels_for_action("win.text_forward_char", &["<Alt>semicolon"]);

    // set attribute box after root is associated
    attribute_box.update_item_type(selection_model.clone());

    // Present window
    window.set_show_menubar(true);
    window.present();

    // initial selection state
    if let Some((_,_)) = selection_to_sno(&selection_model) {
        mediator.emit_by_name::<()>("sno-selected", &[&selection_model]);
    }

}

// build_sample_tree ///////////////////////////////////////
pub fn build_sample_tree() -> ScenarioNodeObject{

    let o_node1   = ScenarioNodeObject::new_with_seq_id(0, 1  );
    let mut scene1 = Scene::default();
    scene1.bgimg = Some(PathBuf::from("img/sample.png".to_string()));
    scene1.lbl = Some(String::from("slabel1")); scene1.lbl_type = LabelType::Def;
    scene1.crop_en = true;
    scene1.crop.pos.x =  23; scene1.crop.pos.y =  11;
    scene1.crop.dim.w = 383; scene1.crop.dim.h = 457;
    scene1.bgcol = Color{r: 196, g: 196, b: 196};
    *o_node1.get_node().value.borrow_mut() = scenario_node::Item::Scene(scene1);

    let o_node11  = ScenarioNodeObject::new_with_seq_id(0, 11 ); o_node11.set_parent ( o_node1.get_node()  ); o_node11.set_bt(BranchType::Child);
    o_node1.set_child( o_node11.get_node() );

    let o_node111 = ScenarioNodeObject::new_with_seq_id(0, 111); o_node111.set_parent( o_node11.get_node() ); o_node111.set_bt(BranchType::Child);
    let mut mat111 = Mat::default();
    mat111.name = "mat111".to_string();
    mat111.pos  = Position{ x: 40, y: 40 }; mat111.dim  = Dimension{ w: 40, h: 40 }; mat111.a = 128;
    mat111.lbl  = Some(String::from("mlabel111")); mat111.lbl_type = LabelType::Def;
    mat111.text = "こんにちは！これはサンプルのテキストです．えんいー".to_string();
    *o_node111.get_node().value.borrow_mut() = scenario_node::Item::Mat(mat111);
    o_node11.set_child( o_node111.get_node() );

    let o_node12  = ScenarioNodeObject::new_with_seq_id(0, 12 ); o_node12.set_parent ( o_node11.get_node() ); o_node12.set_bt(BranchType::Neighbor);
    o_node11.set_neighbor( o_node12.get_node() );

    let o_node121 = ScenarioNodeObject::new_with_seq_id(0, 121); o_node121.set_parent( o_node12.get_node() ); o_node121.set_bt(BranchType::Child);
    let mut mat121 = Mat::default();
    mat121.name = "mat121".to_string();
    mat121.pos  = Position{ x: 40, y: 100 }; mat121.dim  = Dimension{ w: 40, h: 40 }; mat121.a = 128;
    mat121.lbl = Some(String::from("mlabel111")); mat121.lbl_type = LabelType::Ref;
    *o_node121.get_node().value.borrow_mut() = scenario_node::Item::Mat(mat121);
    o_node12.set_child( o_node121.get_node() );

    let o_node122 = ScenarioNodeObject::new_with_seq_id(0, 122); o_node122.set_parent( o_node121.get_node()); o_node122.set_bt(BranchType::Neighbor);
    let mut mat122 = Mat::default();
    mat122.name = "mat122".to_string();
    mat122.pos  = Position{ x: 40, y: 160 }; mat122.dim  = Dimension{ w: 40, h: 40 }; mat122.a = 128;
    mat122.lbl = Some(String::from("mlabel21")); mat122.lbl_type = LabelType::Ref;
    *o_node122.get_node().value.borrow_mut() = scenario_node::Item::Mat(mat122);
    o_node121.set_neighbor( o_node122.get_node() );

    let o_node13  = ScenarioNodeObject::new_with_seq_id(0, 13 ); o_node13.set_parent ( o_node12.get_node() ); o_node13.set_bt(BranchType::Neighbor);
    o_node12.set_neighbor( o_node13.get_node() );

    let o_node131 = ScenarioNodeObject::new_with_seq_id(0, 131); o_node131.set_parent( o_node13.get_node() ); o_node131.set_bt(BranchType::Child);
    let mut mat131 = Mat::default();
    mat131.name = "mat131".to_string();
    mat131.pos  = Position{ x: 100, y: 40 }; mat131.dim  = Dimension{ w: 40, h: 40 }; mat131.a = 128;
    *o_node131.get_node().value.borrow_mut() = scenario_node::Item::Mat(mat131);
    o_node13.set_child( o_node131.get_node() );

    let o_node2   = ScenarioNodeObject::new_with_seq_id(0, 2  ); o_node2.set_parent  ( o_node1.get_node()  ); o_node2.set_bt(BranchType::Neighbor);
    let mut scene2 = Scene::default();
    scene2.bgimg = Some(PathBuf::from("img/sample2.png".to_string()));
    scene2.crop_en = true;
    scene2.crop.pos.x =  18; scene2.crop.pos.y =  35;
    scene2.crop.dim.w = 576; scene2.crop.dim.h = 348;
    scene2.bgcol = Color{r: 164, g: 164, b: 164};

    *o_node2.get_node().value.borrow_mut() = scenario_node::Item::Scene(scene2);
    o_node1.set_neighbor( o_node2.get_node() );

    let o_node21  = ScenarioNodeObject::new_with_seq_id(0, 21 ); o_node21.set_parent ( o_node2.get_node()  ); o_node21.set_bt(BranchType::Child);
    let mut mat21 = Mat::default();
    mat21.name = "mat21".to_string();
    mat21.pos  = Position{ x: 100, y: 100 }; mat21.dim  = Dimension{ w: 40, h: 40 }; mat21.a = 128;
    mat21.lbl = Some(String::from("mlabel21")); mat21.lbl_type = LabelType::Def;
    *o_node21.get_node().value.borrow_mut() = scenario_node::Item::Pmat(mat21);
    o_node2.set_child( o_node21.get_node() );

    let o_node22  = ScenarioNodeObject::new_with_seq_id(0, 22 ); o_node22.set_parent ( o_node21.get_node() ); o_node22.set_bt(BranchType::Neighbor);
    let mut mat22 = Mat::default();
    mat22.name = "mat22".to_string();
    mat22.pos  = Position{ x: 100, y: 160 }; mat22.dim  = Dimension{ w: 40, h: 40 }; mat22.a = 128;
    mat22.lbl = Some(String::from("label999")); mat22.lbl_type = LabelType::Ref;
    *o_node22.get_node().value.borrow_mut() = scenario_node::Item::Pmat(mat22);
    o_node21.set_neighbor( o_node22.get_node() );

    let o_node3   = ScenarioNodeObject::new_with_seq_id(0, 3  ); o_node3.set_parent  ( o_node2.get_node()  ); o_node3.set_bt(BranchType::Neighbor);
    let mut scene3 = Scene::default();
    scene3.bgimg = Some(PathBuf::from("img/sample2.png".to_string()));
    scene3.lbl = Some(String::from("slabel1")); scene3.lbl_type = LabelType::Ref;
    scene3.bgcol = Color{r: 164, g: 164, b: 164};
    *o_node3.get_node().value.borrow_mut() = scenario_node::Item::Scene(scene3);
    o_node2.set_neighbor( o_node3.get_node() );

    o_node1
}

// test ////////////////////////////////////////////////////
//extern crate assert_matches;
#[cfg(test)]
#[macro_use]
mod test {
    use super::*;
    use crate::scenario_node::ScenarioNode;
    use crate::scenario_node::Item;
    use crate::scenario_node::ScenarioNodeSerde;

    #[test]
    fn test_scenario_node_dump(){
        let m = Mat::default();
        m.dump();
    }

    #[test]
    fn test_search_labelled_node() {
        let sn_node1 = ScenarioNode::default();
        let sn_node1 = Rc::new( sn_node1 );

        let result = ScenarioNode::search_def_label(sn_node1);
        assert!(result.is_none());

        let root_node: ScenarioNodeObject = build_sample_tree();

        let node11  = root_node.get_node().child.borrow().as_ref().unwrap().clone();
        let node12  = node11.neighbor.borrow().as_ref().unwrap().clone();
        let node121 = node12.child.borrow().as_ref().unwrap().clone();

        let mlabel111 = ScenarioNode::search_def_label(node121);
        assert!( mlabel111.is_some() );
        let mlabel111 = mlabel111.as_ref().unwrap().value.borrow();
        if let Item::Mat(ref m) = &*mlabel111 {
            assert_eq!(m.name, "mat111");
        } else {
            assert!(false);
        }

        let sn_sd = ScenarioNodeSerde::from_sn(root_node.get_node());
        //let sn_sd_vec:ScenarioNodeSerdeVec = sn_sd.into();
        //let sntree_toml = toml::to_string(&sn_sd_vec).unwrap();
        //println!("{}", sntree_toml);

        let sn_sd_json= serde_json::to_string_pretty(&sn_sd).unwrap();
        //println!("{}", sn_sd_vec_json);

        let deserialized: Vec<ScenarioNodeSerde> = serde_json::from_str(&sn_sd_json).unwrap();
        let sn_rev = ScenarioNode::from_serde(deserialized);
        sn_rev.unwrap().dump(0);
    }
}
