use std::cell::Cell;
use std::rc::Rc;
use std::vec::Vec;

use gio::ListModel;
use glib::clone;
use gtk::DragSource;
use gtk::DropTarget;
use gtk::Label;
use gtk::ListItem;
use gtk::ListScrollFlags;
use gtk::ListView;
use gtk::SignalListItemFactory;
use gtk::SingleSelection;
use gtk::TreeExpander;
use gtk::TreeListModel;
use gtk::TreeListRow;
use gtk::Widget;
use gtk::gdk::ContentProvider;
use gtk::gdk::DragAction;
use gtk::gio;
use gtk::glib::Value;
use gtk::glib;
use gtk::prelude::Cast;
use gtk::prelude::CastNone;
use gtk::prelude::EventControllerExt;
use gtk::prelude::ListItemExt;
use gtk::prelude::ListModelExt;
use gtk::prelude::StaticType;
use gtk::prelude::WidgetExt;

use crate::operation_history::Operation;
use crate::operation_history::OperationHistory;
use crate::operation_history::OperationHistoryItem;
use crate::operation_history::TreeManipulationHandle;
use crate::scenario_item_drag_object::ScenarioItemDragObject;
use crate::scenario_node::BranchType;
use crate::scenario_node::ScenarioNode;
use crate::scenario_node_object::ScenarioNodeObject;
use crate::scenario_node_object::adj_seq;

// selection_to_sno ////////////////////////////////////////
pub fn selection_to_sno(s: &SingleSelection) -> Option<(ScenarioNodeObject, gio::ListStore)> {
    let obj = if let Some(i) = s.selected_item() { i } else { return None; };
    let row = obj.downcast_ref::<TreeListRow>().expect("TreeListRow is expected");
    let sno = row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");
    let root_store = s.model().unwrap() // TreeListModel
        .downcast::<TreeListModel>().expect("TreeListModel")
        .model()          // ListModel
        .downcast::<gio::ListStore>().expect("ListStore");
    let store = row_to_parent_store(row, &root_store);

    Some((sno, store))
}

// get_belong_store ////////////////////////////////////////
pub fn get_belong_model(lv: ListView, /* root list view */
                        sn: Rc<ScenarioNode>,
                        force_expand: bool) -> (Option<ListModel>, u32) {

    // 1. traces depth  from child-node
    let mut depth_stack = Vec::new();
    let mut p = sn.clone();
    let mut depth = 0;
    loop {
        let p1 = p.parent.borrow().clone();
        if p1.upgrade().is_some() {
            if p.bt.get() == BranchType::Child {
                depth_stack.push(depth);
                depth = 0;
            } else {
                depth += 1;
            }
            p = p1.upgrade().unwrap();
        } else {
            depth_stack.push(depth);
            break;
        }
    }

    // 2. loop for stack
    let mut model = lv.model().unwrap().downcast::<SingleSelection>().expect("singleselection").model().unwrap(); // listmodel
    let mut depth = depth_stack.pop().unwrap();
    let mut row = model.clone().downcast::<TreeListModel>().expect("treelistmodel").child_row(depth).unwrap();
    loop {
        // if last one, search succeeded
        if depth_stack.is_empty() {
            return (Some(model.upcast::<ListModel>()), depth); }

        if row.is_expandable() && !row.is_expanded() && force_expand {
            row.set_expanded(true); }

        if row.is_expanded(){
            model = row.children().unwrap();
            depth = depth_stack.pop().unwrap();
            row = row.child_row(depth).unwrap();
        } else {
            return (None, 0);
        }
    }
}

// get_parent_sno //////////////////////////////////////////
pub fn get_parent_sno(sno: &ScenarioNodeObject,
                  parent_row: &TreeListRow,
                  store: &gio::ListStore) -> ScenarioNodeObject {
    if sno.get_bt() == BranchType::Child {
        parent_row.item().and_downcast::<ScenarioNodeObject>().expect("sno is expd").clone()
    } else {
        let parent_item = store.item( sno.get_seq() as u32 - 1).unwrap();
        parent_item
            .downcast_ref::<ScenarioNodeObject>().unwrap().clone()
    }
}

// row_to_parent_store /////////////////////////////////////
pub fn row_to_parent_store(row: &TreeListRow, root: &gio::ListStore) -> gio::ListStore {
    if row.depth() > 0 {
        row.parent().unwrap()
            .children().unwrap() // ListModel
            .downcast::<gio::ListStore>().expect("ListStore")
    } else {
        root.clone()
    }
}

