use lfml::html;

#[tokio::main]
async fn main() {
    let app = axum::Router::new().route(
        "/",
        axum::routing::get(|| async {
            html! { "Hello, World!"}
        }),
    );

    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
