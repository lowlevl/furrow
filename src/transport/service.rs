use std::{
    ffi::OsStr,
    path::Path,
    process::{ExitStatus, Stdio},
};

use color_eyre::eyre;
use parse_display::FromStr;
use russh::{server::Msg, Channel, ChannelMsg};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use crate::repository;

#[derive(Debug, FromStr)]
#[display("{} '{repository}'", style = "kebab-case")]
pub enum Service {
    GitUploadPack { repository: repository::Id },
    GitReceivePack { repository: repository::Id },
}

impl Service {
    pub fn repository(&self) -> &repository::Id {
        match self {
            Service::GitUploadPack { repository } => repository,
            Service::GitReceivePack { repository } => repository,
        }
    }

    pub async fn exec(
        &self,
        envs: impl IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>,
        storage: &Path,
        channel: &mut Channel<Msg>,
    ) -> eyre::Result<ExitStatus> {
        let mut child = match self {
            Self::GitUploadPack { repository } => Command::new("git-upload-pack")
                .envs(envs)
                .arg("--strict")
                .arg("--timeout=1")
                .arg(repository.to_path(storage))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()?,
            Self::GitReceivePack { repository } => Command::new("git-receive-pack")
                .arg(repository.to_path(storage))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()?,
        };

        let (mut stdin, mut stdout) = (
            child.stdin.take(),
            child
                .stdout
                .take()
                .expect("Unable to take service's `stdout` handle"),
        );

        let mut done = false;
        loop {
            let mut buf = [0u8; 4096 * 8];

            tokio::select! {
                read = stdout.read(&mut buf) => {
                    let n = read?;

                    if n > 0 {
                        channel.data(&buf[..n]).await?;
                    } else {
                        done = true;
                    }
                }
                msg = channel.wait() => {
                    tracing::trace!("Received channel message: {msg:?}");

                    match msg {
                        Some(ChannelMsg::Data { data }) => {
                            if let Some(stdin) = &mut stdin {
                                stdin.write_all(&data).await?;
                            }
                        }
                        None | Some(ChannelMsg::Eof) => {
                            if let Some(mut stdin) = stdin.take() {
                                stdin.flush().await?;

                                drop(stdin);
                            }
                        },
                        _ => ()
                    }
                }
                // Exit the loop once everything is flushed
                true = async { stdin.is_none() && done } => {
                    break;
                }
            }
        }

        Ok(child.wait().await?)
    }
}
