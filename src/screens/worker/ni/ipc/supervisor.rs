//! The screen thread's synchronous adapter owns a bounded-lifetime runtime.

use tokio::runtime::Runtime;

use super::*;

pub(in crate::screens::worker) struct Supervisor {
    session: Option<Session>,
    runtime: Option<Runtime>,
}

impl Supervisor {
    #[cfg(windows)]
    pub(in crate::screens::worker) fn open(
        identity: Identity,
        cancellation: Cancellation,
    ) -> anyhow::Result<Self> {
        let helper = std::env::current_exe()?
            .parent()
            .context("bridge executable directory")?
            .join("s4-screen-supervisor.exe");
        let mut command = Command::new(helper);
        // Unmodified Encdr treats this serial as a preference and may fall back.
        command.arg("--serial").arg(identity.serial());
        Self::spawn(command, cancellation, Limits::default())
    }

    pub(super) fn spawn(
        command: Command,
        cancellation: Cancellation,
        limits: Limits,
    ) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        // Establish the runtime drop guard before any child/setup can fail.
        let mut supervisor = Self {
            session: None,
            runtime: Some(runtime),
        };
        supervisor.session = Some(
            supervisor
                .runtime
                .as_ref()
                .context("screen runtime closed")?
                .block_on(Session::connect(command, cancellation, limits))?,
        );
        Ok(supervisor)
    }

    fn parts(&mut self) -> anyhow::Result<(&Runtime, &mut Session)> {
        Ok((
            self.runtime.as_ref().context("screen runtime closed")?,
            self.session.as_mut().context("screen session closed")?,
        ))
    }
}

impl Transport for Supervisor {
    fn publish(&mut self, frames: &[Frame; 2]) -> anyhow::Result<()> {
        let (runtime, session) = self.parts()?;
        runtime.block_on(session.publish(frames))
    }
    fn poll_control(&mut self) -> anyhow::Result<()> {
        let (runtime, session) = self.parts()?;
        runtime.block_on(session.poll_control())
    }
    fn close(&mut self) -> anyhow::Result<()> {
        let (runtime, session) = self.parts()?;
        runtime.block_on(session.close())
    }
}

impl Drop for Supervisor {
    fn drop(&mut self) {
        if self.session.as_ref().is_some_and(|session| !session.closed)
            && let Err(error) = self.close()
        {
            tracing::warn!(%error, "SCREEN_RELEASE_UNCONFIRMED");
        }
        // Windows Tokio child stdio uses blocking-pool operations. Even if an
        // OS termination/reap fails, runtime teardown must not wait forever.
        self.session = None;
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_timeout(Duration::from_millis(100));
        }
    }
}
