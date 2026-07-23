use netchronicle_db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub auth_required: bool,
}

impl AppState {
    pub fn new(db: DbPool, auth_required: bool) -> Self {
        Self { db, auth_required }
    }
}