// row_to_parent_row ///////////////////////////////////////
pub fn row_to_parent_row(r: &TreeListRow) -> TreeListRow {
    if r.depth() == 0 { // root-root
        r.clone()
    } else { // (Child &&, depth > 0) or neighbor
        r.parent().unwrap()
    }
}

// detect_descendant ///////////////////////////////////////
fn detect_descendant(parent: &TreeListRow, child: &TreeListRow) -> bool {
    if parent == child {
        return true; }
    if child.parent().is_none() {
        return false; }
    return detect_descendant(parent, &child.parent().unwrap());
}

// expander_to_store ///////////////////////////////////////
// TODO: 空になった直後のexpanderを開こうとするとクラッシュする
fn expander_to_store(e: &TreeExpander, depth: u32) -> gio::ListStore {
    if depth == 0 {
        e.parent().unwrap()
            .parent().and_downcast::<ListView>().expect("ListView is expected")
            .model().unwrap() // SelectionModel
            .downcast::<SingleSelection>().expect("SingleSelection")
            .model().unwrap() // TreeListModel
            .downcast::<TreeListModel>().expect("TreeListModel")
            .model()          // ListModel
            .downcast::<gio::ListStore>().expect("ListStore")
    } else {
        e.list_row().unwrap()
            .parent().unwrap()   // depthが0の場合はこのunwrapが失敗するので，parent...経由ででListViewを取得する
            .children().unwrap() // ListModel
            .downcast::<gio::ListStore>().expect("ListStore")
    }
}

// Operation when a label is dropped to Label / Expander
//
// | drop       | bt of    | drop area                         |
// | target     | dest     |------------------+----------------|
// |            |          | upper half       | lower half     |
// |------------+----------+------------------+----------------|
// | Label      | child    | parent child     | dest child     |
// |            | neighbor | parent neighbor  | dest child     |
// |------------+----------+------------------+----------------|
// | Expander   | child    | parent child     | dest neighbor  |
// |            | neighbor | parent neighbor  | dest neighbor  |
//
// 基本は上記で作成，
// Item種別(Group, Scene, Page, Mat, Ovimg, Pmat)間の関係で
// ダメな場合は，child/neighborを入れ替えて試行

// expander_to_dest_member /////////////////////////////////
fn expander_to_dest_member2(e: &TreeExpander, root_store: gio::ListStore)
                            -> TreeManipulationHandle{
    let dest_row= e.list_row().unwrap();
    let dest_sno= dest_row
        .item().and_downcast::<ScenarioNodeObject>().expect("sno is expd");
    let dest_depth= dest_row.depth();
    let dest_store= expander_to_store(e, dest_depth);

    let dest_parent_row   = row_to_parent_row(&dest_row);
    let dest_parent_store = row_to_parent_store(&dest_parent_row, &root_store);
    let dest_parent_sno   = get_parent_sno(&dest_sno, &dest_parent_row, &dest_store);

    TreeManipulationHandle{
        bt           : dest_sno.get_bt().into(),
        row          : Some(dest_row.clone().into()),
        sno          : Some(dest_sno.clone().into()),
        store        : Some(dest_store.clone().into()),
        depth        : Cell::new(dest_row.depth()),
        size         : Cell::new(dest_store.n_items()),
        parent_row   : Some(dest_parent_row.clone().into()),
        parent_sno   : Some(dest_parent_sno.into()),
        parent_store : Some(dest_parent_store.into()),
    }
}

// src_value_to_src_member /////////////////////////////////
fn src_value_to_src_member2(v: &Value) ->
    (TreeManipulationHandle, gio::ListStore, Rc<OperationHistory>){

        let drag_obj = v.get::<ScenarioItemDragObject>().expect("scn itm drag obj is expd");

        let src_row = drag_obj
            .get_list_item()
            .item().and_downcast::<TreeListRow>().expect("tlrow is expected");
        let src_sno = src_row
            .item().and_downcast::<ScenarioNodeObject>().expect("sno is expected");
        let src_depth = src_row.depth();
        let src_store = expander_to_store(&drag_obj
                                          .get_list_item()
                                          .child().expect("child")
                                          .downcast::<TreeExpander>().expect("TreeExpander"),
                                          src_depth);

        let src_parent_row   = row_to_parent_row(&src_row);
        let src_parent_store = row_to_parent_store(&src_parent_row, &drag_obj.get_root_store());
        let src_parent_sno   = get_parent_sno(&src_sno, &src_parent_row, &src_store);

        let hdl = TreeManipulationHandle{
            bt           : src_sno.get_bt().into(),
            row          : Some(src_row.clone().into()),
            sno          : Some(src_sno.clone().into()),
            store        : Some(src_store.clone().into()),
            depth        : Cell::new(src_depth),
            size         : Cell::new(src_store.n_items()),
            parent_row   : Some(src_parent_row.clone().into()),
            parent_sno   : Some(src_parent_sno.into()),
            parent_store : Some(src_parent_store.into()),
        };
        (hdl, drag_obj.get_root_store(), drag_obj.get_history())
}

