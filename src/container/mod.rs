pub mod engine;
pub mod persist;

use std::process::{Command, Stdio};

use crate::error::AppError;

pub(crate) fn run_attached_command(program: &str, args: &[String]) -> Result<(), AppError> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            Err(AppError::Subprocess(
                code,
                format!("{} exited with code {}", program, code),
            ))
        }
        Err(err) => Err(AppError::Io(err)),
    }
}
