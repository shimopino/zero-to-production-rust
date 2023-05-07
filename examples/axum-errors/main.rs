use axum::{
    // async_trait,
    // body::Body,
    // extract::{rejection::JsonRejection, FromRequest, MatchedPath},
    // http::{Request, StatusCode},
    // response::IntoResponse,
    routing::get,
    Router,
};
// use serde_json::Value;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello World" }));
    // .route("/custom-rejection", post(custom_rejection_handler));

    // let listner = tokio::net::TcpListener::bind("127.0.0.1:8090")
    //     .await
    //     .unwrap();
    // axum::serve(listner, app).await.unwrap()

    let addr = SocketAddr::from(([127, 0, 0, 1], 8090));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

// Extractorをカスタマイズして、拡張できるようにする
// struct Json<T>(pub T);

// async fn custom_rejection_handler(Json(value): Json<Value>) -> impl IntoResponse {
//     Json(dbg!(value))
// }

// #[async_trait]
// impl<S, T> FromRequest<S> for Json<T>
// where
//     axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
//     S: Send + Sync,
// {
//     type Rejection = (StatusCode, axum::Json<Value>);

//     async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
//         let (mut parts, body) = req.into_parts();

//         // We can use other extractors to provide better rejection messages.
//         // For example, here we are using `axum::extract::MatchedPath` to
//         // provide a better error message.
//         //
//         // Have to run that first since `Json` extraction consumes the request.
//         let path = parts
//             .extract::<MatchedPath>()
//             .await
//             .map(|path| path.as_str().to_owned())
//             .ok();

//         let req = Request::from_parts(parts, body);

//         match axum::Json::<T>::from_request(req, state).await {
//             Ok(value) => Ok(Self(value.0)),
//             // convert the error from `axum::Json` into whatever we want
//             Err(rejection) => {
//                 let payload = serde_json::json!({
//                     "message": rejection.body_text(),
//                     "origin": "custom_extractor",
//                     "path": path,
//                 });

//                 Err((rejection.status(), axum::Json(payload)))
//             }
//         }
//     }
// }
