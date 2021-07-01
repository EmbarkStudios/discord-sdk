use crate::relations::{events::RelationshipEvent, Relationship};
use parking_lot::RwLock;

#[derive(Debug)]
pub struct Relationships {
    pub relationships: RwLock<Vec<std::sync::Arc<Relationship>>>,
}

impl Relationships {
    pub fn new(relations: Vec<Relationship>) -> Self {
        Self {
            relationships: RwLock::new(relations.into_iter().map(std::sync::Arc::new).collect()),
        }
    }

    pub fn on_event(&self, re: RelationshipEvent) {
        match re {
            RelationshipEvent::Update(rel) => {
                let mut rels = self.relationships.write();
                match rels.iter().position(|r| r.user.id == rel.user.id) {
                    Some(i) => {
                        rels[i] = rel;
                    }
                    None => rels.push(rel),
                }
            }
        }
    }
}
