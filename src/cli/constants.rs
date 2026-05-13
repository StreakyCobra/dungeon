pub(crate) const SUBCOMMAND_RUN: &str = "run";
pub(crate) const SUBCOMMAND_IMAGE: &str = "image";
pub(crate) const SUBCOMMAND_IMAGE_BUILD: &str = "build";
pub(crate) const SUBCOMMAND_CACHE: &str = "cache";
pub(crate) const SUBCOMMAND_CACHE_RESET: &str = "reset";

pub(crate) const FLAG_HELP: &str = "help";
pub(crate) const FLAG_VERSION: &str = "version";
pub(crate) const FLAG_DEBUG: &str = "debug";
pub(crate) const FLAG_PERSIST: &str = "persist";
pub(crate) const FLAG_PERSISTED: &str = "persisted";
pub(crate) const FLAG_DISCARD: &str = "discard";
pub(crate) const FLAG_COMMAND: &str = "command";
pub(crate) const FLAG_IMAGE: &str = "image";
pub(crate) const FLAG_PORT: &str = "port";
pub(crate) const FLAG_CACHE: &str = "cache";
pub(crate) const FLAG_MOUNT: &str = "mount";
pub(crate) const FLAG_ENV: &str = "env";
pub(crate) const FLAG_ENV_FILE: &str = "env-file";
pub(crate) const FLAG_RUN_ARG: &str = "run-arg";
pub(crate) const FLAG_SKIP_CWD: &str = "skip-cwd";
pub(crate) const FLAG_IPV6: &str = "ipv6";
pub(crate) const FLAG_NO_IPV6: &str = "no-ipv6";
pub(crate) const FLAG_ALLOW_DNS: &str = "allow-dns";
pub(crate) const FLAG_DENY_DNS: &str = "deny-dns";
pub(crate) const FLAG_ALLOW_DOMAIN: &str = "allow-domain";
pub(crate) const FLAG_ALLOW_HOST: &str = "allow-host";
pub(crate) const FLAG_TAG: &str = "tag";
pub(crate) const FLAG_NO_CACHE: &str = "no-cache";
pub(crate) const FLAG_CONTEXT: &str = "context";
pub(crate) const ARG_PATHS: &str = "paths";

pub(crate) const RESERVED_GROUP_NAMES: &[&str] = &[
    "general",
    FLAG_HELP,
    FLAG_VERSION,
    FLAG_DEBUG,
    FLAG_PERSIST,
    FLAG_PERSISTED,
    FLAG_DISCARD,
    FLAG_COMMAND,
    FLAG_IMAGE,
    FLAG_PORT,
    FLAG_CACHE,
    FLAG_MOUNT,
    FLAG_ENV,
    FLAG_ENV_FILE,
    FLAG_RUN_ARG,
    FLAG_SKIP_CWD,
    FLAG_IPV6,
    FLAG_NO_IPV6,
    FLAG_ALLOW_DNS,
    FLAG_DENY_DNS,
    FLAG_ALLOW_DOMAIN,
    FLAG_ALLOW_HOST,
    FLAG_TAG,
    FLAG_NO_CACHE,
    FLAG_CONTEXT,
    ARG_PATHS,
    SUBCOMMAND_RUN,
    SUBCOMMAND_IMAGE,
    SUBCOMMAND_IMAGE_BUILD,
    SUBCOMMAND_CACHE,
    SUBCOMMAND_CACHE_RESET,
];
