use crate::{cli, container, error::AppError};

pub fn run() -> Result<(), AppError> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let parsed = cli::parse_args(args)?;

    let resolved = crate::config::resolve_with_defaults(&parsed)?;

    if parsed.show_version {
        println!("{}", cli::build_version());
        return Ok(());
    }

    if parsed.reset_cache {
        container::podman::reset_cache_volume()?;
    }

    match resolved.persist_mode {
        container::persist::PersistMode::Discard => {
            container::persist::discard_container(&resolved.container_name)?;
        }
        container::persist::PersistMode::Reuse => {
            container::persist::ensure_container_session(&resolved.container_name)?;
        }
        container::persist::PersistMode::Create => {
            let spec = container::podman::build_podman_command(
                &resolved.settings,
                &resolved.paths,
                true,
                Some(&resolved.container_name),
                resolved.skip_cwd,
            )?;
            container::persist::run_persisted_session(&resolved.container_name, spec)?;
        }
        container::persist::PersistMode::None => {
            let spec = container::podman::build_podman_command(
                &resolved.settings,
                &resolved.paths,
                false,
                None,
                resolved.skip_cwd,
            )?;
            container::podman::run_podman_command(spec)?;
        }
    }

    Ok(())
}
