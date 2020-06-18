//pretty basic implementation of vectors because what even are crates
pub struct Vector {
		pub x: f32,
		pub y: f32,
}
#[allow(dead_code)]
impl Vector {
		pub fn new(x: f32, y: f32) -> Vector {
				Vector { x, y }
		}
		
		pub fn add(&self, other: &Vector) -> Vector {
				Vector::new(self.x + other.x, self.y + other.y)
		}

		pub fn subtract(&self, other: &Vector) -> Vector {
				Vector::new(self.x - other.x, self.y - other.y)
		}

		pub fn scale_by(&self, number: f32) -> Vector {
				Vector::new(self.x * number, self.y * number)
		}

		pub fn length(&self) -> f32 {
				self.x.hypot(self.y)
		}

		pub fn normalize(&self) -> Vector {
				self.scale_by(1_f32 / self.length())
		}
		
		pub fn equal_to(&self, other: &Vector) -> bool {
				self.x == other.x && self.y == other.y
		}
		
		pub fn is_opposite(&self, other: &Vector) -> bool {
				let sum = self.add(other);
				sum.equal_to(&Vector::new(0_f32, 0_f32))
		}

		pub fn dot_product(&self, other: &Vector) -> f32 {
				self.x * other.x + self.y * other.y
		}
		pub fn rotate(&self, other: &Vector) -> Vector {
				Vector::new(
						(self.x * other.x / (other.x.powi(2) + other.y.powi(2)).sqrt()) -
								(self.y * other.y / (other.x.powi(2) + other.y.powi(2)).sqrt())
								,
						(self.x * other.y / (other.x.powi(2) + other.y.powi(2)).sqrt()) +
								(self.y * other.x / (other.x.powi(2) + other.y.powi(2)).sqrt())
				)
		}
}
