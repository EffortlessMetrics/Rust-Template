use tracing_subscriber::{fmt, EnvFilter};

pub fn init() {
    let _ = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init();
}
