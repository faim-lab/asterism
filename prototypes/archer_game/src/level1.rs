use super::vector::Vector;
use math::round;

pub fn make_level_1() -> Vec<((f32, f32), (Vector, f32, u32))> {
    let mut elements: Vec<((f32, f32), (Vector, f32, u32))> = Vec::new();
    make_background(&mut elements);
    make_player(&mut elements);
    elements
}
fn make_background(current_elements: &mut Vec<((f32, f32), (Vector, f32, u32))>) {
    for i in 0..256 {
        let x_pos: f32 = 6.25 + ((i % 16) as f32 * 12.5);
        let y_pos: f32 = 6.25 + round::floor((i / 16) as f64, 0) as f32 * 12.5;

        let facing: Vector = Vector::new(1_f32, 0_f32);
        let size: f32 = 0.25;
        let texture: u32 = 0_u32;

        current_elements.push(((x_pos, y_pos), (facing, size, texture)));
    }
}

fn make_player(current_elements: &mut Vec<((f32, f32), (Vector, f32, u32))>) {
    current_elements.push(((100_f32, 100_f32), (Vector::new(1_f32, 0_f32), 0.25, 1_u32)));
}

//pub fn make_arrow(current_elements: &mut Vec<((f32, f32), (Vector, f32, u32))>) {
//        current_elements.push(
//                ((100_f32, 100_f32), (Vector::new(1_f32, 0_f32), 0.25, 2_u32))
//        );
//
//}
