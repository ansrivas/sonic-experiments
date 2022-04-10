use crate::channel::Channel;
use sqlx::PgPool;

/// AppState to share with the server instance.
/// All your dependencies can be injected here.
pub struct AppState {
    pub pgpool: PgPool,
    pub channel: Channel,
}
