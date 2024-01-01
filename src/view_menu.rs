pub mod view_actions{
    use gtk::SingleSelection;
    use gtk::TreeListRow;
    use gtk::gio::SimpleAction;
    use gtk::prelude::Cast;
    use gtk::prelude::CastNone;
    use gtk::prelude::ListModelExt;
    use gtk::prelude::ObjectExt;

    use crate::isv2_mediator::Isv2Mediator;
    use crate::isv2_parameter::Isv2Parameter;
    use crate::scenario_node::Item;
    use crate::scenario_node_object::ScenarioNodeObject;
    use crate::sno_list::selection_to_sno;

    pub const ACT_CLOSE_ALL_PAGE   : &str = "view_close_all_page";
    pub const ACT_SELECT_NEXT_PAGE : &str = "view_select_next_page";
    pub const ACT_SELECT_PREV_PAGE : &str = "view_select_prev_page";
    pub const ACT_TOGGLE_BGIMG     : &str = "view_toggle_bgimg";

    // select_near_page ////////////////////////////////
    fn select_near_page(sel : SingleSelection, downward: bool){
        if sel.n_items() < 1 { return; }
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

    // act_select_prev_page //////////////////////////////////
    pub fn act_select_prev_page(sel : SingleSelection) -> SimpleAction{
        let act = SimpleAction::new(ACT_SELECT_PREV_PAGE, None);
        act.connect_activate(move|_act, _val|{
            select_near_page(sel.clone(), false);
        });
        act
    }
    // act_select_next_page //////////////////////////////////
    pub fn act_select_next_page(sel : SingleSelection) -> SimpleAction{
        let act = SimpleAction::new(ACT_SELECT_NEXT_PAGE, None);
        act.connect_activate(move|_act, _val|{
            select_near_page(sel.clone(), true);
        });
        act
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
