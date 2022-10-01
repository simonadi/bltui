use futures::StreamExt;

use super::AppEvent;

pub async fn spawn_adapter_watcher(
    mut events: std::pin::Pin<
        Box<dyn futures::Stream<Item = btleplug::api::CentralEvent> + std::marker::Send>,
    >,
    tx: tokio::sync::mpsc::Sender<AppEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            tx.send(AppEvent::Adapter(event)).await.unwrap();
        }
    });
}
