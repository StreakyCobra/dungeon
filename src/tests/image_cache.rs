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

    let spec = container::engine::build_image_command(&build.tag, build.no_cache, &build.context);
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(
        command,
        "podman build -f images/Containerfile.archlinux -t localhost/dungeon ."
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

    let spec = container::engine::build_image_command(&build.tag, build.no_cache, &build.context);
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(
        command,
        "podman build -f images/Containerfile.archlinux -t localhost/dungeon-dev --no-cache ./images"
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

    let spec = container::engine::build_cache_reset_command(config::Engine::Podman);
    let command = format!("{} {}", spec.program, spec.args.join(" "));

    assert_eq!(command, "podman volume rm -f dungeon-cache");
}
