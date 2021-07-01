use super::*;

#[derive(Debug, Clone)]
pub enum RelationshipEvent {
    Update(std::sync::Arc<Relationship>),
}
