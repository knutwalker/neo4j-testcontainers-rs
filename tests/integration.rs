use neo4j_testcontainers::{prelude::*, runners::AsyncRunner as _, Neo4j};
use neo4rs::Graph;

#[tokio::test]
async fn it_works() {
    let container = Neo4j::default().start().await;

    let uri = container.image().bolt_uri_ipv4();
    assert!(uri.starts_with("bolt://"));

    let uri = container.image().http_uri_ipv4();
    assert!(uri.starts_with("http://"));

    let uri = container.image().bolt_uri_ipv4();
    let auth_user = container.image().user().expect("default user");
    let auth_pass = container.image().password().expect("default password");

    let graph = Graph::new(uri, auth_user, auth_pass).await.unwrap();
    let mut result = graph.execute(neo4rs::query("RETURN 1")).await.unwrap();
    let row = result.next().await.unwrap().unwrap();
    let value: i64 = row.get("1").unwrap();
    assert_eq!(1, value);
}
