pub mod text_edit{
    use gtk::MovementStep;
    use gtk::DeleteType;
    use gtk::ApplicationWindow;
    use gtk::glib::VariantTy;
    use gtk::gio::SimpleAction;
    use gtk::TextView;
    use gtk::prelude::*;

    pub const ACT_CURSOR_MOVE : &str = "move_cursor";
    #[derive(Debug, Clone, Copy)]
    pub enum ActCursorCmd {
        FwdChar,    BackChar,
        FwdWord,    BackWord,
        NextLine,   PrevLine,
        NextLine3,  PrevLine3,
        BegLine,    EndLine,
    }

    pub const ACT_DEL_TEXT : &str = "del_text";
    #[derive(Debug, Clone, Copy)]
    pub enum ActDelTextCmd {
        DelBackChar,  DelChar,
        KillLine,
        BackKillWord, KillWord,
    }

    pub const ACT_INS_TEXT : &str = "ins_text";
    #[derive(Debug, Clone, Copy)]
    pub enum ActInsTextCmd {
        NewLine, OpenLine, Dakuten
    }

    pub const ACT_C_N_P_TEXT : &str = "c_n_p_text";
    #[derive(Debug, Clone, Copy)]
    pub enum ActCnPTextCmd {
        Copy, Cut, Paste
    }

