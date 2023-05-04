use axum::response::Html;

mod post;
pub use post::login;

pub async fn login_form() -> Html<&'static str> {
    Html(include_str!("login.html"))
}
