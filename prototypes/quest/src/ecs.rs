#[derive(Debug, PartialEq, Eq, Hash)]
enum ComponentType {
		MapPosition,
		RenderablePosition,
}

pub trait ComponentTrait {
		fn add_entity_to_component(&mut self, // how do i do this part, need the actual component here);
		fn update_component(&mut self);
		fn react_to_input(&mut self);
		fn remove_entity_from_component(&mut self);
}

struct ComponentVecWrapper<T> where T: ComponentTrait {
		component_type: ComponentType,
		component_vec: T,
}

struct Manager {
		components: Vec<ComponentVecWrapper>,
}

impl Manager {
		fn add_entity(/*something here*/) {
				for component in self.components {
						if input_hashmap.conatains_key(component.component_type) {
								component.add_entity_to_component(Some(input_hashmap[component_type]));
						} else {
								component.add_entity_to_component(None);
						}
				}
		}
}
