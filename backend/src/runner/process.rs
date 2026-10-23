// Запуск программы игрока как отдельного процесса с ограничением по времени.
//
// ВАЖНО: пока код выполняется прямо на машине с сервером, без изоляции.
// Годится только для разработки. Перед тем как сервер увидят другие люди, этот файл
// заменит Docker-песочница — check.rs при этом не изменится.
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::error::AppError;

pub enum ProcessOutput {
    Finished { stdout: String, stderr: String },
    TimedOut,
}

// command — программа и аргументы, например ["node", "solution.js"]
pub async fn run_process(
    command: &[&str],
    dir: &Path,
    input: String,
    time_limit: Duration,
) -> Result<ProcessOutput, AppError> {
    let mut child = Command::new(command[0])
        .args(&command[1..])
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true) // при выходе по таймауту процесс убивается вместе с child
        .spawn()
        .map_err(|err| AppError::Internal(format!("Не удалось запустить {}: {err}", command[0])))?;

    if let Some(mut stdin) = child.stdin.take() {
        // Ошибку записи игнорируем: если код игрока упал до чтения stdin, это покажет stderr
        stdin.write_all(input.as_bytes()).await.ok();
    }

    match tokio::time::timeout(time_limit, child.wait_with_output()).await {
        Ok(output) => {
            let output = output.map_err(|err| AppError::Internal(err.to_string()))?;
            Ok(ProcessOutput::Finished {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
        Err(_) => Ok(ProcessOutput::TimedOut),
    }
}
