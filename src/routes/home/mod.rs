use axum::response::Html;

pub async fn home() -> Html<&'static str> {
    // 以下のHTTPヘッダーが自動的に付与される
    // Content-Type: text/html
    Html(include_str!("home.html"))
}
