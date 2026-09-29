//! Grok Build device sign-in on a worker with an isolated home.
//! Only auth.json travels to the vault; logs, sessions and local configuration
//! stay out of it. The worker removes the temporary login home after vaulting.
use anyhow::{bail, Context, Result};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    process::{Child, ChildStderr, Command},
};

const TO_CODE: Duration = Duration::from_secs(20);
const EXPIRES: Duration = Duration::from_secs(15 * 60);

pub struct Pending {
    pub user_code: String,
    pub verification_url: String,
}

pub struct Waiting {
    child: Child,
    lines: Lines<BufReader<ChildStderr>>,
    home: PathBuf,
}

pub async fn start(state: &Path, home: &Path) -> Result<(Pending, Waiting)> {
    tokio::fs::create_dir_all(home).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(home, std::fs::Permissions::from_mode(0o700)).await?;
    }
    let binary = crate::runtime::grok_binary(state).await?;
    let mut child = Command::new(binary)
        .args(["--no-auto-update", "login", "--device-auth"])
        .env("GROK_HOME", home)
        .env_remove("XAI_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .context("starting Grok Build device sign-in")?;
    let stderr = child
        .stderr
        .take()
        .context("Grok Build login has no stderr")?;
    let mut lines = BufReader::new(stderr).lines();
    let pending = match tokio::time::timeout(TO_CODE, read_code(&mut lines)).await {
        Ok(found) => found?,
        Err(_) => {
            let _ = child.start_kill();
            bail!("Grok Build did not print a device code within 20 seconds");
        }
    };
    Ok((
        pending,
        Waiting {
            child,
            lines,
            home: home.to_path_buf(),
        },
    ))
}

async fn read_code<R: tokio::io::AsyncBufRead + Unpin>(lines: &mut Lines<R>) -> Result<Pending> {
    while let Some(line) = lines.next_line().await? {
        let Some(at) = line.find("https://accounts.x.ai/oauth2/device?") else {
            continue;
        };
        let url: String = line[at..]
            .chars()
            .take_while(|c| !c.is_whitespace())
            .collect();
        let Some(code) = url.split("user_code=").nth(1) else {
            continue;
        };
        let code = code.split('&').next().unwrap_or(code);
        if !code.is_empty() {
            return Ok(Pending {
                user_code: code.into(),
                verification_url: url,
            });
        }
    }
    bail!("Grok Build login ended before showing a device code")
}

impl Waiting {
    pub async fn finish(mut self) -> Result<Vec<u8>> {
        let outcome = tokio::time::timeout(EXPIRES, async {
            // Drain stderr so the child's pipe cannot block its exit. Never
            // log it: future CLI versions may print account information.
            while self.lines.next_line().await?.is_some() {}
            Ok::<_, anyhow::Error>(self.child.wait().await?)
        })
        .await;
        let _ = self.child.start_kill();
        match outcome {
            Err(_) => bail!("Grok Build sign-in expired before approval"),
            Ok(Err(e)) => Err(e),
            Ok(Ok(status)) if !status.success() => bail!("Grok Build sign-in was not approved"),
            Ok(Ok(_)) => tokio::fs::read(self.home.join("auth.json"))
                .await
                .context("Grok Build said it signed in but wrote no auth.json"),
        }
    }

    pub async fn cancel(mut self) -> Result<()> {
        let _ = self.child.start_kill();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn code_is_taken_from_device_url() {
        let input = b"To sign in, open this URL in your browser:\nhttps://accounts.x.ai/oauth2/device?user_code=ABCD-EFGH\n";
        let mut lines = BufReader::new(&input[..]).lines();
        let pending = read_code(&mut lines).await.unwrap();
        assert_eq!(pending.user_code, "ABCD-EFGH");
    }

    /// Run manually with a Firetower-installed pinned CLI to detect a change
    /// in the stream or shape of the real device-code response.
    #[tokio::test]
    #[ignore = "requires a Firetower-installed Grok Build CLI and xAI network access"]
    async fn real_cli_prints_a_device_code_on_stderr() {
        let root = std::env::var(ft_core::WORKER_ROOT_ENV)
            .expect("set FIRETOWER_WORKER_ROOT to a worker with Grok Build installed");
        let home = tempfile::tempdir().unwrap();
        let (pending, waiting) = start(Path::new(&root), home.path()).await.unwrap();
        assert!(pending
            .verification_url
            .starts_with("https://accounts.x.ai/oauth2/device?"));
        assert!(!pending.user_code.is_empty());
        waiting.cancel().await.unwrap();
    }
}
