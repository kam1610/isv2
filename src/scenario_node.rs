//! group
//!   scene(同じ画層のまとまり:
//!         bgimg, bgcol, crop, lbl(ラベル名), lblref(ラベル名参照))
//!     page(クリック単位)
//!       mat(テキストの背景: col, pos, dim, r, a, v(縦書き), lbl, lblref, txt)
//!       ovimg(オーバーレイイメージ: path(画像), pos, a)
//!     pmat(クリック単位，matがひとつのみの場合，属性はmatと同じ)
//!
//! -> enum Item の候補は group / scene / page / mat / ovimg / pmat
//!    親子関係は
//!    1. group(管理の単位，実体なし)
//!      2. scene(同一背景の単位)
//!        3.1 page(クリックの単位，実体なし)
//!          3.1.1 mat
//!          3.1.2 ovimg
//!        3.2. pmat(pageの特殊形, matと等価)

use std::cell::{RefCell,Cell};
use std::collections::VecDeque;
use std::convert::From;
use std::fmt;
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use serde::{Deserialize, Serialize};
use dunce;

// ScenarioNodeSerde ///////////////////////////////////////
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HasBranches{ Both, Neighbor, Child, None, }
#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioNodeSerde{
    pub value       : RefCell<Item>,
    pub bt          : Cell<BranchType>,
    pub id          : Cell<i32>,
    pub has_n_and_c : Cell<HasBranches>,
}
impl From<&ScenarioNode> for ScenarioNodeSerde{
    fn from(sn: &ScenarioNode) -> Self{
        let has_n_and_c =
            match (sn.child.borrow().as_ref().is_some(),
                   sn.neighbor.borrow().as_ref().is_some()) {
                (true,  true ) => HasBranches::Both,
                (true,  false) => HasBranches::Child,
                (false, true ) => HasBranches::Neighbor,
                (false, false) => HasBranches::None,
        };
        Self{
            value : RefCell::new((*sn.value.borrow()).clone()),
            bt    : sn.bt.clone(),
            id    : sn.id.clone(),
            has_n_and_c: has_n_and_c.into(),
        }
    }
}
impl ScenarioNodeSerde{
    // from_sn /////////////////////////////////////////////
    pub fn from_sn(root: Rc<ScenarioNode>) -> Vec<ScenarioNodeSerde>{
        let mut dest: Vec<ScenarioNodeSerde> = vec![];
        let mut iter = vec![root];
        loop{
            let p = ScenarioNode::traverse(&mut iter);
            if p.is_some(){
                let sn_ser = ScenarioNodeSerde::from(&*p.clone().unwrap());
                dest.push(sn_ser);
            } else {
                break;
            }
        }
        dest
    }
}
impl fmt::Display for ScenarioNodeSerde {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut disp_str    = String::new();
        let mut mat_text    = String::new();
        let mut scene_bgimg = String::new();
        match &*self.value.borrow() {
            Item::Group    => {disp_str += "Group:";},
            Item::Scene(s) => {
                disp_str += "Scene:";
                if let Some(bgimg) = &s.bgimg{
                    scene_bgimg = bgimg.to_str().unwrap().to_string();
                }
            },
            Item::Page(_p) => {disp_str += "Page :";},
            Item::Mat(m)   => {disp_str += "Mat  :"; mat_text = m.text.clone(); },
            Item::Ovimg(_o)=> {disp_str += "Ovimg:";},
            Item::Pmat(pm) => {disp_str += "Pmat :"; mat_text = pm.text.clone(); },
        }
        if self.bt.get() == BranchType::Child{
            disp_str += "C:"; }
        else {
            disp_str += "N:"; }
        match self.has_n_and_c.get() {
            HasBranches::Both     => { disp_str += "n+c: "; },
            HasBranches::Child    => { disp_str += "c  : "; },
            HasBranches::Neighbor => { disp_str += "n  : "; },
            HasBranches::None     => { disp_str += "nil: "; },
        }
        disp_str += &mat_text.replace("\n","");
        disp_str += &scene_bgimg;

