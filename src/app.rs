use crate::{cli, container, error::AppError};

pub fn run() -> Result<(), AppError> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let parsed = cli::parse_args(args)?;

    if parsed.show_help {
        return Ok(());
    }

    if parsed.show_version {
        println!("{}", cli::build_version());
        return Ok(());
    }

    match &parsed.action {
        cli::Action::None => Ok(()),
        cli::Action::ImageBuild(build) => {
            let spec = container::engine::build_image_command(
                build.engine,
                build.flavor.containerfile_path(),
                &build.tag,
                build.no_cache,
                &build.context,
            );
            container::engine::run_container_command(spec)
        }
        cli::Action::CacheReset(cache) => container::engine::reset_cache_volume(cache.engine),
        cli::Action::Run => run_container_session(parsed),
    }
}

fn run_container_session(parsed: cli::ParsedCLI) -> Result<(), AppError> {
    let resolved = crate::config::resolve_with_defaults(&parsed)?;

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
