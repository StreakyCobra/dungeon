use crate::{cli, config, container};

#[test]
fn image_build_defaults_to_podman_and_default_tag() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec![
        "image".to_string(),
        "build".to_string(),
        "archlinux".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, defaults, file_cfg, env_cfg).expect("parse args");

    let build = match parsed.action {
        cli::Action::ImageBuild(build) => build,
        _ => panic!("expected image build action"),
    };

    let spec = container::engine::build_image_command(
        build.engine,
        build.flavor.containerfile_path(),
        &build.tag,
        build.no_cache,
        &build.context,
    );
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(
        command,
        "podman build -f images/Containerfile.archlinux -t localhost/dungeon ."
    );
}

#[test]
fn image_build_supports_docker_no_cache_and_custom_context() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec![
        "image".to_string(),
        "build".to_string(),
        "ubuntu".to_string(),
        "--engine".to_string(),
        "docker".to_string(),
        "--tag".to_string(),
        "localhost/dungeon-ubuntu".to_string(),
        "--no-cache".to_string(),
        "--context".to_string(),
        "./images".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, defaults, file_cfg, env_cfg).expect("parse args");

    let build = match parsed.action {
        cli::Action::ImageBuild(build) => build,
        _ => panic!("expected image build action"),
    };

    let spec = container::engine::build_image_command(
        build.engine,
        build.flavor.containerfile_path(),
        &build.tag,
        build.no_cache,
        &build.context,
    );
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(
        command,
        "docker build -f images/Containerfile.ubuntu -t localhost/dungeon-ubuntu --no-cache ./images"
    );
}

#[test]
fn cache_reset_uses_selected_engine() {
    let defaults = config::Config::default();
    let file_cfg = config::Config::default();
    let env_cfg = config::Config::default();
    let args = vec![
        "cache".to_string(),
        "reset".to_string(),
        "--engine".to_string(),
        "docker".to_string(),
    ];

    let parsed =
        cli::parse_args_with_sources(args, defaults, file_cfg, env_cfg).expect("parse args");

    let cache = match parsed.action {
        cli::Action::CacheReset(cache) => cache,
        _ => panic!("expected cache reset action"),
    };

    let spec = container::engine::build_cache_reset_command(cache.engine);
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(command, "docker volume rm -f dungeon-cache");
}
