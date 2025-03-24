use std::any::{Any, TypeId};
use std::collections::HashMap;

// The entity ID type
pub type Entity = usize;

// Trait for component vectors
pub trait ComponentVec {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn push_none(&mut self);
    fn remove(&mut self, idx: usize);
}

// Concrete component vector implementation
impl<T: 'static> ComponentVec for Vec<Option<T>> {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
    
    fn push_none(&mut self) {
        self.push(None);
    }
    
    fn remove(&mut self, idx: usize) {
        self[idx] = None;
    }
}

// The ECS World
pub struct World {
    entities: Vec<Entity>,
    next_entity: Entity,
    components: HashMap<TypeId, Box<dyn ComponentVec>>,
    free_entities: Vec<Entity>,
}

impl World {
    // Create a new empty world
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_entity: 0,
            components: HashMap::new(),
            free_entities: Vec::new(),
        }
    }
    
    // Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let entity = if let Some(entity) = self.free_entities.pop() {
            self.entities[entity] = entity;
            entity
        } else {
            let entity = self.next_entity;
            self.next_entity += 1;
            self.entities.push(entity);
            
            // Add None for this entity in all component vecs
            for component_vec in self.components.values_mut() {
                component_vec.push_none();
            }
            
            entity
        };
        
        entity
    }
    
    // Delete an entity
    pub fn delete_entity(&mut self, entity: Entity) {
        if entity >= self.entities.len() {
            return;
        }
        
        // Remove all components
        for component_vec in self.components.values_mut() {
            component_vec.remove(entity);
        }
        
        // Mark entity as free
        self.entities[entity] = 0;
        self.free_entities.push(entity);
    }
    
    // Remove a specific component from an entity
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        if entity >= self.entities.len() {
            return;
        }
        
        let type_id = TypeId::of::<T>();
        if let Some(component_vec) = self.components.get_mut(&type_id) {
            component_vec.remove(entity);
        }
    }
    
    // Register a component type
    fn register_component<T: 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.components.contains_key(&type_id) {
            let mut component_vec: Vec<Option<T>> = Vec::new();
            
            // Add None for each existing entity
            for _ in 0..self.entities.len() {
                component_vec.push(None);
            }
            
            self.components.insert(type_id, Box::new(component_vec));
        }
    }
    
    // Add a component to an entity
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        if entity >= self.entities.len() {
            return;
        }
        
        // Register component type if needed
        self.register_component::<T>();
        
        // Add component to entity
        let type_id = TypeId::of::<T>();
        let component_vec = self.components.get_mut(&type_id).unwrap();
        let component_vec = component_vec
            .as_any_mut()
            .downcast_mut::<Vec<Option<T>>>()
            .unwrap();
        
        component_vec[entity] = Some(component);
    }
    
    // Get component for an entity
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        if entity >= self.entities.len() {
            return None;
        }
        
        let type_id = TypeId::of::<T>();
        let component_vec = self.components.get(&type_id)?;
        let component_vec = component_vec
            .as_any()
            .downcast_ref::<Vec<Option<T>>>()
            .unwrap();
        
        if let Some(component) = &component_vec[entity] {
            Some(component)
        } else {
            None
        }
    }
    
    // Get mutable component for an entity
    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        if entity >= self.entities.len() {
            return None;
        }
        
        let type_id = TypeId::of::<T>();
        let component_vec = self.components.get_mut(&type_id)?;
        let component_vec = component_vec
            .as_any_mut()
            .downcast_mut::<Vec<Option<T>>>()
            .unwrap();
        
        if let Some(component) = &mut component_vec[entity] {
            Some(component)
        } else {
            None
        }
    }
    
    // Query for all entities with a specific component
    pub fn query<T: 'static>(&self) -> Vec<(Entity, &T)> {
        let mut result = Vec::new();
        
        let type_id = TypeId::of::<T>();
        if let Some(component_vec) = self.components.get(&type_id) {
            let component_vec = component_vec
                .as_any()
                .downcast_ref::<Vec<Option<T>>>()
                .unwrap();
            
            for (entity, component) in component_vec.iter().enumerate() {
                if let Some(component) = component {
                    result.push((entity, component));
                }
            }
        }
        
        result
    }
    
    // Get a list of all entities
    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }
} 