    // act_cursor_move /////////////////////////////////////
    pub fn act_cursor_move(win: ApplicationWindow) -> SimpleAction{
        let act = SimpleAction::new(ACT_CURSOR_MOVE, Some(&VariantTy::INT32));
        act.connect_activate(move|_act, val|{
            let current_focus =
                if let Some(f) = GtkWindowExt::focus(&win) {f} else {return;};
            let val = val.expect("expect val").get::<i32>().expect("couldn't get i32 val");
            let val = match val {
                x if x == ActCursorCmd::FwdChar   as i32 => {(MovementStep::VisualPositions,  1)},
                x if x == ActCursorCmd::BackChar  as i32 => {(MovementStep::VisualPositions, -1)},
                x if x == ActCursorCmd::FwdWord   as i32 => {(MovementStep::Words,            1)},
                x if x == ActCursorCmd::BackWord  as i32 => {(MovementStep::Words,           -1)},
                x if x == ActCursorCmd::NextLine  as i32 => {(MovementStep::DisplayLines,     1)},
                x if x == ActCursorCmd::PrevLine  as i32 => {(MovementStep::DisplayLines,    -1)},
                x if x == ActCursorCmd::NextLine3 as i32 => {(MovementStep::DisplayLines,     3)},
                x if x == ActCursorCmd::PrevLine3 as i32 => {(MovementStep::DisplayLines,    -3)},
                x if x == ActCursorCmd::BegLine   as i32 => {(MovementStep::DisplayLineEnds, -1)},
                x if x == ActCursorCmd::EndLine   as i32 => {(MovementStep::DisplayLineEnds,  1)},
                _ => { println!("(act_cursor_move) unexpected val: {}", val); return; }
            };

            if let Some(view) = current_focus.downcast_ref::<TextView>(){
                view.emit_move_cursor(val.0, val.1, false);
            } else if let Some(text) = current_focus.downcast_ref::<gtk::Text>(){
                text.emit_move_cursor(val.0, val.1, false);
            }
        });
        act
    }
    // act_delete_text ///////////////////////////////////////
    pub fn act_delete_text(win: ApplicationWindow) -> SimpleAction{
        let act = SimpleAction::new(ACT_DEL_TEXT, Some(&VariantTy::INT32));
        act.connect_activate(move|_act, val|{
            let current_focus =
                if let Some(f) = GtkWindowExt::focus(&win) {f} else {return;};
            let val = val.expect("expect val").get::<i32>().expect("couldn't get i32 val");
            let val = match val {
                x if x == ActDelTextCmd::DelBackChar as i32 => {(DeleteType::Chars, -1)},
                x if x == ActDelTextCmd::DelChar     as i32 => {(DeleteType::Chars,  1)},
                x if x == ActDelTextCmd::KillLine    as i32 => {
                    // workaround
                    // TODO: remove the line(=Delete one char) when the line has empty
                    if let Some(view) = current_focus.downcast_ref::<TextView>(){
                        view.emit_move_cursor(MovementStep::DisplayLineEnds, 1, true);
                        view.emit_delete_from_cursor(DeleteType::DisplayLineEnds, 1);
                    }
                    (DeleteType::DisplayLineEnds, 1)
                },
                x if x == ActDelTextCmd::BackKillWord as i32 => {(DeleteType::WordEnds, -1)},
                x if x == ActDelTextCmd::KillWord     as i32 => {(DeleteType::WordEnds,  1)},
                _ => { println!("(act_edit_text) unexpected val: {}", val); return; }
            };

            if let Some(view) = current_focus.downcast_ref::<TextView>(){
                view.emit_delete_from_cursor(val.0, val.1);
            } else if let Some(text) = current_focus.downcast_ref::<gtk::Text>(){
                text.emit_delete_from_cursor(val.0, val.1);
            }
        });
        act
    }
    // act_insert_text /////////////////////////////////////
    pub fn act_insert_text(win: ApplicationWindow) -> SimpleAction{
        let act = SimpleAction::new(ACT_INS_TEXT, Some(&VariantTy::INT32));
        act.connect_activate(move|_act, val|{
            let current_focus =
                if let Some(f) = GtkWindowExt::focus(&win) {f} else {return;};
            let view =
                if let Some(v) = current_focus.downcast_ref::<TextView>() {v} else {return;};
            let val = val.expect("expect val").get::<i32>().expect("couldn't get i32 val");

            match val{
                x if x == ActInsTextCmd::NewLine as i32 => {
                    view.emit_insert_at_cursor("\n");
                },
                x if x == ActInsTextCmd::OpenLine as i32 =>{
                    view.emit_insert_at_cursor("\n");
                    view.emit_move_cursor(MovementStep::DisplayLines, -1, false);
                    view.emit_move_cursor(MovementStep::DisplayLineEnds, 1, false);
                },
                x if x == ActInsTextCmd::Dakuten as i32 => {
                    view.emit_insert_at_cursor("゛");
                },
                _ => { println!("(act_insert_text) unexpected val: {}", val); }
            }
        });
        act
    }
    // act_c_n_p_text //////////////////////////////////////
    pub fn act_c_n_p_text(win: ApplicationWindow) -> SimpleAction{
        let act = SimpleAction::new(ACT_C_N_P_TEXT, Some(&VariantTy::INT32));
        act.connect_activate(move|_act, val|{
            let current_focus =
                if let Some(f) = GtkWindowExt::focus(&win) {f} else {return;};
            let val = val.expect("expect val").get::<i32>().expect("couldn't get i32 val");

            if let Some(view) = current_focus.downcast_ref::<TextView>(){
                match val{
                    x if x == ActCnPTextCmd::Copy  as i32 => {
                        view.emit_copy_clipboard(); },
                    x if x == ActCnPTextCmd::Paste as i32 => {
                        view.emit_paste_clipboard(); },
                    x if x == ActCnPTextCmd::Cut   as i32 => {
                        view.emit_cut_clipboard(); },
                    _ => { println!("(act_c_n_p_text) unexpected val: {}", val); }
                }
            } else if let Some(text) = current_focus.downcast_ref::<gtk::Text>(){
                match val{
                    x if x == ActCnPTextCmd::Copy  as i32 => {
                        text.emit_copy_clipboard(); },
                    x if x == ActCnPTextCmd::Paste as i32 => {
                        text.emit_paste_clipboard(); },
                    x if x == ActCnPTextCmd::Cut   as i32 => {
                        text.emit_cut_clipboard(); },
                    _ => { println!("(act_c_n_p_text) unexpected val: {}", val); }
                }
            }
        });
        act
    }
}
