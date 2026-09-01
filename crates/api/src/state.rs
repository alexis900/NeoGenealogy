use neogenealogy_storage::Storage;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
}

impl AppState {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }
}
