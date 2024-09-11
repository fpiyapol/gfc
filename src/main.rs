#[tokio::main]
async fn main() {
    let _ = gfc::init_docker().await;
}
