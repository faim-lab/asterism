#[derive(Clone, Copy)]
pub enum StoryPhase {
		Menu,
//		Exposition(u8),
//		Tutorial(u8),
//		Dialogue(u8),
//		EnemyMove(u8),
		PlayerMove,
}

pub struct StoryManager {
		current_phase: usize,
		phases_vec: Vec<StoryPhase>,
}

impl StoryManager {
		pub fn new() -> StoryManager {
				StoryManager {
						current_phase: 0,
						phases_vec: vec![
								StoryPhase::Menu, StoryPhase::PlayerMove
						],
				}
		}

		pub fn get_current_phase(&self) -> StoryPhase {
				self.phases_vec[self.current_phase]
		}

		pub fn advance_phase(&mut self) {
				self.current_phase += 1;
		}
}
