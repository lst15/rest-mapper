use anyhow::{Context, Result};
use std::{io::ErrorKind, path::PathBuf, process::ExitStatus, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStderr, ChildStdout, Command},
};

use crate::collector::protocol;

#[derive(Debug, Clone)]
pub struct CollectorConfig {
    pub session_id: String,
    pub output_path: PathBuf,
    pub url: String,
    pub script_path: PathBuf,
    pub node_bin: String,
    pub browser: String,
    pub headless: bool,
}

pub struct CollectorProcess {
    child: Child,
    stdout_task: tokio::task::JoinHandle<()>,
    stderr_task: tokio::task::JoinHandle<()>,
}

impl CollectorProcess {
    pub async fn spawn(config: CollectorConfig) -> Result<Self> {
        let mut cmd = Command::new(&config.node_bin);
        cmd.arg(config.script_path)
            .arg("--session-id")
            .arg(config.session_id)
            .arg("--output")
            .arg(config.output_path)
            .arg("--url")
            .arg(config.url)
            .arg("--browser")
            .arg(config.browser);

        if config.headless {
            cmd.arg("--headless");
        }

        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .with_context(|| "falha ao iniciar collector Node/Playwright")?;

        let stdout = child
            .stdout
            .take()
            .context("collector sem stdout disponível")?;
        let stderr = child
            .stderr
            .take()
            .context("collector sem stderr disponível")?;

        let stdout_task = tokio::spawn(read_stdout(stdout));
        let stderr_task = tokio::spawn(read_stderr(stderr));

        Ok(Self {
            child,
            stdout_task,
            stderr_task,
        })
    }

    pub async fn request_shutdown(&mut self) -> Result<()> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            match stdin.write_all(b"shutdown\n").await {
                Ok(_) => {
                    let _ = stdin.flush().await;
                }
                Err(err) if err.kind() == ErrorKind::BrokenPipe => {}
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        Ok(self.child.try_wait()?)
    }

    pub async fn wait(&mut self) -> Result<ExitStatus> {
        Ok(self.child.wait().await?)
    }

    pub async fn finalize(self, exit_status: ExitStatus, log_timeout: Duration) -> Result<()> {
        let _ = tokio::time::timeout(log_timeout, self.stdout_task).await;
        let _ = tokio::time::timeout(log_timeout, self.stderr_task).await;

        if !exit_status.success() {
            eprintln!(
                "[flowtrace] collector terminou com status não-zero: {:?}",
                exit_status.code()
            );
        }

        Ok(())
    }
}

async fn read_stdout(stdout: ChildStdout) {
    let mut lines = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(protocol::CollectorStdoutMessage::Status { message, detail }) =
            protocol::parse_status_line(&line)
        {
            if let Some(detail) = detail {
                println!("[collector] {message} {detail}");
            } else {
                println!("[collector] {message}");
            }
        } else {
            println!("[collector] {line}");
        }
    }
}

async fn read_stderr(stderr: ChildStderr) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        eprintln!("[collector:stderr] {line}");
    }
}
