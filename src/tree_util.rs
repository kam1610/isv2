pub mod tree_manipulate{
    use std::rc::Rc;
    use std::cell::Cell;
    use std::sync::atomic::{AtomicI32, Ordering};

    use gtk::glib::VariantTy;
    use gtk::gio::SimpleAction;
    use gtk::TreeListModel;
    use gtk::TreeListRow;
    use gtk::SingleSelection;
    use gtk::gio::ListStore;
    use gtk::gio;
    use gtk::prelude::*;

    use crate::isv2_button::Isv2Button;
    use crate::operation_history::TreeManipulationHandle;
    use crate::scenario_node::ScenarioNode;
    use crate::scenario_node_object::ScenarioNodeObject;
    use crate::sno_list::get_parent_sno;
    use crate::sno_list::row_to_parent_row;
    use crate::sno_list::row_to_parent_store;
    use crate::operation_history::OperationHistory;
    use crate::tree_util::tree_manipulate;
    use crate::scenario_node;
    use crate::scenario_node::{Scene, Page, Mat, Ovimg};
    use crate::sno_list::selection_to_sno;

    pub const ACT_TREE_NODE_ADD   : &str = "tree_node_add";
    pub const ACT_TREE_NODE_GROUP : &str = "group";
    pub const ACT_TREE_NODE_SCENE : &str = "scene";
    pub const ACT_TREE_NODE_PAGE  : &str = "page";
    pub const ACT_TREE_NODE_MAT   : &str = "mat";
    pub const ACT_TREE_NODE_OVIMG : &str = "ovimg";
    pub const ACT_TREE_NODE_PMAT  : &str = "pmat";

    // act_tree_node_add ///////////////////////////////////
    pub fn act_tree_node_add(sel: SingleSelection, hist: Rc<OperationHistory>) -> SimpleAction{
        let act = SimpleAction::new(ACT_TREE_NODE_ADD, Some(&VariantTy::STRING));
        act.connect_activate(move|_act, val|{
            let val = val
                .expect("expect val")
                .get::<String>()
                .expect("couldn't get &str val");

            // prepare new node ////////////////////////////
            let new_node = ScenarioNodeObject::new_with_seq_id(0, tree_manipulate::gen_id());
            *new_node.get_node().value.borrow_mut() = {
                if      val == ACT_TREE_NODE_GROUP { scenario_node::Item::Group }
                else if val == ACT_TREE_NODE_SCENE { scenario_node::Item::Scene(Scene::default()) }
                else if val == ACT_TREE_NODE_PAGE  { scenario_node::Item::Page(Page::default()) }
                else if val == ACT_TREE_NODE_MAT   { scenario_node::Item::Mat(Mat::default()) }
                else if val == ACT_TREE_NODE_OVIMG { scenario_node::Item::Ovimg(Ovimg::default()) }
                else if val == ACT_TREE_NODE_PMAT  { scenario_node::Item::Pmat(Mat::default()) }
                else { println!("(act_tree_node_add) unexpected condition"); return; }
            };
            // judge the node can be added to selected position
            let (sno, _) =
                if let Some((a,b)) = selection_to_sno(&sel) {
                    (a,b)
                } else {
                    println!("no node is selected");
                    let n = ScenarioNode::new(); // dummy node to indicate Group when empty list
                    n.set_value(scenario_node::Item::Group);    // empty list is treated as a group
                    let sno = ScenarioNodeObject::new_from( Rc::new(n) );
                    (sno, gio::ListStore::with_type(ScenarioNodeObject::static_type()))
                };
            let sno_value = sno.get_node();
            let sno_value = &*sno_value.value.borrow();
            if !ScenarioNode::can_be_neighbor_or_child_auto(sno_value,
                                                            &*new_node.get_node().value.borrow()){
                return;
            }


        });
        act
    }
    // isv2button_to_dest_member ///////////////////////////////
    pub fn isv2button_to_dest_member4(b: &Isv2Button) ->
        Result<TreeManipulationHandle, &'static str> {

            if b.get_selection().selected_item().is_none() {
                return Err("not selected"); }

            let root_store= b.get_selection() // selection is a member of Isv2Button
                .model().unwrap() // TreeListModel
                .downcast::<TreeListModel>().expect("TreeListModel")
                .model()          // ListModel
                .downcast::<gio::ListStore>().expect("ListStore");

            let obj               = b.get_selection().selected_item().unwrap();
            let dest_row          = obj.downcast_ref::<TreeListRow>().expect("TreeListRow is expected");
            let dest_sno          = dest_row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");
            let dest_store        = row_to_parent_store(dest_row, &root_store);

            let dest_parent_row   = row_to_parent_row(dest_row);
            let dest_parent_store = row_to_parent_store(&dest_parent_row, &root_store);
            let dest_parent_sno   = get_parent_sno(&dest_sno, &dest_parent_row, &dest_store);

            let hdl = TreeManipulationHandle{
                bt           : dest_sno.get_bt().into(),
                row          : Some(dest_row.clone().into()),
                sno          : Some(dest_sno.into()),
                store        : Some(dest_store.clone().into()),
                depth        : Cell::new(dest_row.depth()),
                size         : Cell::new(dest_store.n_items()),
                parent_row   : Some(dest_parent_row.clone().into()),
                parent_sno   : Some(dest_parent_sno.into()),
                parent_store : Some(dest_parent_store.into()),
            };
            Ok( hdl )
        }
    // search_sn_upward ////////////////////////////////////////
    pub fn search_row_with_sn_up_in_ssel(single_selection : &SingleSelection,
                                         sn               : Rc<ScenarioNode>,
                                         mut from_n       : u32) -> (Option<TreeListRow>, u32) {
        loop {
            let row      = single_selection.item(from_n).unwrap().downcast::<TreeListRow>().expect("row");
            let dest_sno = row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");
            if Rc::ptr_eq( &dest_sno.get_node(), &sn ) {
                return (Some(row.clone()), from_n); }
            if from_n == 0 { return (None, from_n); }
            from_n -= 1;
        }
    }
    // search_sn_downward //////////////////////////////////////
    pub fn search_row_with_sn_down_in_ssel(single_selection : &SingleSelection,
                                           sn               : Rc<ScenarioNode>,
                                           mut from_n       : u32) -> (Option<TreeListRow>, u32) {
        loop {
            let row      = single_selection.item(from_n).unwrap().downcast::<TreeListRow>().expect("row");
            let dest_sno = row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");
            if Rc::ptr_eq( &dest_sno.get_node(), &sn ) {
                return (Some(row.clone()), from_n); }
            if (from_n as i32) == ((single_selection.n_items() as i32) - 1) { return (None, from_n); }
            from_n += 1;
        }
    }
    // add_node_to_empty_store /////////////////////////////
    pub fn add_node_to_empty_store(a: Isv2Button, sno: &ScenarioNodeObject) {
        a.get_store().insert( 0, sno );
    }
    // gen_id /////////////////////////////////////////////////
    pub fn gen_id() -> i32 {
        static COUNT: AtomicI32 = AtomicI32::new(1000);
        COUNT.fetch_add(1, Ordering::SeqCst)
    }
    // append_neighbors ////////////////////////////////////////
    pub fn append_neighbors(model: &ListStore, sn: Rc<ScenarioNode>, seq: i32){
        let obj= ScenarioNodeObject::new_from(sn.clone());
        obj.set_seq(seq);
        model.append( &obj );
        // println!("(append_neighbors) sn: {}, seq:{}", sn, seq);
        if let Some(nbr) = (*sn.neighbor.borrow_mut()).as_ref(){
            append_neighbors(model, nbr.clone(), seq+1);
        }
    }

}