        write!(f, "{}", disp_str)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioNodeSerdeVec { pub nodes: Vec<ScenarioNodeSerde> }
impl From<Vec<ScenarioNodeSerde>> for ScenarioNodeSerdeVec {
    fn from(nodes: Vec<ScenarioNodeSerde>) -> Self {
        ScenarioNodeSerdeVec { nodes }
    }
}
// ScenarioNode ////////////////////////////////////////////
#[derive(Debug)]
pub struct ScenarioNode {
    pub value   : RefCell<Item>,
    pub bt      : Cell<BranchType>,
    pub parent  : RefCell<Weak<ScenarioNode>>, // Cellはコピー/置き換えになっちゃうのでRefCell
    pub child   : RefCell<Option<Rc<ScenarioNode>>>,
    pub neighbor: RefCell<Option<Rc<ScenarioNode>>>,
    pub id      : Cell<i32>,
}
impl Default for ScenarioNode{
    fn default() -> Self{
        ScenarioNode{
            value   : RefCell::new(Item::Page( Page{ name: String::from("new_page") } )),
            bt      : Cell::new(BranchType::Child),
            parent  : RefCell::new(Weak::new()),
            child   : RefCell::new(None),
            neighbor: RefCell::new(None),
            id      : Cell::new(0),
        }
    }
}
// ScenarioNode ////////////////////////////////////////////////////
fn dump_mat(m: &Mat) -> String{
    let mut s= String::new();
    s+= &("M(".to_owned() +
          "c:" +
          &m.col.r.to_string() + "," +
          &m.col.g.to_string() + "," +
          &m.col.b.to_string() + "," +
          " p:" +
          &m.pos.x.to_string() + "," +
          &m.pos.y.to_string() + "," +
          " d:" +
          &m.dim.w.to_string() + "," +
          &m.dim.h.to_string() + "," +
          " r:" + &m.r.to_string() + "," +
          " a:" + &m.a.to_string() + "," +
          " " + &m.name + "),");
    if let Some(a)= &m.src   { s+= &("s".to_owned() + a); }
    if let Some(a)= &m.lbl   { s+= &("l".to_owned() + a); }
    s+= &("lt:".to_owned() + &m.lbl_type.to_string());
    s
}
impl fmt::Display for ScenarioNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s= String::from( self.id.get().to_string() );
        match &(*self.value.borrow()){
            Item::Group    => s+= "G,",
            Item::Scene(c) => {
                s+= "S,";
                if let Some(ref a)= c.bgimg{ s+= &("b[".to_owned() + a.to_str().unwrap() + "]"); }
                s+= &("c:".to_owned() +
                      &c.bgcol.r.to_string() + "," +
                      &c.bgcol.g.to_string() + "," +
                      &c.bgcol.b.to_string() + ",");
                s+= "ci:";
                s+= &(c.crop.pos.x.to_string() + "," + &c.crop.pos.y.to_string());
                s+= &(c.crop.dim.w.to_string() + "," + &c.crop.dim.h.to_string());
                if let Some(l)= &c.lbl   { s+= &("l".to_owned() + l); }
                s+= &("lt:".to_owned() + &c.lbl_type.to_string() + ",")
            },
            Item::Page(p)  => s+= &("P(".to_owned() + &p.name + ")")  ,
            Item::Mat(m)   => s+= &(dump_mat(m) + ","),
            Item::Ovimg(o) => s+= &("O([".to_owned() + &o.path + "]," +
                                    "p:" +
                                    &o.pos.x.to_string() + "," +
                                    &o.pos.y.to_string() + "),"),
            Item::Pmat(m)  => s+= &("P".to_owned() + &dump_mat(m) + ","),
        }
        match self.bt.get(){
            BranchType::Child => s+= "b:c,",
            _                 => s+= "b:n,",
        }
        s+= "p:";
        if let Some(p) = &self.parent.borrow().clone().upgrade(){
            match &(*p.value.borrow()){
                Item::Group    => s+= "G",
                Item::Scene(_c)=> s+= "S",
                Item::Page(_p) => s+= "P",
                Item::Mat(_m)  => s+= "M",
                Item::Ovimg(_o)=> s+= "O",
                Item::Pmat(_m) => s+= "pm",
            }
        }
        write!(f, "{}", s)
    }
}
// impl From<&ScenarioNodeSere> for ScenarioNode{
//     fn from(sn: &ScenarioNodeSere) -> Self{
//         //todo
//     }
// }
impl From<ScenarioNodeSerde> for ScenarioNode{
    fn from(sns: ScenarioNodeSerde) -> Self{
        Self{
            value    : sns.value,
            bt       : sns.bt,
            parent   : RefCell::new(Weak::new()),
            child    : RefCell::new(None),
            neighbor : RefCell::new(None),
            id       : sns.id
        }
    }
}
impl ScenarioNode {
    pub fn set_value(&self, v: Item){
        *self.value.borrow_mut()= v;
    }
    pub fn set_bt(&self, bt: BranchType){
        self.bt.set(bt);
    }
    pub fn set_parent(&self, p: Weak<ScenarioNode>){
        *self.parent.borrow_mut()= p;
    }
    pub fn set_child(&self, c: Rc<ScenarioNode>){
        *self.child.borrow_mut()= Some(c);
    }
    pub fn set_neighbor(&self, n: Rc<ScenarioNode>){
        *self.neighbor.borrow_mut()= Some(n);
    }
    pub fn unset_neighbor(&self){
        *self.neighbor.borrow_mut()= None;
    }
    pub fn new() -> ScenarioNode{
        ScenarioNode{
            value   : RefCell::new(Item::Page( Page{ name: String::from("new_page") } )),
            bt      : Cell::new(BranchType::Child),
            parent  : RefCell::new(Weak::new()),
            child   : RefCell::new(None),
            neighbor: RefCell::new(None),
            id      : Cell::new(0),
        }
    }
    pub fn remove(&self){
        let self_p= (*self.parent.borrow_mut()).upgrade();

        if self_p.is_some() { // parentあり -> root以外
            let self_p= self_p.unwrap().clone();
            let mut self_p_cn; // child or neighbor
            if self.bt == BranchType::Child.into() {
                self_p_cn= self_p.child.borrow_mut();
            } else {
                self_p_cn= self_p.neighbor.borrow_mut();
            }
            if let Some(self_n) = (*self.neighbor.borrow_mut()).as_ref(){
                *self_p_cn= Some(self_n.clone());
            } else {
                *self_p_cn= None;
            }
            if let Some(self_n) = (*self.neighbor.borrow_mut()).as_ref() {
                self_n.set_bt( self.bt.get() );
                self_n.set_parent(self.parent.clone().take());
            }
        } else { // rootの場合
            if let Some(self_n) = (*self.neighbor.borrow_mut()).as_ref() {
                self_n.set_bt(BranchType::Child);
                self_n.set_parent(Weak::new());
            }
        }
    }
    pub fn dump (&self, depth: usize){
        println!("{}{}", " ".repeat(depth), self);
        if let Some(c) = (*self.child.borrow_mut()).as_ref(){
            c.dump(depth + 2);
        }
        if let Some(n) = (*self.neighbor.borrow_mut()).as_ref(){
            n.dump(depth);
        }
    }
    // summary_str /////////////////////////////////////////
    pub fn summary_str(&self) -> String{
        let mut disp_str    = String::new();
        let mut mat_text    = String::new();
        let mut page_text   = String::new();
        let mut scene_bgimg = String::new();
        match &*self.value.borrow() {
            Item::Group    => {disp_str += "Group:";},
            Item::Scene(s) => {
                disp_str += "Scene:";
                if let Some(bgimg) = &s.bgimg{
                    scene_bgimg = bgimg.to_str().unwrap().to_string();
                }
            },
            Item::Page(p)  => {disp_str += "Page:"; page_text = p.name.clone(); },
            Item::Mat(m)   => {disp_str += "Mat:"; mat_text = m.text.clone(); },
            Item::Ovimg(_o)=> {disp_str += "Ovimg:";},
            Item::Pmat(pm) => {disp_str += "Pmat:"; mat_text = pm.text.clone(); },
        }

        disp_str += &mat_text.replace("\n","");
        disp_str += &scene_bgimg;
        disp_str += &page_text;

        disp_str
    }
    // get_belong_(item|scene|page) ////////////////////////
    fn get_belong_item(p : &Rc<ScenarioNode>,
                       predicate: impl Fn(&'_ Item) -> bool) -> Option<Rc<ScenarioNode>> {
        let mut p = p.clone();
        loop {
            if predicate(&p.value.borrow()) {
                return Some(p.clone());
            }
            let p1 = p.parent.borrow().clone();
            if p1.upgrade().is_some() {
                p = p1.upgrade().unwrap();
            } else {
                return None;
            }
        }
    }
    pub fn get_belong_group(p : &Rc<ScenarioNode>) -> Option<Rc<ScenarioNode>> {
        Self::get_belong_item(p, |i|{if let Item::Group = i {true} else {false} })
    }
    pub fn get_belong_scene(p : &Rc<ScenarioNode>) -> Option<Rc<ScenarioNode>> {
        Self::get_belong_item(p, |i|{if let Item::Scene(_) = i {true} else {false} })
    }
    pub fn get_belong_page(p : &Rc<ScenarioNode>) -> Option<Rc<ScenarioNode>> {
        Self::get_belong_item(p,
                              |i|{
                                  match i {
                                      Item::Page(_) => true,
                                      Item::Pmat(_) => true,
                                      _ => false
                                  }
                              })
    }
    // update_bgimg_relative_path //////////////////////////
    pub fn update_bgimg_relative_path(n: &Rc<ScenarioNode>,
                                      prev_base_dir: PathBuf,
                                      new_base_dir : &PathBuf){
        // find root
        let mut p = n.clone();
        loop{
            let pp = p.parent.borrow().upgrade();
            if pp.is_none() {
                break; }
            p = pp.unwrap();
        }
        let mut vec = vec![p.clone()];
        loop{
            let p = Self::traverse(&mut vec);
            if p.is_some() && p.clone().unwrap().is_scene() {
                let prev_bgimg = p.clone().unwrap().get_scene_bgimg();
                if prev_bgimg.is_none() { continue; }
                let prev_abs_file = prev_base_dir.join( prev_bgimg.unwrap() );
                let prev_abs_file = dunce::canonicalize(&prev_abs_file).expect("canonicalize");
                if let Ok(new_relpath) = prev_abs_file.strip_prefix(new_base_dir) {
                    p.unwrap().set_scene_bgimg(Some(new_relpath.to_path_buf()));
                } else {
                    p.unwrap().set_scene_bgimg(Some(prev_abs_file));
                }
            } else if p.is_none() {
                break;
            } else {
                continue;
            }
        }
    }
    // from_serde //////////////////////////////////////////
    pub fn from_serde(ser: Vec<ScenarioNodeSerde>) -> Option<Rc<ScenarioNode>> {

        let mut stack: Vec<Rc<ScenarioNode>> = Vec::new();
        let mut head: Option<Rc<ScenarioNode>> = None;
        let mut ser = VecDeque::from(ser);
        let mut root: Option<Rc<ScenarioNode>> = None;

        loop{
            let node = ser.pop_front();
            if node.is_none(){ break; }
            let node = node.unwrap();

            println!("(from_serde) {}", node);

            let has_n_and_c = node.has_n_and_c.get();
            let sn = Rc::new(ScenarioNode::from(node));
            if head.is_some() {
                if sn.bt.get() == BranchType::Neighbor {
                    Self::mv_to_neighbor(head.clone().unwrap(), sn.clone());
                } else {
                    Self::mv_to_child(head.clone().unwrap(), sn.clone());
                }
            } else { // root
                root = Some(sn.clone());
            }
            head = Some(sn.clone());

            match has_n_and_c {
                HasBranches::None => {
                    if stack.len() == 0 {
                        break;
                    }
                    head = stack.pop();
                },
                HasBranches::Both => {
                    stack.push( sn.clone() );
                },
                _ => ()
            }

        }
        root
    }
    // mv_to_parent ////////////////////////////////////////
    /// make B a child/neighbor of A's parent
    pub fn mv_to_parent(a: Rc::<ScenarioNode>, b: Rc<ScenarioNode>) -> bool{
        if Rc::ptr_eq(&a, &b){ return true; }

        if (!ScenarioNode::can_be_child( &*b.value.borrow()/*p*/, &*a.value.borrow()))/*c*/ &&
            (!ScenarioNode::can_be_neighbor( &*b.value.borrow()/*p*/, &*a.value.borrow()))/*c*/ {
            return false;
        }

        // 1. remove B
        b.remove();
        // 2. set the parent of A to B,
        //   and the branch type of B to A's bt,
        //   and the branch tyep of A to neighbor
        b.set_bt(a.bt.get());
        a.set_bt(BranchType::Neighbor);
        // 3. set the neighbor of B to A
        b.set_neighbor(a.clone());

        if let Some(a_p) = (*a.parent.borrow_mut()).upgrade(){
            // 4. if exists, the neighbor of A's parent to B
            a_p.set_neighbor( b.clone() );
            // 5. if exists, set the parent of B to A's parent or empty.
            b.set_parent(Rc::downgrade(&a_p));
        } else {
            b.set_parent(Weak::new());
        }
        // 6. set the parent of A to B
        a.set_parent(Rc::downgrade(&b));
        true
    }
    // mv_to_child /////////////////////////////////////////
    /// make B a child node of A
    pub fn mv_to_child(a: Rc::<ScenarioNode>, b: Rc<ScenarioNode>) -> bool{
        if Rc::ptr_eq(&a, &b){ return true; }

        if !ScenarioNode::can_be_child( &*a.value.borrow()/*p*/, &*b.value.borrow())/*c*/  {
            return false;
        }

        // 1. remove B
        b.remove();
        // 2. set the parent of B to A, and the branch type of B to child
        b.set_parent(Rc::downgrade(&a));
        b.set_bt(BranchType::Child);
        // 3. set the neighbor of B to {the child of A or None},
        if let Some(a_c) = (*a.child.borrow_mut()).as_ref(){
            a_c.set_bt(BranchType::Neighbor);
            b.set_neighbor(a_c.clone());
        } else {
            b.unset_neighbor();
        }
        // 4. if exists, set the parent of child of A
        //    (in this timing, already B's neighbor) to B
        if let Some(b_n) = (*b.neighbor.borrow_mut()).as_ref(){
            b_n.set_parent(Rc::downgrade(&b));
        }
        // 5. set the child of A to B
        a.set_child(b.clone());

        true
    }
    // mv_to_neighbor //////////////////////////////////////
    /// make B a neighbor of A
    pub fn mv_to_neighbor(a: Rc::<ScenarioNode>, b: Rc<ScenarioNode>) -> bool{
        if Rc::ptr_eq(&a, &b){ return true; }

        if !ScenarioNode::can_be_neighbor( &*a.value.borrow()/*p*/, &*b.value.borrow())/*c*/  {
            return false;
        }

        // 1. remove B
        b.remove();
        // 2. set the parent of B to A, and the branch type of B to neighbor
        b.set_parent(Rc::downgrade(&a));
        b.set_bt(BranchType::Neighbor);
        // 3. set the neighbor of B to {the neighbor of A or None},
        if let Some(a_n) = (*a.neighbor.borrow_mut()).as_ref(){
            a_n.set_bt(BranchType::Neighbor);
            b.set_neighbor(a_n.clone());
        } else {
            b.unset_neighbor();
        }
        // 4. if exists, set the parent of neighbor of A
        //    (in this timing, already B's neighbor) to B
        if let Some(b_n) = (*b.neighbor.borrow_mut()).as_ref(){
            b_n.set_parent(Rc::downgrade(&b));
        }
        // 5. set the neighbor of A to B
        a.set_neighbor(b.clone());

        true
    }
    // search_def_label ////////////////////////////////////
    fn extract_label(item: &impl LabelledItem) -> Option<String> {
        if (item.get_label_type() == LabelType::Ref ||
            item.get_label_type() == LabelType::RefNoRect) &&
            item.get_label().as_ref().is_some() {
                item.get_label()
            } else {
                None
            }
    }
    fn detect_def_label(item: &impl LabelledItem, lbl: &str) -> bool{
        if (item.get_label_type() == LabelType::Def) &&
            (item.get_label().as_ref().unwrap().as_str() == lbl){
                true
            } else {
                false
            }
    }
    pub fn search_def_label(sn: Rc<ScenarioNode>) -> Option<Rc<ScenarioNode>>{
        // extracts label name of ref
        let sn_value = sn.value.borrow();
        let lbl =
            match &*sn_value {
                Item::Mat(m) | Item::Pmat(m) => {
                    Self::extract_label(m) },
                Item::Scene(s) => {
                    Self::extract_label(s) },
                _ => { return None; }
            };
        let lbl = if let Some(l) = lbl { l } else { return None; };
        Self::search_def_label_node(sn.clone(), |item| {
            match item{
                Item::Mat(m) | Item::Pmat(m) => {
                    if sn.is_mat() || sn.is_pmat() {
                        Self::detect_def_label(m, &lbl) }
                    else {
                        false }
                },
                Item::Scene(s) => {
                    if sn.is_scene() {
                        Self::detect_def_label(s, &lbl) }
                    else {
                        false }
                },
                _ => false
            }
        })
    }
    // traverse ////////////////////////////////////////////
    pub fn traverse(vec: &mut Vec<Rc<ScenarioNode>>) -> Option<Rc<ScenarioNode>>{
        if vec.len() == 0 {
            return None; }
        let p = vec.pop().unwrap();

        let p_temp = p.clone();
        let pp = p_temp.neighbor.borrow();
        if pp.is_some() {
            vec.push( pp.as_ref().unwrap().clone() ); }
        let pp = p_temp.child.borrow();
        if pp.is_some() {
            vec.push( pp.as_ref().unwrap().clone() ); }
        Some(p)
    }
    // search_def_label_node ///////////////////////////////
    pub fn search_def_label_node(n: Rc<ScenarioNode>,
                                 predicate: impl Fn(&'_ Item) -> bool) -> Option<Rc<ScenarioNode>>{
        let mut p = n.clone();
        loop{
            let pp = p.parent.borrow().upgrade();
            if pp.is_none() {
                break; }
            p = pp.unwrap();
        }
        // depth-first traversal with stack
        let mut vec = vec![p.clone()];

        loop{
            let p = Self::traverse(&mut vec);
            if p.is_some(){
                if predicate( &*p.clone().unwrap().value.borrow() ){
                    return Some(p.unwrap().clone());
                }
            } else {
                break;
            }
        }
        None
    }
    // can_be_child ////////////////////////////////////////
    pub fn can_be_child(p: &Item, c: &Item) -> bool{ // can c become p's child?
        match p{
            Item::Group => {
                match c {
                    Item::Group    => true,
                    Item::Scene(_) => true,
                    Item::Page(_)  => false,
                    Item::Mat(_)   => false,
                    Item::Ovimg(_) => false,
                    Item::Pmat(_)  => false,
                }
            },
            Item::Scene(_) => {
                match c {
                    Item::Page(_)  => true,
                    Item::Pmat(_)  => true,
                    Item::Mat(_)   => true,
                    _ => false,
                }
            },
            Item::Page(_) => {
                match c {
                    Item::Mat(_)   => true,
                    Item::Ovimg(_) => true,
                    _ => false,
                }
            },
            _ => false
        }
    }
    // can_be_neighbor /////////////////////////////////////
    pub fn can_be_neighbor(p: &Item, n: &Item) -> bool{ // can c become p's neighbor?
        match p{
            Item::Group => {
                match n {
                    Item::Group    => true,
                    Item::Scene(_) => true,
                    Item::Page(_)  => false,
                    Item::Mat(_)   => false,
                    Item::Ovimg(_) => false,
                    Item::Pmat(_)  => false,
                }
            },
            Item::Scene(_) => {
                match n {
                    Item::Group    => true,
                    Item::Scene(_) => true,
                    _ => false,
                }
            },
            Item::Page(_) => {
                match n {
                    Item::Page(_)   => true,
                    Item::Pmat(_)   => true,
                    Item::Mat(_)    => false,
                    _ => false,
                }
            },
            Item::Mat(_) => {
                match n {
                    Item::Pmat(_)  => true,
                    Item::Mat(_)   => true,
                    Item::Ovimg(_) => true,
                    _ => false,
                }
            },
            Item::Ovimg(_) => {
                match n {
                    Item::Mat(_)   => true,
                    Item::Ovimg(_) => true,
                    _ => false,
                }
            },
            Item::Pmat(_) => {
                match n {
                    Item::Page(_) => true,
                    Item::Pmat(_) => true,
                    Item::Mat(_)  => true,
                    _ => false,
                }
            },
        }
    }
    // can_be_neighbor_or_child_auto ///////////////////////
    pub fn can_be_neighbor_or_child_auto(p: &Item, n: &Item) -> bool{
        match p{
            Item::Group => {
                match n {
                    Item::Group    => true,
                    Item::Scene(_) => true,
                    _ => false,
                }
            },
            Item::Scene(_) => {
                match n {
                    Item::Group    => true,
                    Item::Scene(_) => true,
                    Item::Page(_)  => true,
                    Item::Mat(_)   => true,
                    Item::Ovimg(_) => false,
                    Item::Pmat(_)  => true,
                }
            },
            Item::Page(_) => {
                match n {
                    Item::Group    => true,
                    Item::Scene(_) => true,
                    Item::Page(_)  => true,
                    Item::Mat(_)   => true,
                    Item::Ovimg(_) => true,
                    Item::Pmat(_)  => true,
                }
            },
            Item::Mat(_) | Item::Ovimg(_)=> {
                match n {
                    Item::Group    => true,
                    Item::Scene(_) => true,
                    Item::Page(_)  => true,
                    Item::Mat(_)   => true,
                    Item::Ovimg(_) => true,
                    Item::Pmat(_)  => true,
                }
            },
            Item::Pmat(_) => {
                match n {
                    Item::Group    => true,
                    Item::Scene(_) => true,
                    Item::Page(_)  => true,
                    Item::Mat(_)   => false,
                    Item::Ovimg(_) => false,
                    Item::Pmat(_)  => true,
                }
            },
        }
    }
    // get_label_type //////////////////////////////////
    pub fn get_label_type(&self) -> Option<LabelType>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.lbl_type )
            },
            Item::Scene(s) => {
                Some( s.lbl_type )
            }
            _ => None,
        }
    }
    // set_label_type //////////////////////////////////
    pub fn set_label_type(&self, lt: LabelType) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.lbl_type = lt;
            },
            Item::Scene(ref mut s) => {
                s.lbl_type = lt;
            },
            _ => (),
        }
    }
    // get_label ///////////////////////////////////////
    pub fn get_label(&self) -> Option<String>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                m.lbl.clone()
            },
            Item::Scene(s) => {
                s.lbl.clone()
            },
            _ => None,
        }
    }
    // set_mat_type //////////////////////////////////
    pub fn set_label(&self, lbl: Option<String>) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.lbl = lbl;
            },
            Item::Scene(ref mut s) => {
                s.lbl = lbl;
            }
            _ => ()
        }
    }
    //// mat ///////////////////////////////////////////////
    // mat_pos_dim /////////////////////////////////////////
    pub fn get_mat_pos_dim_with_label(sn: Rc::<ScenarioNode>) -> Option<(i32, i32, i32, i32)>{
        if let Some(sn_label_source) = Self::search_def_label(sn.clone()){
            sn_label_source.get_mat_pos_dim()
        } else {
            sn.get_mat_pos_dim()
        }

    }
    pub fn get_mat_pos_dim(&self) -> Option<(i32, i32, i32, i32)>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( (m.pos.x, m.pos.y, m.dim.w, m.dim.h) )
            },
            _ => None,
        }
    }
    pub fn set_mat_pos_dim(&self, x: i32, y: i32, w: i32, h: i32){
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.pos.x = x; m.pos.y = y;
                m.dim.w = w; m.dim.h = h;
            },
            _ => ()
        }
    }
    pub fn get_mat_pos_dim_f64(&self) -> Option<(f64, f64, f64, f64)>{
        if let Some((x, y, w, h)) = self.get_mat_pos_dim() {
            Some((x as f64, y as f64, w as f64, h as f64))
        } else {
            None
        }
    }
    // mat_rgba ////////////////////////////////////////////
    pub fn get_mat_rgba(&self) -> Option<Vec<u32>>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( vec![m.col.r, m.col.g, m.col.b, m.a] )
            },
            _ => None,
        }
    }
    pub fn get_mat_rgba_tuple_f64(&self) -> Option<(f64, f64, f64, f64)>{
        let col_v =
            if let Some(v) = self.get_mat_rgba() { v } else { return None; };
        let col_v: Vec<_> = col_v.iter().map(|c|{ (*c as f64) / 255.0 }).collect();
        Some((col_v[0], col_v[1], col_v[2], col_v[3]))
    }
    pub fn set_mat_rgba(&self, rgba: Vec<u32>){
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.col.r = rgba[0];
                m.col.g = rgba[1];
                m.col.b = rgba[2];
                m.a     = rgba[3];
            },
            _ => ()
        }
    }
    // mat_text ////////////////////////////////////////
    pub fn get_mat_text(&self) -> Option<String>{
        match &(*self.value.borrow()){
            Item::Mat(ref m) | Item::Pmat(ref m) => {
                Some(m.text.clone())
            },
            _ => None,
        }
    }
    pub fn set_mat_text(&self, text: &str) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.text = text.to_string();
            },
            _ => ()
        }
    }
    // mat_font_rgba ///////////////////////////////////
    pub fn get_mat_font_rgba(&self) -> Option<Vec<u32>>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( vec![m.font_col.r, m.font_col.g, m.font_col.b, m.font_a] )
            },
            _ => None,
        }
    }
    pub fn get_mat_font_rgba_tuple_f64(&self) -> Option<(f64, f64, f64, f64)>{
        let col_v =
            if let Some(v) = self.get_mat_font_rgba() { v } else { return None; };
        let col_v: Vec<_> = col_v.iter().map(|c|{ (*c as f64) / 255.0 }).collect();
        Some((col_v[0], col_v[1], col_v[2], col_v[3]))
    }
    // mat_font_family /////////////////////////////////
    pub fn get_mat_font_family(&self) -> Option<String>{
        match &(*self.value.borrow()){
            Item::Mat(ref m) | Item::Pmat(ref m) => {
                Some(m.font_family.clone())
            },
            _ => None,
        }
    }
    pub fn set_mat_font_family(&self, font_family: &str) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.font_family = font_family.to_string();
            },
            _ => ()
        }
    }
    // mat_font_size ///////////////////////////////////
    pub fn get_mat_font_size(&self) -> Option<i32>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.font_size )
            },
            _ => None,
        }
    }
    pub fn set_mat_font_size(&self, font_size: i32) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.font_size = font_size;
            },
            _ => ()
        }
    }
    // mat_font_rgba //////////////////////////////////
    pub fn set_mat_font_rgba(&self, rgba: Vec<u32>){
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.font_col.r = rgba[0];
                m.font_col.g = rgba[1];
                m.font_col.b = rgba[2];
                m.font_a     = rgba[3];
            },
            _ => ()
        }
    }
    // mat_font_weight /////////////////////////////////////
    pub fn get_mat_font_weight(&self) -> Option<String>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.font_weight.clone() )
            },
            _ => None,
        }
    }
    pub fn set_mat_font_weight(&self, w: String) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.font_weight = w;
            },
            _ => ()
        }
    }
    // mat_font_rgba_2 /////////////////////////////////
    pub fn get_mat_font_rgba_2(&self) -> Option<Vec<u32>>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( vec![m.font_col_2.r, m.font_col_2.g, m.font_col_2.b, m.font_a_2] )
            },
            _ => None,
        }
    }
    pub fn get_mat_font_rgba_tuple_f64_2(&self) -> Option<(f64, f64, f64, f64)>{
        let col_v =
            if let Some(v) = self.get_mat_font_rgba_2() { v } else { return None; };
        let col_v: Vec<_> = col_v.iter().map(|c|{ (*c as f64) / 255.0 }).collect();
        Some((col_v[0], col_v[1], col_v[2], col_v[3]))
    }
    pub fn set_mat_font_rgba_2(&self, rgba: Vec<u32>){
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.font_col_2.r = rgba[0];
                m.font_col_2.g = rgba[1];
                m.font_col_2.b = rgba[2];
                m.font_a_2     = rgba[3];
            },
            _ => ()
        }
    }
    // mat_font_outl_2 /////////////////////////////////
    pub fn get_mat_font_outl_2(&self) -> Option<f64>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.font_outl_2 )
            },
            _ => None,
        }
    }
    pub fn set_mat_font_outl_2(&self, font_outl: f64) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.font_outl_2 = font_outl;
            },
            _ => ()
        }
    }
    // mat_font_weight_2 ///////////////////////////////////
    pub fn get_mat_font_weight_2(&self) -> Option<String>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.font_weight_2.clone() )
            },
            _ => None,
        }
    }
    pub fn set_mat_font_weight_2(&self, w: String) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.font_weight_2 = w;
            },
            _ => ()
        }
    }
    // mat_vertical ////////////////////////////////////////
    pub fn get_mat_vertical(&self) -> Option<bool>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.vertical )
            },
            _ => None,
        }
    }
    pub fn set_mat_vertical(&self, v: bool) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.vertical = v;
            },
            _ => ()
        }
    }
    // mat_line_spacing ////////////////////////////////////
    pub fn get_mat_line_spacing(&self) -> Option<f32>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.line_spacing )
            },
            _ => None,
        }
    }
    pub fn set_mat_line_spacing(&self, s: f32) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.line_spacing = s;
            },
            _ => ()
        }
    }
    // mat_r ///////////////////////////////////////////////
    pub fn get_mat_r(&self) -> Option<i32>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.r )
            },
            _ => None,
        }
    }
    pub fn set_mat_r(&self, r: i32) {
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.r = r;
            },
            _ => ()
        }
    }
    // mat_bgimg /////////////////////////////////////
    pub fn get_mat_bgimg(&self) -> Option<PathBuf>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                if let Some(b) = &m.bgimg { Some(b.clone()) }
                else { None }
            },
            _ => None,
        }
    }
    pub fn set_mat_bgimg(&self, p: Option<PathBuf>){
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.bgimg = p;
            },
            _ => ()
        }
    }
    // mat_bg_en /////////////////////////////////////////
    pub fn get_mat_bg_en(&self) -> Option<bool>{
        match &(*self.value.borrow()) {
            Item::Mat(m) | Item::Pmat(m) => {
                Some( m.bg_en )
            },
            _ => None,
        }
    }
    pub fn set_mat_bg_en(&self, en: bool){
            match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.bg_en = en;
            },
            _ => ()
        }
    }
    // mat_text_offset /////////////////////////////////////
    pub fn get_mat_text_pos(&self) -> Option<(i32, i32)>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( (m.text_pos.x, m.text_pos.y) )
            },
            _ => None,
        }
    }
    pub fn get_mat_text_pos_f64(&self) -> Option<(f64, f64)>{
        match &(*self.value.borrow()){
            Item::Mat(m) | Item::Pmat(m) => {
                Some( (m.text_pos.x as f64, m.text_pos.y as f64) )
            },
            _ => None,
        }
    }
    pub fn set_mat_text_pos(&self, x: i32, y: i32){
        match *self.value.borrow_mut(){
            Item::Mat(ref mut m) | Item::Pmat(ref mut m) => {
                m.text_pos.x = x; m.text_pos.y = y;
            },
            _ => ()
        }
    }
    //// scene /////////////////////////////////////////////
    // scene_bgcol ////////////////////////////////////
    pub fn get_scene_bgcol(&self) -> Option<Vec<u32>>{
        match &(*self.value.borrow()){
            Item::Scene(s) => {
                Some( vec![s.bgcol.r, s.bgcol.g, s.bgcol.b] )
            },
            _ => None,
        }
    }
    // scene_crop_pos_dim //////////////////////////////
    pub fn get_scene_crop_pos_dim(&self) -> Option<(i32, i32, i32, i32, bool)>{
        match &(*self.value.borrow()) {
            Item::Scene(s) => {
                Some( (s.crop.pos.x, s.crop.pos.y,
                       s.crop.dim.w, s.crop.dim.h,
                       s.crop_en) )
            },
            _ => None,
        }
    }
    pub fn set_scene_crop_pos_dim(&self, x: i32, y: i32, w: i32, h: i32){
        match *self.value.borrow_mut(){
            Item::Scene(ref mut s) => {
                s.crop.pos.x = x; s.crop.pos.y = y;
                s.crop.dim.w = w; s.crop.dim.h = h;
            },
            _ => ()
        }
    }
    // scene_crop_en ///////////////////////////////////
    pub fn get_scene_crop_en(&self) -> Option<bool>{
        match &(*self.value.borrow()) {
            Item::Scene(s) => {
                Some( s.crop_en )
            },
            _ => None,
        }
    }
    pub fn set_scene_crop_en(&self, en: bool){
            match *self.value.borrow_mut(){
            Item::Scene(ref mut s) => {
                s.crop_en = en;
            },
            _ => ()
        }
    }
    // scene_bgimg /////////////////////////////////////
    pub fn get_scene_bgimg(&self) -> Option<PathBuf>{
        match &(*self.value.borrow()){
            Item::Scene(s) => {
                if let Some(b) = &s.bgimg { Some(b.clone()) }
                else { None }
            },
            _ => None,
        }
    }
    pub fn set_scene_bgimg(&self, p: Option<PathBuf>){
        match *self.value.borrow_mut(){
            Item::Scene(ref mut s) => {
                s.bgimg = p;
            },
            _ => ()
        }
    }
    // scene_bg_en /////////////////////////////////////////
    pub fn get_scene_bg_en(&self) -> Option<bool>{
        match &(*self.value.borrow()) {
            Item::Scene(s) => {
                Some( s.bg_en )
            },
            _ => None,
        }
    }
    pub fn set_scene_bg_en(&self, en: bool){
            match *self.value.borrow_mut(){
            Item::Scene(ref mut s) => {
                s.bg_en = en;
            },
            _ => ()
        }
    }
    // scene_bg_rgba ///////////////////////////////////
    pub fn get_scene_bg_rgba(&self) -> Option<Vec<u32>>{
        match &(*self.value.borrow()){
            Item::Scene(s) => {
                Some( vec![s.bgcol.r, s.bgcol.g, s.bgcol.b, 255] )
            },
            _ => None,
        }
    }
    pub fn set_scene_bg_rgba(&self, rgba: Vec<u32>){
        match *self.value.borrow_mut(){
            Item::Scene(ref mut s) => {
                s.bgcol.r = rgba[0];
                s.bgcol.g = rgba[1];
                s.bgcol.b = rgba[2];
            },
            _ => ()
        }
    }
    // is_mat //////////////////////////////////////////////
    pub fn is_mat(&self) -> bool{
        if let Item::Mat(_) = &(*self.value.borrow()) { true } else { false } }
    // is_scene ////////////////////////////////////////////
    pub fn is_scene(&self) -> bool{
        if let Item::Scene(_) = &(*self.value.borrow()) { true } else { false } }
    // is_group ////////////////////////////////////////////
    pub fn is_group(&self) -> bool{
        if let Item::Group = &(*self.value.borrow()) { true } else { false } }
    // is_pmat /////////////////////////////////////////////
    pub fn is_pmat(&self) -> bool{
        if let Item::Pmat(_) = &(*self.value.borrow()) { true } else { false } }
}
// BranchType //////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BranchType{ Child, Neighbor, }
// Item ////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Item{
    Group,
    Scene(Scene),
    Page(Page),
    Mat(Mat),
    Ovimg(Ovimg),
    Pmat(Mat),
}
impl Default for Item{
    fn default() -> Self{
        Item::Group
    }
}
// Page ///////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub name : String,
}
impl Default for Page {
    fn default() -> Self {Self{name: "".to_string()}} }
