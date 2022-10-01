use super::AppEvent;

pub fn spawn_ticker(tick_rate: std::time::Duration, tx: tokio::sync::mpsc::Sender<AppEvent>) {
    let mut ticker = tokio::time::interval(tick_rate);

    tokio::spawn(async move {
        loop {
            ticker.tick().await;
            tx.send(AppEvent::Tick).await.unwrap();
        }
    });
}
