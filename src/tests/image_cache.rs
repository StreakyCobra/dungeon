use crate::{cli, config, container};

#[test]
fn image_build_defaults_to_podman_and_default_tag() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["image".to_string(), "build".to_string()];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    let build = match parsed.action {
        cli::Action::ImageBuild(build) => build,
        _ => panic!("expected image build action"),
    };

    let spec = container::engine::build_image_command(
        &parsed.settings,
        &build.tag,
        build.no_cache,
        &build.context,
    );
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(
        command,
        "podman build -f images/Containerfile -t localhost/dungeon ."
    );
}

#[test]
fn image_build_supports_no_cache_and_custom_context() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec![
        "image".to_string(),
        "build".to_string(),
        "--tag".to_string(),
        "localhost/dungeon-dev".to_string(),
        "--no-cache".to_string(),
        "--context".to_string(),
        "./images".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    let build = match parsed.action {
        cli::Action::ImageBuild(build) => build,
        _ => panic!("expected image build action"),
    };

    let spec = container::engine::build_image_command(
        &parsed.settings,
        &build.tag,
        build.no_cache,
        &build.context,
    );
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(
        command,
        "podman build -f images/Containerfile -t localhost/dungeon-dev --no-cache ./images"
    );
}

#[test]
fn cache_reset_uses_podman() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec!["cache".to_string(), "reset".to_string()];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    let _cache = match parsed.action {
        cli::Action::CacheReset(cache) => cache,
        _ => panic!("expected cache reset action"),
    };

    let spec = container::engine::build_cache_reset_command(&parsed.settings);
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(command, "podman volume rm -f dungeon-cache");
}

#[test]
fn image_build_accepts_podman_args_from_config_and_cli() {
    let defaults = config::Config::default();
    let mut file_cfg = config::Config::default();
    file_cfg.settings.podman_args = Some(vec!["-c".to_string(), "agent-vm".to_string()]);
    let env_cfg = config::Config::default();
    let args = vec![
        "image".to_string(),
        "build".to_string(),
        "--podman-arg=--log-level=debug".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    let build = match parsed.action {
        cli::Action::ImageBuild(build) => build,
        _ => panic!("expected image build action"),
    };

    let settings = config::resolve_settings(
        config::Sources {
            defaults: defaults.settings.clone(),
            file: file_cfg.settings.clone(),
            env: env_cfg.settings.clone(),
            cli: parsed.settings.clone(),
        },
        &defaults.groups,
        &[],
    )
    .expect("resolve settings");
    let spec = container::engine::build_image_command(
        &settings,
        &build.tag,
        build.no_cache,
        &build.context,
    );
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(
        command,
        "podman -c agent-vm --log-level=debug build -f images/Containerfile -t localhost/dungeon ."
    );
}

#[test]
fn cache_reset_accepts_podman_args_from_config_and_cli() {
    let defaults = config::Config::default();
    let mut file_cfg = config::Config::default();
    file_cfg.settings.podman_args = Some(vec!["-c".to_string(), "agent-vm".to_string()]);
    let env_cfg = config::Config::default();
    let args = vec![
        "cache".to_string(),
        "reset".to_string(),
        "--podman-arg=--log-level=debug".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, &defaults, &file_cfg, &env_cfg).expect("parse args");

    let _cache = match parsed.action {
        cli::Action::CacheReset(cache) => cache,
        _ => panic!("expected cache reset action"),
    };

    let settings = config::resolve_settings(
        config::Sources {
            defaults: defaults.settings.clone(),
            file: file_cfg.settings.clone(),
            env: env_cfg.settings.clone(),
            cli: parsed.settings.clone(),
        },
        &defaults.groups,
        &[],
    )
    .expect("resolve settings");
    let spec = container::engine::build_cache_reset_command(&settings);
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(
        command,
        "podman -c agent-vm --log-level=debug volume rm -f dungeon-cache"
    );
}

#[test]
fn image_build_uses_podman_args_from_always_on_groups() {
    let defaults = config::Config::default();
    let mut file_cfg = config::Config::default();
    file_cfg.always_on_groups = Some(vec!["vm".to_string()]);
    file_cfg.groups.insert(
        "vm".to_string(),
        config::GroupConfig {
            settings: config::Settings {
                podman_args: Some(vec!["-c".to_string(), "agent-vm".to_string()]),
                ..config::Settings::default()
            },
            disabled: false,
        },
    );
    let sources = config::LoadedConfigSources {
        defaults,
        file: file_cfg,
        env: config::Config::default(),
    };
    let args = vec!["image".to_string(), "build".to_string()];

    let parsed = cli::parse_args_with_sources(
        args,
        &sources.defaults,
        &sources.file,
        &sources.env,
    )
    .expect("parse args");

    let build = match parsed.action {
        cli::Action::ImageBuild(build) => build,
        _ => panic!("expected image build action"),
    };

    let settings = config::resolve_global_settings(&parsed.settings, &sources).expect("resolve settings");
    let spec = container::engine::build_image_command(
        &settings,
        &build.tag,
        build.no_cache,
        &build.context,
    );

    assert_eq!(
        format!("{} {}", spec.program, spec.args.join(" ")),
        "podman -c agent-vm build -f images/Containerfile -t localhost/dungeon ."
    );
}

#[test]
fn cache_reset_uses_podman_args_from_always_on_groups() {
    let defaults = config::Config::default();
    let mut file_cfg = config::Config::default();
    file_cfg.always_on_groups = Some(vec!["vm".to_string()]);
    file_cfg.groups.insert(
        "vm".to_string(),
        config::GroupConfig {
            settings: config::Settings {
                podman_args: Some(vec!["-c".to_string(), "agent-vm".to_string()]),
                ..config::Settings::default()
            },
            disabled: false,
        },
    );
    let sources = config::LoadedConfigSources {
        defaults,
        file: file_cfg,
        env: config::Config::default(),
    };
    let args = vec!["cache".to_string(), "reset".to_string()];

    let parsed = cli::parse_args_with_sources(
        args,
        &sources.defaults,
        &sources.file,
        &sources.env,
    )
    .expect("parse args");

    let _cache = match parsed.action {
        cli::Action::CacheReset(cache) => cache,
        _ => panic!("expected cache reset action"),
    };

    let settings = config::resolve_global_settings(&parsed.settings, &sources).expect("resolve settings");
    let spec = container::engine::build_cache_reset_command(&settings);

    assert_eq!(
        format!("{} {}", spec.program, spec.args.join(" ")),
        "podman -c agent-vm volume rm -f dungeon-cache"
    );
}
