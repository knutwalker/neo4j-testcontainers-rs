use neo4j_testcontainers::Neo4j;
use neo4rs::Graph;
use testcontainers::clients::Cli;

#[tokio::test]
async fn it_works() {
    let docker = Cli::default();
    let container = docker.run(Neo4j::default());

    let uri = container.image().bolt_uri_ipv4();
    assert!(uri.starts_with("bolt://"));

    let uri = container.image().http_uri_ipv4();
    assert!(uri.starts_with("http://"));

    let uri = container.image().bolt_uri_ipv4();
    let uri = uri.trim_start_matches("bolt://");
    let auth_user = container.image().user();
    let auth_pass = container.image().pass();

    let graph = Graph::new(uri, auth_user, auth_pass).await.unwrap();
    let mut result = graph.execute(neo4rs::query("RETURN 1")).await.unwrap();
    let row = result.next().await.unwrap().unwrap();
    let value: i64 = row.get("1").unwrap();
    assert_eq!(1, value);
}
