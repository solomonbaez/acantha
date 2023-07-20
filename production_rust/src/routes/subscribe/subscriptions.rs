use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;
use crate::utils::see_other;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(value.email)?;
        let name = SubscriberName::parse(value.name)?;
        Ok(NewSubscriber { email, name })
    }
}

// TODO: Create separate API integration
// hard http responses are useful here, just not for my website
// #[tracing::instrument(
//     name = "Adding a new subscriber",
//     skip(form, pool, email_client),
//     fields(
//         subscriber_email = %form.email,
//         subscriber_name = %form.name
//     )
// )]
// pub async fn subscribe(
//     form: web::Form<FormData>,
//     pool: web::Data<PgPool>,
//     email_client: web::Data<EmailClient>,
//     base_url: web::Data<ApplicationBaseUrl>,
// ) -> Result<HttpResponse, SubscribeError> {
//     let new_subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;
//     let mut transaction = pool
//         .begin()
//         .await
//         .context("Failed to aquire a Postgres connection from the pool.")?;
//     let subscriber_id = insert_subscriber(&new_subscriber, &mut transaction)
//         .await
//         .context("Failed to insert a new subscriber in the database.")?;

//     let subscription_token = generate_subscription_token();

//     store_token(&mut transaction, subscriber_id, &subscription_token)
//         .await
//         .context("Failed to store the confirmation token for a new subscriber.")?;
//     transaction
//         .commit()
//         .await
//         .context("Failed to commit SQLX transaction to store a new subscriber.")?;
//     send_confirmation_email(
//         &email_client,
//         new_subscriber,
//         &base_url.0,
//         &subscription_token,
//     )
//     .await
//     .context("Failed to send a confirmation email.")?;

//     Ok(HttpResponse::Ok().finish())
// }

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let response = see_other("/subscriptions");

    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(e) => {
            FlashMessage::error(e).send();
            return Ok(response);
        }
    };

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to aquire a Postgres connection from the pool.")?;
    match insert_subscriber(&new_subscriber, &mut transaction).await {
        Ok(subscriber_id) => {
            let subscription_token = generate_subscription_token();
            store_token(&mut transaction, subscriber_id, &subscription_token)
                .await
                .context("Failed to store the confirmation token for a new subscriber.")?;
            transaction
                .commit()
                .await
                .context("Failed to commit SQLX transaction to store a new subscriber.")?;
            if send_confirmation_email(
                &email_client,
                new_subscriber,
                &base_url.0,
                &subscription_token,
            )
            .await
            .is_ok()
            {
                FlashMessage::info("You are now subscribed!").send();
            } else {
                FlashMessage::error(
                    "Failed to send confirmation email, please check input fields.",
                )
                .send();
            }
        }
        Err(_) => {
            FlashMessage::error("It seems you're already subscribed!").send();
        }
    }

    Ok(response)
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );

    let html_body = format!(
        "Welcome to our newsletter!<br />
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let text_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(&new_subscriber.email, "Welcome!", &html_body, &text_body)
        .await
}

#[tracing::instrument(name = "Saving to database", skip(subscriber, transaction))]
pub async fn insert_subscriber(
    subscriber: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        &subscriber_id,
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await?;

    Ok(subscriber_id)
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Saving subscription token to database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await?;

    Ok(())
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)] // impl Display, source
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub struct StoreTokenError(sqlx::Error);
impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token."
        )
    }
}
impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
