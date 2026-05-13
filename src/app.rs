use crate::{cli, container, error::AppError};

pub fn run() -> Result<(), AppError> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let sources = crate::config::load_sources()?;
    let parsed =
        cli::parse_args_with_sources(args, &sources.defaults, &sources.file, &sources.env)?;

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
            let settings = resolve_global_settings(
                &parsed.settings,
                &sources.defaults,
                &sources.file,
                &sources.env,
            )?;
            let spec = container::engine::build_image_command(
                &settings,
                &build.tag,
                build.no_cache,
                &build.context,
            );
            container::engine::run_container_command(spec)
        }
        cli::Action::CacheReset(_) => {
            let settings = resolve_global_settings(
                &parsed.settings,
                &sources.defaults,
                &sources.file,
                &sources.env,
            )?;
            container::engine::reset_cache_volume(&settings)
        }
        cli::Action::Run => run_container_session(parsed, &sources),
    }
}

fn resolve_global_settings(
    cli_settings: &crate::config::Settings,
    defaults: &crate::config::Config,
    file: &crate::config::Config,
    env: &crate::config::Config,
) -> Result<crate::config::Settings, AppError> {
    crate::config::resolve_settings(
        crate::config::Sources {
            defaults: defaults.settings.clone(),
            file: file.settings.clone(),
            env: env.settings.clone(),
            cli: cli_settings.clone(),
        },
        &defaults.groups,
        &[],
    )
}

fn run_container_session(
    parsed: cli::ParsedCLI,
    sources: &crate::config::LoadedConfigSources,
) -> Result<(), AppError> {
    let resolved = crate::config::resolve(&parsed, sources)?;

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

    match resolved.persist_mode {
        container::persist::PersistMode::Discard => {
            container::persist::discard_container(&resolved.container_name, &resolved.settings)?;
        }
        container::persist::PersistMode::Reuse => {
            container::persist::ensure_container_session(
                &resolved.container_name,
                &resolved.settings,
            )?;
        }
        container::persist::PersistMode::Create => {
            let spec = container::engine::build_container_command(
                &resolved.settings,
                &resolved.paths,
                true,
                Some(&resolved.container_name),
                resolved.skip_cwd,
            )?;
            container::persist::run_persisted_session(
                &resolved.container_name,
                spec,
                &resolved.settings,
            )?;
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
