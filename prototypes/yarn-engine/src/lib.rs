#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
#![allow(unused_imports)]

use asterism::{
    control::{KeyboardControl, MacroquadInputWrapper},
    linking::GraphedLinking,
    resources::InstantResources,
};
use macroquad::prelude::*;

pub struct Logics {
    pub linking: GraphedLinking<NodeID>,
    pub resources: InstantResources<RsrcPool, u16>,
    pub control: KeyboardControl<ActionID, MacroquadInputWrapper>,
}

pub type NodeID = usize;
pub type ActionID = usize;
pub type RsrcPool = usize;

impl Logics {
    fn new() -> Self {
        Self {
            linking: GraphedLinking::new(),
            resources: InstantResources::new(),
            control: KeyboardControl::new(),
            // chance logics?
            // progression logics?
        }
    }
}

// this feels very convoluted but i think it's right?
fn draw_text(text: &str, textbox: &Rect, text_params: TextParams) {
    let mut line = "".to_string();
    let mut measure_line = "".to_string();
    let mut y = textbox.y;
    for paragraph in text.lines() {
        for word in paragraph.split_whitespace() {
            measure_line += word;
            let measure = measure_text(
                &measure_line,
                Some(text_params.font),
                text_params.font_size,
                text_params.font_scale,
            );
            if measure.width > textbox.w {
                draw_text_ex(&line, textbox.x, y, text_params);
                line.clear();
                measure_line.clear();
                y += text_params.font_size as f32;
            } else {
                line += word;
            }
            measure_line += word;
            measure_line += " ";
        }
    }
}
