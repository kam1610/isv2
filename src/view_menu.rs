pub mod view_actions{
    use gtk::SingleSelection;
    use gtk::TreeListRow;
    use gtk::gio::SimpleAction;
    use gtk::prelude::Cast;
    use gtk::prelude::CastNone;
    use gtk::prelude::ListModelExt;
    use gtk::prelude::ObjectExt;
    use gtk::glib::VariantTy;
    use gtk::prelude::*;

    use crate::isv2_mediator::Isv2Mediator;
    use crate::isv2_parameter::Isv2Parameter;
    use crate::scenario_node::Item;
    use crate::scenario_node_object::ScenarioNodeObject;
    use crate::sno_list::selection_to_sno;

    pub const ACT_FOCUS_VIEW  : &str = "select_text_view";
    #[derive(Debug, Clone, Copy)]
    pub enum ActFocusViewCmd {
        TextView, TreeView,
    }

    pub const ACT_CLOSE_ALL_PAGE   : &str = "view_close_all_page";
    pub const ACT_TOGGLE_BGIMG     : &str = "view_toggle_bgimg";

    pub const ACT_TREE_NODE_SEL : &str = "tree_node_sel";
    #[derive(Debug, Clone, Copy)]
    pub enum ActTreeNodeSelCmd {
        FwdNode,      BackNode,
        FwdNode3,     BackNode3,
        FwdPage,      BackPage,
        Collapse,     Expand,
    }

    // act_focus_view //////////////////////////////////////
    pub fn act_focus_view(text_view: impl WidgetExt,
                          tree_view: impl WidgetExt) -> SimpleAction{
        let act = SimpleAction::new(ACT_FOCUS_VIEW, Some(&VariantTy::INT32));
        act.connect_activate(move|_act, val|{
            let val = val.expect("expect val").get::<i32>().expect("couldn't get i32 val");
                 if val == ActFocusViewCmd::TextView as i32 { text_view.grab_focus(); }
            else if val == ActFocusViewCmd::TreeView as i32 { tree_view.grab_focus(); }
        });
        act
    }
    // select_near_node ////////////////////////////////////
    fn select_near_node(sel: &SingleSelection, num: i32, downward: bool){
        if sel.n_items() < 2 { return; }
        let mut n = sel.selected() as i32;
        if downward { n+= num; } else { n-= num; }

        let lim = (sel.n_items() as i32) - 1;
        if lim < n { n = lim; }
        if n   < 0 { n = 0;   }

        sel.set_selected(n as u32);
    }
    // expand_node /////////////////////////////////////////
    fn expand_node(sel: &SingleSelection, expand: bool){
        if sel.n_items() < 1 { return; }
        let n   = sel.selected() as i32;
        let row = sel.item(n as u32).unwrap().downcast::<TreeListRow>().expect("row");
        row.set_expanded(expand);
    }
    // act_tree_node_sel ///////////////////////////////////
    pub fn act_tree_node_sel(sel: SingleSelection) -> SimpleAction{
        let act = SimpleAction::new(ACT_TREE_NODE_SEL, Some(&VariantTy::INT32));
        act.connect_activate(move|_act, val|{
            let val = val.expect("expect val").get::<i32>().expect("couldn't get i32 val");
                 if val == ActTreeNodeSelCmd::FwdNode   as i32 { select_near_node(&sel, 1, true ); }
            else if val == ActTreeNodeSelCmd::BackNode  as i32 { select_near_node(&sel, 1, false); }
            else if val == ActTreeNodeSelCmd::FwdNode3  as i32 { select_near_node(&sel, 3, true ); }
            else if val == ActTreeNodeSelCmd::BackNode3 as i32 { select_near_node(&sel, 3, false); }
            else if val == ActTreeNodeSelCmd::FwdPage   as i32 { select_near_page(&sel, true ); }
            else if val == ActTreeNodeSelCmd::BackPage  as i32 { select_near_page(&sel, false); }
            else if val == ActTreeNodeSelCmd::Expand    as i32 { expand_node(&sel, true ); }
            else if val == ActTreeNodeSelCmd::Collapse  as i32 { expand_node(&sel, false); }
        });
        act
    }
    // select_near_page ////////////////////////////////
    fn select_near_page(sel : &SingleSelection, downward: bool){
        if sel.n_items() < 2 { return; }
        let mut n = sel.selected() as i32;

        // if group or scene is selected in initial, then open it
        if downward {
            let row = sel.item(n as u32).unwrap().downcast::<TreeListRow>().expect("row");
            let sno = row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");
            if sno.get_node().is_group() || sno.get_node().is_scene(){
                row.set_expanded(true); }
            n+= 1;
        } else {
            n-= 1;
        }

        loop{
            if ((sel.n_items() as i32)-1) < n { break; }
            if n < 0 { break; }

            let row = sel.item(n as u32).unwrap().downcast::<TreeListRow>().expect("row");
            let sno = row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");

            let mut select = false;
            match *sno.get_node().value.borrow() {
                Item::Group   | Item::Scene(_) => { row.set_expanded(true); },
                Item::Page(_) | Item::Pmat(_)  => { select = true; },
                _ => ()
            }
            if select{ sel.set_selected(n as u32); break; }

            if downward {
                n+= 1; }
            else {
                n-= 1; }
        }
    }
    // act_close_all_page //////////////////////////////////
    pub fn act_close_all_page(sel : SingleSelection) -> SimpleAction{
        let act = SimpleAction::new(ACT_CLOSE_ALL_PAGE, None);
        act.connect_activate(move|_act, _val|{
            let mut n = 0;
            loop{

                if ((sel.n_items() as i32)-1) < n { break; }
                let row = sel.item(n as u32).unwrap().downcast::<TreeListRow>().expect("row");
                let sno = row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");

                match &*sno.get_node().value.borrow() {
                    Item::Group    => { row.set_expanded(true ); },
                    Item::Scene(_) => { row.set_expanded(true ); },
                    Item::Page(_)  => { row.set_expanded(false); },
                    _ => ()
                }

                n+=1;
            }
        });

        act
    }
    // act_toggle_bgimg ////////////////////////////////////
    pub fn act_toggle_bgimg(param : Isv2Parameter, mediator: Isv2Mediator, selection: SingleSelection) -> SimpleAction{
        let act = SimpleAction::new(ACT_TOGGLE_BGIMG, None);
        act.connect_activate(move|_act, _val|{
            if param.property::<bool>("bgimg_en") {
                param.set_property("bgimg_en", false); }
            else {
                param.set_property("bgimg_en", true); }
            if let Some((sno,_store)) = selection_to_sno(&selection) {
                mediator.emit_by_name::<()>("scene-attribute-changed", &[&sno]);
            }
        });
        act
    }

}
