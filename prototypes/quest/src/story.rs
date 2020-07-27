#[derive(Clone, Copy)]
pub enum StoryPhase {
		Menu,
//		Exposition(u8),
//		Tutorial(u8),
//		Dialogue(u8),
		PlayerMove,
		EnemyMove,
		Death,
}

pub struct StoryManager {
		current_phase: usize,
		new_queued: bool,
		phases_vec: Vec<StoryPhase>,
}

impl StoryManager {
		pub fn new() -> StoryManager {
				StoryManager {
						current_phase: 0,
						new_queued: false,
						phases_vec: vec![
								StoryPhase::Menu, /*StoryPhase::Dialogue(0),*/ StoryPhase::PlayerMove, StoryPhase::EnemyMove, StoryPhase::Death
						],
				}
		}

		pub fn get_current_phase(&self) -> StoryPhase {
				self.phases_vec[self.current_phase]
		}

		pub fn advance_phase(&mut self) {
				self.new_queued = true;
		}
		pub fn update(&mut self) {
				if self.new_queued {
						self.current_phase += 1;
						self.new_queued = false;
				}
		}
}
