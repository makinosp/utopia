use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn starts_ephemeral_postgres_container() {
    let image = Postgres::default().with_tag("17-alpine");
    let container = image.start().await.expect("start postgres container");

    let host = container.get_host().await.expect("host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("postgres port");

    assert!(!host.is_empty());
    assert!(port > 0);
}
