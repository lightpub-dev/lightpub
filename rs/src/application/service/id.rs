use uuid::Uuid;

pub struct IDGenerationService {}

impl IDGenerationService {
    pub fn generate_id() -> Uuid {
        Uuid::now_v7()
    }
}
