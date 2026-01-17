use crate::{cli, container, error::AppError};

pub fn run() -> Result<(), AppError> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let input = cli::parse_args(args)?;

    if input.show_version {
        println!("{}", cli::build_version());
        return Ok(());
    }

    if input.persist_mode == container::persist::PersistMode::None
        && !input.reset_cache
        && input.settings == crate::config::Settings::default()
        && input.paths.is_empty()
    {
        return Ok(());
    }

    if input.reset_cache {
        container::podman::reset_cache_volume()?;
    }

    match input.persist_mode {
        container::persist::PersistMode::Discard => {
            container::persist::discard_container(&input.container_name)?;
        }
        container::persist::PersistMode::Reuse => {
            container::persist::ensure_container_session(&input.container_name)?;
        }
        container::persist::PersistMode::Create => {
            let spec = container::podman::build_podman_command(
                &input.settings,
                &input.paths,
                true,
                Some(&input.container_name),
            )?;
            container::persist::run_persisted_session(&input.container_name, spec)?;
        }
        container::persist::PersistMode::None => {
            let spec = container::podman::build_podman_command(
                &input.settings,
                &input.paths,
                false,
                None,
            )?;
            container::podman::run_podman_command(spec)?;
        }
    }

    Ok(())
}
