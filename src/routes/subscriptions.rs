use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    log::info!(
        "adding '{}' '{}' as a new subscriber",
        form.email,
        form.name
    );
    log::info!("saving new subscriber details in the DB");
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
    // We use `get_ref` to get an immutable reference to the `PgConnection`
    // wrapped by `web::Data`.
    .execute(connection.as_ref())
    .await
    {
        Ok(_) => {
            log::info!("new subscriber added in the db");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
