use crate::{cli, container, error::AppError};

pub fn run() -> Result<(), AppError> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let parsed = cli::parse_args(args)?;

    let resolved = crate::config::resolve_with_defaults(&parsed)?;

    if parsed.show_help {
        return Ok(());
    }

    if parsed.show_version {
        println!("{}", cli::build_version());
        return Ok(());
    }

    if parsed.debug {
        let spec = container::engine::build_container_command(
            &resolved.settings,
            &resolved.paths,
            false,
            None,
            resolved.skip_cwd,
        )?;
        println!("{} {}", spec.program, spec.args.join(" "));
        return Ok(());
    }

    if parsed.reset_cache {
        let engine = resolved.settings.engine.unwrap_or_default();
        container::engine::reset_cache_volume(engine)?;
    }

    let engine = resolved.settings.engine.unwrap_or_default();

    match resolved.persist_mode {
        container::persist::PersistMode::Discard => {
            container::persist::discard_container(&resolved.container_name, engine)?;
        }
        container::persist::PersistMode::Reuse => {
            container::persist::ensure_container_session(&resolved.container_name, engine)?;
        }
        container::persist::PersistMode::Create => {
            let spec = container::engine::build_container_command(
                &resolved.settings,
                &resolved.paths,
                true,
                Some(&resolved.container_name),
                resolved.skip_cwd,
            )?;
            container::persist::run_persisted_session(&resolved.container_name, spec, engine)?;
        }
        container::persist::PersistMode::None => {
            let spec = container::engine::build_container_command(
                &resolved.settings,
                &resolved.paths,
                false,
                None,
                resolved.skip_cwd,
            )?;
            container::engine::run_container_command(spec)?;
        }
    }

    Ok(())
}
