use bltui::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::init().await;

    app.run().await;

    Ok(())
}
