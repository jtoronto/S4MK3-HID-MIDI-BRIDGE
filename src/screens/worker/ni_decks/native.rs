use super::*;
use crate::screens::ni_service::{NiService, Raster};

impl Transport for NiService {
    async fn publish(&mut self, frames: &[Frame; 2]) -> anyhow::Result<()> {
        let [left, right] = frames.each_ref();
        self.publish_pair([Raster::new(&left.pixels)?, Raster::new(&right.pixels)?])
            .await?;
        Ok(())
    }

    async fn apply(
        &mut self,
        desired: [DeckOutput; 2],
        allow_play: impl Fn(usize) -> bool + Send + Sync,
    ) -> anyhow::Result<[DeckOutput; 2]> {
        Ok(self.set_deck_output_guarded(desired, allow_play).await?)
    }

    async fn poll(&mut self) -> anyhow::Result<()> {
        self.poll_control().await?;
        Ok(())
    }

    async fn quiesce(&mut self) -> anyhow::Result<()> {
        self.quiesce_decks().await?;
        Ok(())
    }
}
