use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    // actix-webは type-map 機能を使って同じ型に紐づく値を注入する
    // この機能を使って依存性の注入を実現できる
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // もしも何かしら調査が必要になった場合に備えて、情報はログに出力するようにする
    let request_id = Uuid::new_v4();
    // Spansを作成してログを出力する
    // Spanでは key-value な値としてログを登録することで構造化を行う
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id, // % をつけることで Display 実装を使うようにしている
        form.email,
        form.name
    );
    //
    let _request_span_guard = request_span.enter();

    // enterは実行しない。 .instrument で登録する
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    // await を呼び出す前に tracing を紐づける
    // これで Future が完了するまでに Executor によって何回ポーリングされたのかわかる
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