// label_drop_remove_style //////////////////////////////////////
fn label_drop_remove_style(w: Widget, u: bool, l: bool) {
    if u { w.add_css_class   ("indicate_upper"); }
    else { w.remove_css_class("indicate_upper"); }

    if l { w.add_css_class   ("indicate_lower"); }
    else { w.remove_css_class("indicate_lower"); }
}

// label_drop_function /////////////////////////////////////
fn label_drop_function(d: &DropTarget, v: &Value, _x: f64, y: f64) -> bool{

    // obtain src
    let (src_hdl, root_store, history) =
        src_value_to_src_member2(v);
    let src_row    = src_hdl.row.as_ref().unwrap();
    let src_sno    = src_hdl.sno.as_ref().unwrap();
    let src_store  = src_hdl.store.as_ref().unwrap();

    // obtain dest
    let dest_hdl =
        expander_to_dest_member2( &d.widget()
                                   .parent().and_downcast::<TreeExpander>()
                                   .expect("expander is expected"),
                                   root_store);
    let dest_sno        = dest_hdl.sno.as_ref().unwrap();
    let dest_row        = dest_hdl.row.as_ref().unwrap();
    let dest_parent_sno = dest_hdl.parent_sno.as_ref().unwrap();
    let dest_store      = dest_hdl.store.as_ref().unwrap();

    // check: move to descendant -> ignore
    if detect_descendant(&src_row, &dest_row) {
        println!("moving to descendant is ignored");
        label_drop_remove_style( d.widget(), false, false );
        return false;
    }

    let new_node= ScenarioNodeObject::new_from( src_sno.get_node() );

    let mut h= OperationHistoryItem::default();

    if y < (d.widget().height()/2).into() { // upper-half
        if dest_sno.get_bt() == BranchType::Child { // parent に mv_to_child
            if (*dest_sno.get_node().parent.borrow_mut()).upgrade().is_some() {
                h.ope = Operation::MvToParentChild.into();
                if !ScenarioNode::mv_to_child(dest_parent_sno.get_node(), new_node.get_node()){
                    label_drop_remove_style( d.widget(), false, false );
                    return false;
                }
            } else {
                h.ope = Operation::MvToParent.into();
                if !ScenarioNode::mv_to_parent(dest_sno.get_node(), new_node.get_node()){
                    label_drop_remove_style( d.widget(), false, false );
                    return false;
                }
            }
        } else { // parent に mv_to_neighbor
            h.ope = Operation::MvToParentNeighbor.into();
            if !ScenarioNode::mv_to_neighbor(dest_parent_sno.get_node(), new_node.get_node()){
                label_drop_remove_style( d.widget(), false, false );
                return false;
            }
        }
        new_node.set_seq( dest_sno.get_seq() );
        adj_seq( &dest_store, dest_sno.get_seq(), 1 );
        dest_store.insert( (dest_sno.get_seq() as u32) - 1, &new_node ); // -1: because +1 at previouse adj_seq()

        // remove src
        adj_seq(&src_store, src_sno.get_seq() + 1, -1);
        src_store.remove( src_sno.get_seq() as u32 );
    } else { // lower-half -> dest child
        h.ope= Operation::MvToDestChild.into();
        if !ScenarioNode::mv_to_child(dest_sno.get_node(), new_node.get_node()){
            label_drop_remove_style( d.widget(), false, false );
            return false;
        }
        new_node.set_seq( 0 );
        if let Some(m) = dest_row.children(){
            let s= m.downcast::<gio::ListStore>().expect("ListStore");
            adj_seq( &s, 0, 1 );
            s.insert( 0, &new_node );
        }
        else {
            let dest_node= ScenarioNodeObject::new_from( dest_sno.get_node() );
            dest_node.set_seq( dest_sno.get_seq() );
            dest_store.remove( dest_sno.get_seq() as u32 );
            dest_store.insert( dest_sno.get_seq() as u32, &dest_node );
        }

        // remove src
        adj_seq(&src_store, src_sno.get_seq() + 1, -1);
        src_store.remove( src_sno.get_seq() as u32 );
    }

    label_drop_remove_style( d.widget(), false, false );

    h.src     = src_hdl;
    h.dest    = dest_hdl;
    h.new_sno = Some(new_node.clone().into());
    history.push(h.clone());

    true
}

