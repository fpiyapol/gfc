use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    gfc::start().await
}
