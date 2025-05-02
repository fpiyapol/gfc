use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    gfc::init().await
}
