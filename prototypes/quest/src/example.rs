use super::vector::Vector;
use math::round;

pub fn make_example_1() -> Vec<((f32, f32), (Vector, f32, u32))> {
		let mut elements: Vec<((f32, f32), (Vector, f32, u32))>
				= Vec::new();
		make_board(&mut elements);
		elements
}
fn make_board(current_elements: &mut Vec<((f32, f32), (Vector, f32, u32))>) {
		for i in 0..64 {
				let x_pos: f32 = 6.25 + ((i % 8) as f32 * 12.5);
				let y_pos: f32 = 6.25 + round::floor((i / 8) as f64 , 0) as f32 * 12.5;

				let facing: Vector = Vector::new(1_f32, 0_f32);
				let size: f32 = 0.25;
				let texture: u32 = (i % 2) + (round::floor((i / 8) as f64, 0) as u32 % 2);

				current_elements.push(
						((x_pos, y_pos), (facing, size, texture))
				);
		}
}
