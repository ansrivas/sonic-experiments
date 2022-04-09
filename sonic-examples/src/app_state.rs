use sonic_channel::*;
use sqlx::PgPool;

/// AppState to share with the server instance.
/// All your dependencies can be injected here.
pub struct AppState {
    pub pgpool: PgPool,
    pub ingest: IngestChannel,
    pub search: SearchChannel,
    pub control: ControlChannel,
}
