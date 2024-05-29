use eyre::{Context, Report};
use myutil::log::{LogMode, init_log};

fn main() {
    init_log(LogMode::General, tracing::Level::TRACE);
    display1();
}

fn display1() {
    tracing::trace!("[trace]-1 log info");
    tracing::debug!("[debug]-2 log info");
    tracing::info!("[info]-3 log info");
    tracing::warn!("[warn]-4 log info");
    tracing::error!("[error]-5 log info");
    tracing::error!("[error]-5.1 {}", my_err());
    tracing::error!("[error]-5.2 {:?}", my_err());
    tracing::error!("[error]-5.3 {:#}", my_err());
    tracing::error!("[error]-5.4 {:#?}", my_err());
}

fn my_err() -> Report {
    || -> eyre::Result<()> {
        Err(eyre::eyre!("error: my error 1."))
    }().context("my error 2.").context("my error 3.").unwrap_err()
}