// Color ///////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r : u32,
    pub g : u32,
    pub b : u32,
}
impl Default for Color{
    fn default() -> Self { Self{r: 128, g: 128, b: 128} } }
// Position ////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x : i32,
    pub y : i32,
}
impl Default for Position{
    fn default() -> Self{ Self{x: 0, y: 0} }
}
impl Position{
    fn from_xy(x: i32, y: i32) -> Self{ Self{x, y} }
}
// Dimension ///////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub w : i32,
    pub h : i32,
}
impl Default for Dimension{
    fn default() -> Self{ Self{w: 100, h: 100} } }
// Ovimg ///////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ovimg {
    pub path  : String,
    pub pos   : Position,
    pub a     : f64,
}
impl Default for Ovimg {
    fn default() -> Self{ Self{path: "".to_string(), pos: Position::default(), a: 1.0} }
}
// CropInfo ////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropInfo {
    pub pos : Position,
    pub dim : Dimension
}
// LabelledItem ///////////////////////////////////////////
pub trait LabelledItem {
    fn get_label(&self) -> Option<String>;
    fn get_label_type(&self) -> LabelType;
}
// Scene ///////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub bgimg    : Option<PathBuf>,
    pub bg_en    : bool,
    pub bgcol    : Color,
    pub crop     : CropInfo,
    pub crop_en  : bool,
    pub lbl      : Option<String>,
    pub lbl_type : LabelType,
}
impl Default for Scene{
    fn default() -> Self{
        Self{
            bgimg    : None,
            bg_en    : true,
            bgcol    : Color::default(),
            crop     : CropInfo{ pos: Position::default(), dim: Dimension::default() },
            crop_en  : false,
            lbl      : None,
            lbl_type : LabelType::None,
        }
    }
}
impl LabelledItem for Scene{
    fn get_label(&self) -> Option<String>{
        self.lbl.clone() }
    fn get_label_type(&self) -> LabelType {
        self.lbl_type }
}
// Mat /////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum LabelType { None, Def, Ref, RefNoRect }
impl fmt::Display for LabelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s= String::from(
            match self{
                LabelType::None      => "n",
                LabelType::Def       => "d",
                LabelType::Ref       => "r",
                LabelType::RefNoRect => "rnr",
            });
        write!(f, "{}", s)
    }
}
impl LabelType{
    pub fn pretty_format(&self) -> &str{
        match self{
            LabelType::None      => "None",
            LabelType::Def       => "Def",
            LabelType::Ref       => "Ref",
            LabelType::RefNoRect => "RefNoRect"
        }
    }
    pub fn from(s: &str) -> Self{
        match s{
            "None"      => LabelType::None,
            "Def"       => LabelType::Def,
            "Ref"       => LabelType::Ref,
            "RefNoRect" => LabelType::RefNoRect,
            _           => LabelType::None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mat {
    pub col           : Color,
    pub pos           : Position,
    pub dim           : Dimension,
    pub r             : i32,
    pub a             : u32,
    pub src           : Option<String>,
    pub lbl           : Option<String>,
    pub lbl_type      : LabelType,
    pub name          : String, // this field is only for debug
    pub font_col      : Color,
    pub font_a        : u32,
    pub font_weight   : String,
    pub font_col_2    : Color,
    pub font_a_2      : u32,
    pub font_outl_2   : f64,
    pub font_weight_2 : String,
    pub font_size     : i32,
    pub font_family   : String,
    pub line_spacing  : f32,
    pub vertical      : bool,
    pub text          : String,
    #[serde(default)]
    pub bgimg         : Option<PathBuf>,
    #[serde(default)]
    pub bg_en         : bool,
    #[serde(default)]
    pub text_pos      : Position,
}
impl Mat {
    pub fn dump(&self) {
        println!{"    col= {:?}", self.col};
        println!{"    pos= {:?}", self.pos};
        println!{"    dim= {:?}", self.dim};
        println!{"    r= {}, a= {}", self.r, self.a};
        print_opt_str(&self.src,      String::from("    src"));
        print_opt_str(&self.lbl,      String::from("    lbl"));
        println!("    ltype= {:?}", self.lbl_type)
    }
}
impl Default for Mat{
    fn default() -> Self{
        Self{
            col           : Color{ r:105, g:124, b:144 },
            pos           : Position::default(),
            dim           : Dimension::default(),
            r             : 8,
            a             : 134,
            src           : None,
            lbl           : None,
            lbl_type      : LabelType::None,
            name          : "mat".to_string(),
            text          : "text".to_string(),
            font_col      : Color{ r:0, g:0, b:0 },
            font_a        : 162,
            font_weight   : "Bold".to_string(),
            font_col_2    : Color{r: 255, g: 255, b: 255},
            font_a_2      : 177,
            font_outl_2   : 4.0,
            font_weight_2 : "Normal".to_string(),
            font_size     : 22,
            font_family   : String::from("Rounded M+ 1m"),
            line_spacing  : 0.8,
            vertical      : false,
            bgimg         : None,
            bg_en         : false,
            text_pos      : Position::from_xy(0, 0),
        }
    }
}
impl LabelledItem for Mat{
    fn get_label(&self) -> Option<String>{
        self.lbl.clone() }
    fn get_label_type(&self) -> LabelType {
        self.lbl_type }
}
// print_opt_str ///////////////////////////////////////////
fn print_opt_str(a: &Option<String>, prefix: String){
    if let Some(s) = a {
        println!("{}= {}", prefix, s); }
    else {
        println!("{}= none", prefix) }
}

// debug
// impl Drop for ScenarioNode {
//     fn drop(&mut self) {
//         println!("> Dropping {}", self.id.get());
//     }
// }