// expander_drop_function //////////////////////////////////
fn expander_drop_function(d: &DropTarget, v: &Value, _x: f64, y: f64) -> bool{
    // obtain src
    let (src_hdl, root_store, history) =
        src_value_to_src_member2(v);
    let src_row    = src_hdl.row.as_ref().unwrap();
    let src_sno    = src_hdl.sno.as_ref().unwrap();
    let src_store  = src_hdl.store.as_ref().unwrap();

    // obtain dest
    let dest_hdl =
        expander_to_dest_member2( &d.widget()
                                   .downcast::<TreeExpander>()
                                   .expect("expander is expected"),
                                   root_store);
    let dest_sno        = dest_hdl.sno.as_ref().unwrap();
    let dest_row        = dest_hdl.row.as_ref().unwrap();
    let dest_parent_sno = dest_hdl.parent_sno.as_ref().unwrap();
    let dest_store      = dest_hdl.store.as_ref().unwrap();

    // check: move to descendant -> ignore
    if detect_descendant(&src_row, &dest_row) {
        println!("moving to descendant is ignored");
        label_drop_remove_style( d.widget(), false, false );
        return false;
    }

    let new_node= ScenarioNodeObject::new_from( src_sno.get_node() );

    let mut h= OperationHistoryItem::default();

    if y < (d.widget().height()/2).into() { // upper half
        if dest_sno.get_bt() == BranchType::Child { // parent に mv_to_child
            if (*dest_sno.get_node().parent.borrow_mut()).upgrade().is_some() {
                h.ope = Operation::MvToParentChild.into();
                if !ScenarioNode::mv_to_child(dest_parent_sno.get_node(), new_node.get_node()){
                    label_drop_remove_style( d.widget(), false, false );
                    return false;
                }
            } else {
                h.ope = Operation::MvToParent.into();
                if !ScenarioNode::mv_to_parent(dest_sno.get_node(), new_node.get_node()){
                    label_drop_remove_style( d.widget(), false, false );
                    return false;
                }
            }
        } else { // parent に mv_to_neighbor
            h.ope = Operation::MvToParentNeighbor.into();
            if !ScenarioNode::mv_to_neighbor(dest_parent_sno.get_node(), new_node.get_node()){
                label_drop_remove_style( d.widget(), false, false );
                return false;
            }
        }
        new_node.set_seq( dest_sno.get_seq() );
        adj_seq( &dest_store, dest_sno.get_seq(), 1 );
        dest_store.insert( (dest_sno.get_seq() as u32) - 1, &new_node ); // -1: because +1 at previouse adj_seq()
    } else { // lower-half -> dest に mv_to_neighbor
        h.ope= Operation::MvToDestNeighbor.into();
        if !ScenarioNode::mv_to_neighbor(dest_sno.get_node(), new_node.get_node()){
            label_drop_remove_style( d.widget(), false, false );
            return false;
        }
        new_node.set_seq( dest_sno.get_seq() + 1 );
        adj_seq( &dest_store, dest_sno.get_seq() + 1, 1 );
        dest_store.insert( (dest_sno.get_seq() as u32) + 1, &new_node );
    }
    // remove src
    adj_seq(&src_store, src_sno.get_seq() + 1, -1);
    src_store.remove( src_sno.get_seq() as u32 );

    label_drop_remove_style( d.widget(), false, false );

    h.src     = src_hdl;
    h.dest    = dest_hdl;
    h.new_sno = Some(new_node.clone().into());
    history.push(h.clone());

    true
}

