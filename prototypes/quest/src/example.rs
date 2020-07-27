use super::vector::Vector;
use math::round;

pub fn make_example_1() -> Vec<((f32, f32), (Vector, f32, (u32, Option<u32>)))> {
		let mut elements: Vec<((f32, f32), (Vector, f32, (u32, Option<u32>)))>
				= Vec::new();
		make_board(&mut elements);
		make_pieces(&mut elements);
		make_enemy(&mut elements);
		make_fog(&mut elements);
		make_menu(&mut elements);
		elements
}
fn make_fog(current_elements: &mut Vec<((f32, f32), (Vector, f32, (u32, Option<u32>)))>) {
		current_elements.push(
				(
						(69_f32, 81_f32),
						(Vector::new(1_f32, 0_f32), 3.5, (13_u32, None))
				)
		);
}
fn make_board(current_elements: &mut Vec<((f32, f32), (Vector, f32, (u32, Option<u32>)))>) {
		for i in 0..64 {
				let x_pos: f32 = 6.25 + ((i % 8) as f32 * 12.5);
				let y_pos: f32 = 6.25 + round::floor((i / 8) as f64 , 0) as f32 * 12.5;

				let facing: Vector = Vector::new(1_f32, 0_f32);
				let size: f32 = 0.25;
				let base_texture: u32 = (i + (round::floor((i / 8) as f64, 0) as u32 % 2)) % 2;
				let highlight_texture: Option<u32> = Some(base_texture + 2); 
				let texture: (u32, Option<u32>) = (base_texture, highlight_texture);

				current_elements.push(
						((x_pos, y_pos), (facing, size, texture))
				);
		}
}

fn make_pieces(current_elements: &mut Vec<((f32, f32), (Vector, f32, (u32, Option<u32>)))>) {
		/*current_elements.push(
				((81.25, 81_f32), (Vector::new(1_f32, 0_f32), 0.25, (4_u32, None)))
		);*/
		for i in 0..8 {
				current_elements.push(
						(
								((6.25 + 12.5 * i as f32), 81_f32),
								(Vector::new(1_f32, 0_f32), 0.25, (4_u32, None))
						)
				);
		}
		let pieces_tex: Vec<u32> = vec![5, 6, 7, 9, 8, 7, 6, 5];
		for i in 0..8 {
				current_elements.push(
						(
								((6.25 + 12.5 * i as f32), 93.5),
								(Vector::new(1_f32, 0_f32), 0.25, (pieces_tex[i], None))
						)
				);
		}
}

fn make_menu(current_elements: &mut Vec<((f32, f32), (Vector, f32, (u32, Option<u32>)))>) {
		current_elements.push(
				((50_f32, 50_f32), (Vector::new(1_f32, 0_f32), 2.0, (11_u32, None)))
		);
}

fn make_enemy(current_elements: &mut Vec<((f32, f32), (Vector, f32, (u32, Option<u32>)))>) {
		current_elements.push(
				((56.25, 43.57), (Vector::new(1_f32, 0_f32), 0.25, (10_u32, None)))
		);
}
