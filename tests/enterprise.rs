use neo4j_testcontainers::{prelude::*, runners::AsyncRunner as _, Neo4j, RunnableImage};
use neo4rs::Graph;

#[tokio::test]
async fn runs_enterprise() {
    let neo4j = RunnableImage::from(Neo4j::default().with_password("Picard123"));
    let neo4j = match neo4j.with_enterprise_edition() {
        Ok(n) => n,
        Err(e) => {
            eprintln!(
                "Skipping enterprise test, no license acceptance file found: {}",
                e
            );
            return;
        }
    };

    let container = neo4j.start().await;

    let image = container.image();

    let uri = image.bolt_uri_ipv4();
    assert!(uri.starts_with("bolt://"));

    let uri = image.http_uri_ipv4();
    assert!(uri.starts_with("http://"));

    let uri = image.bolt_uri_ipv4();
    let auth_user = image.user().expect("default user");
    let auth_pass = image.password().expect("set password");

    let graph = Graph::new(uri, auth_user, auth_pass).await.unwrap();
    let mut result = graph
        .execute(neo4rs::query(
            "CALL dbms.components() YIELD edition RETURN edition",
        ))
        .await
        .unwrap();
    let row = result.next().await.unwrap().unwrap();
    let value: String = row.get("edition").unwrap();
    assert_eq!(value, "enterprise");
}