// build_tree_list_view ////////////////////////////////////
pub fn build_tree_list_view(tree_list_model: TreeListModel,
                            selection_model: SingleSelection,
                            history_for_factory: Rc<OperationHistory>) -> ListView {
    let factory = SignalListItemFactory::new();
    let list_view = ListView::new(Some(selection_model.clone()), Some(factory.clone()));
    list_view.set_vexpand_set(true);

    // setup handler ///////////////////////////////////////
    factory.connect_setup(move |_, list_item| {
        let expander= TreeExpander::new();
        let label   = Label::new(None);
        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&expander)); // list_item の child は expander
        expander.set_child(Some(&label));

    });
    // teardown ////////////////////////////////////////////
    factory.connect_teardown(clone!(
        @weak list_view, @weak selection_model =>
            move |_factory, _list_item|{
                list_view.scroll_to(selection_model.selected(),
                                    ListScrollFlags::NONE,
                                    None);}));
    // bind handler ////////////////////////////////////////
    factory.connect_bind(move |_, list_item| {
        // bindの引数は
        // 1. GtkSignalListItemFactory* self,
        // 2. GObject* object,
        // 3. gpointer user_data

        let expander = list_item.downcast_ref::<ListItem>().expect("Needs to be ListItem")
            .child().and_downcast::<TreeExpander>().expect("The child has to be a `TreeExpander`.");
        let label = expander.child().and_downcast::<Label>().expect("label is expected");

        let tree_list_row = list_item.downcast_ref::<ListItem>().expect("Needs to be ListItem")
            .item().and_downcast::<TreeListRow>().expect("TreeListRow is expected");

        expander.set_list_row( Some(&tree_list_row) );

        // configure label content ///////////////////////
        let scn_object = tree_list_row
            .item()
            .and_downcast::<ScenarioNodeObject>()
            .expect("ScenarioNodeObject is expected");

        // for debug
        // label.set_label( &(format!("{}",scn_object.get_node()) +
        //                    ",seq:" +
        //                    &scn_object.get_seq().to_string()) );
        label.set_label( &scn_object.get_node().summary_str() );

        label.set_xalign(0.0);
        label.set_vexpand(true); label.set_hexpand(true);

        // configure drag source of label //////////////////

        let drag_source= DragSource::new();
        let scenario_item_drag_source = ScenarioItemDragObject::new();
        scenario_item_drag_source.set_root_store( tree_list_model.model()
                                                          .downcast::<gio::ListStore>()
                                                          .expect("ListStore is expd")) ;
        scenario_item_drag_source.set_history( history_for_factory.clone() );
        scenario_item_drag_source.set_list_item(
            list_item.downcast_ref::<ListItem>().expect("ListItem is expd").clone() );

        drag_source.set_content(
            Some( &ContentProvider::for_value( &Value::from( &scenario_item_drag_source ))));

        drag_source.connect_drag_begin(|_a, _c| { // <DragSource>, <Drag>
            //let widget_paintable= WidgetPaintable::new(glib::bitflags::_core::option::Option::Some(a));
            //a.set_icon(Some(&widget_paintable), 32, 58);
        });
        label.add_controller(drag_source);

        // configure drop target of label //////////////////
        let drop_target= DropTarget::new( ScenarioItemDragObject::static_type(), DragAction::COPY);
        drop_target.connect_drop( label_drop_function );
        drop_target.connect_motion( |d, _x, y|{
            if y < (d.widget().height()/2).into() {
                label_drop_remove_style(d.widget(), true, false);  }
            else {
                label_drop_remove_style(d.widget(), false, true);  }
            DragAction::COPY
        } );
        drop_target.connect_leave(
            |d|{ label_drop_remove_style(d.widget(), false, false); } );

        label.add_controller(drop_target);

        // Expander(リスト行)に対するドロップ(notラベル部分)
        let drop_target2= DropTarget::new( ScenarioItemDragObject::static_type(), DragAction::COPY);
        drop_target2.connect_motion( |d, x, y|{
            let c=
                d.widget()
                .downcast::<TreeExpander>()
                .expect("expander is expected")
                .child().unwrap().allocation();
            let x32 = x as i32;
            let y32 = y as i32;

            if (c.x() <= x32) && (x32 <= c.x() + c.width()) &&
                (c.y() <= y32) && (y32 <= c.y() + c.height()) {
                    label_drop_remove_style(d.widget(), false, false); }
            else if y < (d.widget().height()/2).into() {
                label_drop_remove_style(d.widget(), true, false);  }
            else {
                label_drop_remove_style(d.widget(), false, true);  }
            DragAction::COPY
        } );
        drop_target2.connect_leave(
            |d|{ label_drop_remove_style(d.widget(), false, false); } );
        drop_target2.connect_drop(expander_drop_function);
        expander.add_controller(drop_target2);
    });


    list_view
}
