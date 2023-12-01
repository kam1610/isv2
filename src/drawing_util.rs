pub mod util {
    use gtk::pango::Weight;
    use std::fmt;

    // CursorState
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum CursorState{
        Nw, N, Ne, E,
        Se, S, Sw, W,
        G,
        None
    }
    // detect_edge /////////////////////////////////////////
    pub fn detect_edge(px: i32, py: i32,
                       ax: i32, ay: i32, aw: i32, ah: i32, scale: f64)
                       -> (bool/* result */,
                           CursorState, Option<String> /* cursor name*/)
    {
        let n = (6 as f64 / scale) as i32; // cursor detect margin px (TODO: parameterize)

        // The following condition in 1.--9. are based on the assumption that
        // pointer's (px,py) is within the area.
        if (ax-n <= px) && (px <= ax+aw+n) &&
            (ay-n <= py) && (py <= ay+ah+n) {
                // ok! continue to following code
            } else { // out of area
                return (false, CursorState::None, Some("normal".to_string()));
            }
        // 1. north west ///////////////////////////////
        if (ax-n <= px) && (px <= ax+n) &&
            (ay-n <= py) && (py <= ay+n) {
                return (true, CursorState::Nw, Some( "nw-resize".to_string()) );
            }
        // 2. north east ///////////////////////////////
        if (ax+aw-n <= px) && (px <= ax+aw+n) &&
            (ay-n <= py) && (py <= ay+n) {
                return (true, CursorState::Ne, Some( "ne-resize".to_string()) );
            }
        // 3. south east ///////////////////////////////
        if (ax+aw-n <= px) && (px <= ax+aw+n) &&
            (ay+ah-n <= py) && (py <= ay+ah+n) {
                return (true, CursorState::Se, Some( "se-resize".to_string() ));
            }
        // 4. south west ///////////////////////////////
        if (ax-n <= px) && (px <= ax+n) &&
            (ay+ah-n <= py) && (py <= ay+ah+n) {
                return (true, CursorState::Sw, Some( "sw-resize".to_string() ));
            }
        // 5. north ////////////////////////////////////
        if (ax <= px) && (px <= ax+aw-1) &&
            (ay-n <= py) && (py <= ay+n) {
                return (true, CursorState::N, Some( "n-resize".to_string() ));
            }
        // 6. east /////////////////////////////////////
        if (ax+aw-n <= px) && (px <= ax+aw+n) &&
            (ay <= py) && (py <= ay+ah-1) {
                return (true, CursorState::E, Some( "e-resize".to_string() ));
            }
        // 7. south ////////////////////////////////////
        if (ax <= px) && (px <= ax+aw-1) &&
            (ay+ah-n <= py) && (py <= ay+ah+n) {
                return (true, CursorState::S, Some( "s-resize".to_string() ));
            }
        // 8. west /////////////////////////////////////
        if (ax-n <= px) && (px <= ax+n) &&
            (ay <= py) && (py <= ay+ah-1) {
                return (true, CursorState::W, Some( "w-resize".to_string() ));
            }
        // 9. center(grab) /////////////////////////////
        if (ax-n <= px) && (px <= ax+aw+n) &&
            (ay-n <= py) && (py <= ay+ah+n) {
                return (true, CursorState::G, Some("move".to_string()));
            }
        return (false, CursorState::None, Some("normal".to_string()));
    }
    // gesture_update_rect /////////////////////////////////
    pub fn gesture_update_rect(x: i32, y: i32, w: i32, h: i32,
                               begin_x   : i32, begin_y   : i32,
                               mat_orig_x: i32, mat_orig_y: i32,
                               px: i32, py: i32,
                               cst: CursorState) -> (i32, i32, i32, i32){
        let (mut new_x, mut new_y, mut new_w, mut new_h) = (x, y, w, h);
        match cst {
            CursorState::Nw => {
                if px < (x + w) { new_x = px;         new_w = x + w - px; }
                if py < (y + h) { new_h = y + h - py; new_y = py;         }
            },
            CursorState::N => {
                if py < (y + h) { new_h = y + h - py; new_y = py; }
            },
            CursorState::Ne => {
                if x  < px      { new_w = px - x; }
                if py < (y + h) { new_h = y + h - py; new_y = py;         }
            },
            CursorState::E => {
                if x  < px      { new_w = px - x; }
            },
            CursorState::Se => {
                if x  < px      { new_w = px - x; }
                if y  < py      { new_h = py - y; }
            },
            CursorState::S => {
                if y  < py      { new_h = py - y; }
            },
            CursorState::Sw => {
                if px < (x + w) { new_x = px;         new_w = x + w - px; }
                if y  < py      { new_h = py - y; }
            },
            CursorState::W => {
                if px < (x + w) { new_x = px;         new_w = x + w - px; }
            },
            CursorState::G => {
                let (dx, dy) = (px - begin_x,  py - begin_y);
                if 0 < (mat_orig_x + dx) { new_x = mat_orig_x + dx; }
                if 0 < (mat_orig_y + dy) { new_y = mat_orig_y + dy; }
            }
            _ => ()
        }
        (new_x, new_y, new_w, new_h)
    }

    // get_scale_offset ////////////////////////////////////////
    pub fn get_scale_offset(src_w: i32, src_h: i32, dst_w: i32, dst_h: i32 )
                            -> (f64/*scale*/,
                                i32/*dst w*/, i32/*dst h*/, i32/*ofst x*/, i32/*ofst y*/){

        let scale = (dst_w as f64) / (src_w as f64);
        if ((src_h as f64) * scale) <= (dst_h as f64) {
            let ofst_y = (((dst_h as f64) - ((src_h as f64) * scale)) / 2.0) as i32;
            return (scale, dst_w, ((src_h as f64) * scale) as i32, 0, ofst_y);
        }

        let scale = (dst_h as f64) / (src_h as f64);
        let ofst_x = (((dst_w as f64) - ((src_w as f64) * scale)) / 2.0) as i32;
        (scale, ((src_w as f64) * scale) as i32, dst_h, ofst_x, 0)
    }
    // pango::weight ///////////////////////////////////////
    pub struct StrumWeight(pub Weight);
    impl StrumWeight {
        pub fn variants() -> Vec<(Weight, &'static str)>{
            vec![(Weight::Thin,       "Thin"       ),
                 (Weight::Ultralight, "Ultralight" ),
                 (Weight::Light,      "Light"      ),
                 (Weight::Semilight,  "Semilight"  ),
                 (Weight::Book,       "Book"       ),
                 (Weight::Normal,     "Normal"     ),
                 (Weight::Medium,     "Medium"     ),
                 (Weight::Semibold,   "Semibold"   ),
                 (Weight::Bold,       "Bold"       ),
                 (Weight::Ultrabold,  "Ultrabold"  ),
                 (Weight::Heavy,      "Heavy"      ),
                 (Weight::Ultraheavy, "Ultraheavy" ),]
        }
    }
    impl Into<StrumWeight> for Weight {
        fn into(self) -> StrumWeight {
            StrumWeight(self)
        }
    }
    impl fmt::Display for StrumWeight {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for w_s_tpl in StrumWeight::variants(){
                if w_s_tpl.0 == self.0 {
                    return write!(f, "{}", w_s_tpl.1);
                }
            };
            Err(fmt::Error)
        }
    }
    impl std::str::FromStr for StrumWeight {
        type Err = &'static str;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            for w_s_tpl in StrumWeight::variants(){
                if w_s_tpl.1 == s {
                    return Ok( StrumWeight(w_s_tpl.0) );
                }
            };
            Err("undefined weight")
        }
    }
}
