use std::path::PathBuf;

use anyhow::Context;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Resolve the log directory per the XDG base directory spec
fn log_dir() -> anyhow::Result<PathBuf> {
    let base = match std::env::var_os("XDG_STATE_HOME") {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => {
            let home =
                std::env::var_os("HOME").context("neither XDG_STATE_HOME nor HOME is set")?;
            PathBuf::from(home).join(".local/state")
        }
    };
    Ok(base.join(env!("CARGO_PKG_NAME")))
}

/// Initialise tracing.
///
/// The returned guard flushes the non-blocking writer's worker thread on drop, so the
/// caller must hold it for the process lifetime.
pub fn setup_tracing() -> anyhow::Result<WorkerGuard> {
    let dir = log_dir()?;
    let (non_blocking, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(&dir, "app.log"));

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(concat!(env!("CARGO_PKG_NAME"), "=info,warn")));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .try_init()?;

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_state_home_wins() {
        temp_env::with_vars(
            [
                ("XDG_STATE_HOME", Some("/xdg/state")),
                ("HOME", Some("/home/someone")),
            ],
            || {
                assert_eq!(
                    log_dir().unwrap(),
                    PathBuf::from("/xdg/state").join(env!("CARGO_PKG_NAME"))
                );
            },
        );
    }

    #[test]
    fn empty_xdg_state_home_falls_back_to_home() {
        temp_env::with_vars(
            [
                ("XDG_STATE_HOME", Some("")),
                ("HOME", Some("/home/someone")),
            ],
            || {
                assert_eq!(
                    log_dir().unwrap(),
                    PathBuf::from("/home/someone/.local/state").join(env!("CARGO_PKG_NAME"))
                );
            },
        );
    }

    #[test]
    fn unset_xdg_state_home_falls_back_to_home() {
        temp_env::with_vars(
            [("XDG_STATE_HOME", None), ("HOME", Some("/home/someone"))],
            || {
                assert_eq!(
                    log_dir().unwrap(),
                    PathBuf::from("/home/someone/.local/state").join(env!("CARGO_PKG_NAME"))
                );
            },
        );
    }

    #[test]
    fn both_unset_is_an_error() {
        temp_env::with_vars([("XDG_STATE_HOME", None::<&str>), ("HOME", None)], || {
            assert!(log_dir().is_err());
        });
    }
}
