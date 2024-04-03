use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;
use std::fmt;
use gtk::Application;
use gtk::gio::Menu;
use gtk::gio::MenuItem;
use gtk::prelude::*;

pub struct KeyBind{
    entries: HashMap<String, (String, String)>,
}
impl KeyBind{
    // insert //////////////////////////////////////////////
    fn insert(&mut self, tokens: &Vec<&str>) {
        if tokens.len() != 3 { return; }
        let tokens: Vec<_> = tokens.iter().map(|t| t.trim().to_string()).collect();
        self.entries.insert(tokens[0].clone(),
                            (tokens[1].clone(), tokens[2].clone()));
    }
    // init ////////////////////////////////////////////////
    pub fn init() -> Self{
        let mut keybind = Self{ entries: HashMap::new() };
        let cur_exe_path = env::current_exe();
        if cur_exe_path.is_err(){
            println!("[KeyBind::init()] exe path is not obtained");
            return keybind;
        }
        let mut file_path = cur_exe_path.unwrap();
        file_path.pop();
        file_path.push("keybind.conf");
        let file = File::open(&file_path);
        if file.is_err(){
            println!("[KeyBind::init()] {:?} can't be opened", file_path);
            return keybind;
        }
        let file = file.unwrap();
        for line in BufReader::new(file).lines(){
            if line.is_err(){ continue; }
            let line = line.unwrap();
            let tokens: Vec<_> = line.split(',').collect();
            keybind.insert(&tokens);
        }
        keybind
    }
    // get /////////////////////////////////////////////////
    pub fn get(&self, key: &str) -> Option<&(String, String)>{
        self.entries.get(key)
    }
    // assign //////////////////////////////////////////////
    pub fn assign_acti32_and_accelkey(&self,
                                      acts       : &Vec<(&str, &str, Option<i32>)>,
                                      parent_menu: Option<&Menu>,
                                      app        : &Application,
                                      win_or_app : &str){
        let _ = acts.iter()
            .map(|act|{
                let entry;
                entry = {if let Some(e) = self.get(act.0) {e} else {
                    println!("[keybind] entry of {} is not found", act.0);
                    return;}};

                let mut act_desc = win_or_app.to_string() + act.1;
                if let Some(int_arg) = act.2 {
                    act_desc += &("(".to_string() + &int_arg.to_string() + ")");
                }

                if let Some(parent_menu) = parent_menu {
                    let menu_act = MenuItem::new(Some(&entry.0), Some(&act_desc));
                    parent_menu.append_item(&menu_act);
                }

                app.set_accels_for_action(&act_desc, &[&entry.1]);
            }).collect::<Vec<_>>();
    }
}
// fmt /////////////////////////////////////////////////////
impl fmt::Display for KeyBind{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let mut disp_str = String::new();
        for (key, val) in self.entries.iter(){
            disp_str += &format!("{}, {}, {}\n", key, val.0, val.1); }
        write!(f, "{}", disp_str)
    }
}
// test ////////////////////////////////////////////////////
#[cfg(test)]
#[macro_use]
mod test{
    use crate::KeyBind;
    use std::env;

    #[test]
    fn test_keybind_init(){
        println!("[test_keybind_init]");
        let cur_exe_path = env::current_exe().unwrap();
        println!("cur_exe_path = {:?}", cur_exe_path);
        let kb = KeyBind::init();
        println!("KeyBind: ");
        println!("{}", kb);

        println!("{:?}\n", kb.get("FwdNode"));
        println!("{:?}\n", kb.get("AddTreeNodeGroup"));
        println!("{:?}\n", kb.get("_not_inserted_"));
    }
}
