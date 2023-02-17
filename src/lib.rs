use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
