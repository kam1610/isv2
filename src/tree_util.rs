pub mod tree_manipulate{
    use std::rc::Rc;
    use gtk::gio::ListStore;
    use crate::scenario_node::ScenarioNode;
    use crate::scenario_node_object::ScenarioNodeObject;

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
