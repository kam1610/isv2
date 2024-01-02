pub mod text_edit{
    use gtk::MovementStep;
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

    // act_cursor_move /////////////////////////////////////
    pub fn act_cursor_move(win: ApplicationWindow) -> SimpleAction{
        let act = SimpleAction::new(ACT_CURSOR_MOVE, Some(&VariantTy::INT32));
        act.connect_activate(move|_act, val|{
            let val = val
                .expect("expect val")
                .get::<i32>()
                .expect("couldn't get i32 val");
            let val = match val {
                x if x == ActCursorCmd::FwdChar   as i32 => {(MovementStep::VisualPositions,  1)},
                x if x == ActCursorCmd::BackChar  as i32 => {(MovementStep::VisualPositions, -1)},
                x if x == ActCursorCmd::FwdWord   as i32 => {(MovementStep::Words,  1)},
                x if x == ActCursorCmd::BackWord  as i32 => {(MovementStep::Words, -1)},
                x if x == ActCursorCmd::NextLine  as i32 => {(MovementStep::DisplayLines,  1)},
                x if x == ActCursorCmd::PrevLine  as i32 => {(MovementStep::DisplayLines, -1)},
                x if x == ActCursorCmd::NextLine3 as i32 => {(MovementStep::DisplayLines,  3)},
                x if x == ActCursorCmd::PrevLine3 as i32 => {(MovementStep::DisplayLines, -3)},
                x if x == ActCursorCmd::BegLine   as i32 => {(MovementStep::DisplayLineEnds, -1)},
                x if x == ActCursorCmd::EndLine   as i32 => {(MovementStep::DisplayLineEnds,  1)},
                _ => { println!("(act_cursor_move) unexpected val: {}", val); return; }
            };

            let current_focus =
                if let Some(f) = GtkWindowExt::focus(&win) {f} else {return;};

            if let Some(view) = current_focus.downcast_ref::<TextView>(){
                view.emit_move_cursor(val.0, val.1, false);
            } else if let Some(text) = current_focus.downcast_ref::<gtk::Text>(){
                text.emit_move_cursor(val.0, val.1, false);
            }
        });
        act
    }


}
