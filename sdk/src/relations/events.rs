use super::*;

#[derive(Debug, Clone)]
pub enum RelationshipEvent {
    Update(Relationship),
}
