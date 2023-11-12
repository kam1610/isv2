use std::rc::Rc;
use std::cell::{RefCell,Cell};
use gtk::gio;
use gtk::ListView;
use gtk::SingleSelection;
use gtk::TreeListModel;
use gtk::TreeListRow;
use gtk::prelude::Cast;
use gtk::prelude::ListModelExt;
use crate::scenario_node::BranchType;
use crate::scenario_node::ScenarioNode;
use crate::scenario_node_object::ScenarioNodeObject;
use crate::scenario_node_object::add_child;
use crate::scenario_node_object::add_neighbor;
use crate::scenario_node_object::adj_seq;
use crate::scenario_node_object::remove_node;

// operation_history
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation{
    Remove,
    AddNeighbor,
    AddChild,
    AddRoot,
    MvToParentChild,
    MvToDestChild,
    MvToParentNeighbor,
    MvToDestNeighbor,
    MvToParent,
    Nop,
}

#[derive(Clone, Debug)]
pub struct TreeManipulationHandle{
    pub bt           : Cell<BranchType>,
    pub row          : Option<Rc<TreeListRow>>,
    pub sno          : Option<Rc<ScenarioNodeObject>>,
    pub store        : Option<Rc<gio::ListStore>>,
    pub depth        : Cell<u32>,
    pub size         : Cell<u32>,
    pub parent_row   : Option<Rc<TreeListRow>>,
    pub parent_sno   : Option<Rc<ScenarioNodeObject>>,
    pub parent_store : Option<Rc<gio::ListStore>>,
}
impl Default for TreeManipulationHandle{
    fn default() -> Self{
        TreeManipulationHandle{
            bt           : Cell::new(BranchType::Child),
            row          : None,
            sno          : None,
            store        : None,
            depth        : Cell::new(0),
            size         : Cell::new(0),
            parent_row   : None,
            parent_sno   : None,
            parent_store : None
        }
    }
}

#[derive(Clone)]
pub struct OperationHistoryItem{
    pub ope     : Cell<Operation>,
    pub src     : TreeManipulationHandle,
    pub dest    : TreeManipulationHandle,
    pub new_sno : Option<Rc<ScenarioNodeObject>>,
}
impl OperationHistoryItem{
    pub fn default() -> Self{
        OperationHistoryItem{
            ope            : Cell::new(Operation::Nop),
            src            : TreeManipulationHandle::default(),
            dest           : TreeManipulationHandle::default(),
            new_sno        : None,
        }
    }
    pub fn new_with_root_store(ope  : Operation,
                               store: &gio::ListStore,
                               sno  : &ScenarioNodeObject ) -> OperationHistoryItem{

        let mut src_hdl = TreeManipulationHandle::default();
        src_hdl.bt    = Cell::new(BranchType::Child);
        src_hdl.sno   = Some(sno.clone().into());
        src_hdl.store = Some(store.clone().into());

        OperationHistoryItem{
            ope            : Cell::new(ope),
            src            : src_hdl,
            dest           : TreeManipulationHandle::default(),
            new_sno        : None,
        }
    }
    pub fn new_from_handle(
        ope: Operation,
        hdl: TreeManipulationHandle) -> OperationHistoryItem{
        OperationHistoryItem{
            ope            : Cell::new(ope),
            src            : hdl,
            dest           : TreeManipulationHandle::default(),
            new_sno        : None,
        }
    }
    pub fn set_ope(&self, ope:Operation){
        self.ope.set(ope);
    }
}

pub struct OperationHistory{
    history: RefCell<Vec<OperationHistoryItem>>,
    index  : Cell<i32>,
    lv     : RefCell<Option<Rc<ListView>>>,
    size   : Cell<i32>,
}

impl Default for OperationHistory{
    fn default() -> Self{
        OperationHistory{
            history: RefCell::new(Vec::new()),
            index  : Cell::new(0),
            lv     : RefCell::new(None),
            size   : Cell::new(0),
        }
    }
}
// undo_remove /////////////////////////////////////////////
fn undo_remove(h: &OperationHistoryItem){
    let src_store        = h.src.store.as_ref().unwrap();
    let src_sno          = h.src.sno.as_ref().unwrap();
    let src_parent_sno   = h.src.parent_sno.as_ref().unwrap();
    let src_parent_row   = h.src.parent_row.as_ref().unwrap();
    let src_parent_store = h.src.parent_store.as_ref().unwrap();

    if h.src.bt.get() == BranchType::Neighbor { // like add_neighbor_button
        add_neighbor(&src_parent_sno, &src_sno, &src_store);
    } else if (h.src.bt.get() == BranchType::Child) && (h.src.depth.get() > 0) {
        add_child( &src_parent_sno, &src_sno, &src_parent_row, &src_parent_store);
    } else { // root-root
        if h.src.size.get() <= 1 { // last one
            // do nothing
        } else {
            adj_seq( &src_store, 0, 1 );
            let dest_sn= src_store.item(0).unwrap().downcast_ref::<ScenarioNodeObject>().expect("sno").get_node();
            ScenarioNode::mv_to_parent(dest_sn, src_sno.get_node());
        }
        src_store.insert( 0, src_sno.as_ref() );
    }
}
// undo_add_neighbor ///////////////////////////////////////
fn undo_add_neighbor(h: &OperationHistoryItem){
    let new_sno   = h.new_sno.as_ref().unwrap();
    let src_store = h.src.store.as_ref().unwrap();

    remove_node(src_store, new_sno);
}
// undo_add_child ///////////////////////////////////////
fn undo_add_child(h: &OperationHistoryItem){
    let new_sno = h.new_sno.as_ref().unwrap();

    if h.src.row.as_ref().unwrap().children().is_none() { // 追加で子なし→ありになった場合
        let src_store = h.src.store.as_ref().unwrap();
        let src_sno   = h.src.sno.as_ref().unwrap();
        new_sno.get_node().remove();
        src_store.upcast_ref::<gio::ListModel>().items_changed(src_sno.get_seq() as u32, 1, 1);
    } else { // 既に子がいるところに追加した場合
        let src_store = h.src.row.as_ref().unwrap().children().unwrap().downcast::<gio::ListStore>().expect("ListStore");
        remove_node(&src_store, new_sno);
    }
}
// undo_add_root ///////////////////////////////////////////
fn undo_add_root(h: &OperationHistoryItem){
    let src_store   = h.src.store.as_ref().unwrap();
    let src_sno     = h.src.sno.as_ref().unwrap();

    remove_node(&src_store, src_sno);
}
// undo_moved_source ///////////////////////////////////////
fn undo_moved_source(h: &OperationHistoryItem){
    let src_bt           = h.src.bt.get();
    let src_parent_sno   = h.src.parent_sno.as_ref().unwrap();
    let new_sno          = h.new_sno.as_ref().unwrap();
    let src_store        = h.src.store.as_ref().unwrap();
    let src_depth        = h.src.depth.get();
    let src_parent_store = h.src.parent_store.as_ref().unwrap();
    let src_parent_row   = h.src.parent_row.as_ref().unwrap();

    if src_bt == BranchType::Neighbor {
        add_neighbor(&src_parent_sno, &new_sno, &src_store);
    } else {
        if  src_depth > 0 {
            add_child(&src_parent_sno,
                      &new_sno,
                      &src_parent_row,
                      &src_parent_store);
        } else { // from root
            adj_seq( &src_store, 0, 1 );
            let dest_sn= src_store.item(0).unwrap()
                .downcast_ref::<ScenarioNodeObject>().expect("sno").get_node();
            ScenarioNode::mv_to_parent(dest_sn, new_sno.get_node());
            new_sno.set_seq(0);
            src_store.insert( 0, new_sno.as_ref() );
        }
    }

}
// undo_mv_to_parent_neighbor //////////////////////////////
fn undo_mv_to_parent_neighbor(h: &OperationHistoryItem){
    let new_sno    = h.new_sno.as_ref().unwrap();
    let dest_store = h.dest.store.as_ref().unwrap();

    remove_node(&dest_store, &new_sno);

    undo_moved_source(h);
}
// undo_mv_to_dest_neighbor ////////////////////////////////
fn undo_mv_to_dest_neighbor(h: &OperationHistoryItem){
    let new_sno    = h.new_sno.as_ref().unwrap();
    let dest_store = h.dest.store.as_ref().unwrap();

    remove_node(&dest_store, &new_sno);

    undo_moved_source(h);
}
// undo_mv_to_parent_child /////////////////////////////////
fn undo_mv_to_parent_child(h: &OperationHistoryItem){
    let new_sno    = h.new_sno.as_ref().unwrap();
    let dest_store = h.dest.store.as_ref().unwrap();

    remove_node(&dest_store, &new_sno);

    undo_moved_source(h);
}
// undo_mv_to_dest_child ///////////////////////////////////
fn undo_mv_to_dest_child(h: &OperationHistoryItem){
    let new_sno  = h.new_sno.as_ref().unwrap();
    let dest_row = h.dest.row.as_ref().unwrap();

    if dest_row.children().is_some() {
        let dest_children = &dest_row.children().unwrap().downcast::<gio::ListStore>().expect("ListStore");
        remove_node(&dest_children, &new_sno);
    }

    undo_moved_source(h);
}
// undo_mv_to_parent ///////////////////////////////////////
fn undo_mv_to_parent(h: &OperationHistoryItem){
    let new_sno    = h.new_sno.as_ref().unwrap();
    let dest_store = h.dest.store.as_ref().unwrap();

    remove_node(&dest_store, &new_sno);

    undo_moved_source(h);
}
// redo_remove /////////////////////////////////////////////
fn redo_remove(h: &OperationHistoryItem){
    let src_store   = h.src.store.as_ref().unwrap();
    let src_sno     = h.src.sno.as_ref().unwrap();

    remove_node(&src_store, src_sno);
}
// redo_add_neighbor ///////////////////////////////////////
fn redo_add_neighbor(h: &OperationHistoryItem){
    let src_store = h.src.store.as_ref().unwrap();
    let new_sno   = h.new_sno.as_ref().unwrap();
    let src_sno   = h.src.sno.as_ref().unwrap();
    add_neighbor(&src_sno, &new_sno, &src_store);
}
// redo_add_child //////////////////////////////////////////
fn redo_add_child(h: &OperationHistoryItem){
    let src_store = h.src.store.as_ref().unwrap();
    let new_sno   = h.new_sno.as_ref().unwrap();
    let src_sno   = h.src.sno.as_ref().unwrap();
    let src_row   = h.src.row.as_ref().unwrap();
    add_child( &src_sno, &new_sno, &src_row, &src_store);
}
// redo_add_root ///////////////////////////////////////////
fn redo_add_root(h: &OperationHistoryItem){
    let src_store = h.src.store.as_ref().unwrap();
    let src_sno   = h.src.sno.as_ref().unwrap();
    src_store.insert( 0, src_sno.as_ref() );
}
// redo_mv_to_neighbor /////////////////////////////////////
fn redo_mv_to_parent_neighbor(h: &OperationHistoryItem){
    let dest_store      = h.dest.store.as_ref().unwrap();
    let dest_parent_sno = h.dest.parent_sno.as_ref().unwrap();
    let src_store       = h.src.store.as_ref().unwrap();
    let src_sno         = h.src.sno.as_ref().unwrap();
    let new_sno         = h.new_sno.as_ref().unwrap();

    remove_node(&src_store, &src_sno);
    add_neighbor(&dest_parent_sno, &new_sno, &dest_store);
}
fn redo_mv_to_dest_neighbor(h: &OperationHistoryItem){
    let dest_store      = h.dest.store.as_ref().unwrap();
    let dest_sno        = h.dest.sno.as_ref().unwrap();
    let src_store       = h.src.store.as_ref().unwrap();
    let src_sno         = h.src.sno.as_ref().unwrap();
    let new_sno         = h.new_sno.as_ref().unwrap();

    remove_node(&src_store, &src_sno);
    add_neighbor(&dest_sno, &new_sno, &dest_store);
}
fn redo_mv_to_parent_child(h: &OperationHistoryItem){
    let dest_parent_sno   = h.dest.parent_sno.as_ref().unwrap();
    let dest_parent_store = h.dest.parent_store.as_ref().unwrap();
    let dest_parent_row   = h.dest.parent_row.as_ref().unwrap();
    let src_store         = h.src.store.as_ref().unwrap();
    let src_sno           = h.src.sno.as_ref().unwrap();
    let new_sno           = h.new_sno.as_ref().unwrap();

    remove_node(&src_store, &src_sno);
    add_child(&dest_parent_sno, &new_sno, &dest_parent_row, &dest_parent_store);
}
fn redo_mv_to_dest_child(h: &OperationHistoryItem){
    let dest_sno   = h.dest.sno.as_ref().unwrap();
    let dest_store = h.dest.store.as_ref().unwrap();
    let dest_row   = h.dest.row.as_ref().unwrap();
    let src_store  = h.src.store.as_ref().unwrap();
    let src_sno    = h.src.sno.as_ref().unwrap();
    let new_sno    = h.new_sno.as_ref().unwrap();

    remove_node(&src_store, &src_sno);
    add_child(&dest_sno, &new_sno, &dest_row, &dest_store);
}
fn redo_mv_to_parent(h: &OperationHistoryItem){
    let src_store  = h.src.store.as_ref().unwrap();
    let dest_store = h.dest.store.as_ref().unwrap();
    let dest_sno   = h.dest.sno.as_ref().unwrap();
    let new_sno    = h.new_sno.as_ref().unwrap();

    remove_node(&src_store, &new_sno);
    adj_seq( &dest_store, 0, 1 );
    ScenarioNode::mv_to_parent(dest_sno.get_node(), new_sno.get_node());
    new_sno.set_seq(0);
    dest_store.insert( 0, new_sno.as_ref() );
}
// OperationHistory ////////////////////////////////////////
impl OperationHistory{
    // new /////////////////////////////////////////////////
    pub fn new(lv: ListView) -> OperationHistory{
        let oh=  OperationHistory {
            history: RefCell::new(Vec::new()),
            index  : Cell::new(0),
            size   : Cell::new(0),
            lv     : RefCell::new(Some(lv.clone().into())),
        };
        oh
    }
    pub fn set_list_view(&self, lv: ListView){
        *self.lv.borrow_mut() = Some(Rc::new(lv));
    }
    // push ////////////////////////////////////////////////
    pub fn push(&self, oh: OperationHistoryItem) {
        if self.history.borrow().len() > (self.index.get() as usize){
            self.history.borrow_mut().resize( self.index.get() as usize,
                                              OperationHistoryItem::default() );
        }
        self.history.borrow_mut().push(oh);
        self.index.set( self.index.get() + 1 ); // index indicates lates empty slot
        self.size.set( self.index.get() );
    }
    // redraw_all //////////////////////////////////////////
    pub fn redraw_all(&self){
        let list_model= self.lv.borrow().as_ref().unwrap().as_ref().model().unwrap() // SelectionModel
            .downcast::<SingleSelection>().expect("SingleSelection")
            .model().unwrap() // TreeListModel
            .downcast::<TreeListModel>().expect("TreeListModel")
            .model();          // ListModel
        for i in 0..list_model.n_items() {
            list_model.items_changed(i, 1, 1);
        }
    }

    // undo ////////////////////////////////////////////////
    pub fn undo(&self) -> bool{
        if self.index.get() <= 0 {
            return false; }
        self.index.set( self.index.get() - 1); // decrement before get
        let h= &self.history.borrow()[self.index.get() as usize];
        match h.ope.get() {
            Operation::Remove             => undo_remove(&h),
            Operation::AddNeighbor        => undo_add_neighbor(&h),
            Operation::AddChild           => undo_add_child(&h),
            Operation::AddRoot            => undo_add_root(&h),
            Operation::MvToParentNeighbor => undo_mv_to_parent_neighbor(&h),
            Operation::MvToDestNeighbor   => undo_mv_to_dest_neighbor(&h),
            Operation::MvToParentChild    => undo_mv_to_parent_child(&h),
            Operation::MvToDestChild      => undo_mv_to_dest_child(&h),
            Operation::MvToParent         => undo_mv_to_parent(&h),
            _ => ()
        }

        self.redraw_all();

        true
    }
    // redo ////////////////////////////////////////////////
    pub fn redo(&self) -> bool{
        if self.index.get() >= self.size.get() {
            return false; }
        let h= &self.history.borrow()[self.index.get() as usize];
        match h.ope.get() {
            Operation::Remove             => redo_remove(&h),
            Operation::AddNeighbor        => redo_add_neighbor(&h),
            Operation::AddChild           => redo_add_child(&h),
            Operation::AddRoot            => redo_add_root(&h),
            Operation::MvToParentNeighbor => redo_mv_to_parent_neighbor(&h),
            Operation::MvToDestNeighbor   => redo_mv_to_dest_neighbor(&h),
            Operation::MvToParentChild    => redo_mv_to_parent_child(&h),
            Operation::MvToDestChild      => redo_mv_to_dest_child(&h),
            Operation::MvToParent         => redo_mv_to_parent(&h),
            _ => ()
        }
        self.index.set( self.index.get() + 1 ); // increment after operation

        self.redraw_all();

        true
    }
}
