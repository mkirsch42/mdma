use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    Router,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use maud::html;
use sqlx::PgPool;
use tower_http::services::ServeDir;

mod admin;
mod auth;
mod components;
mod db;
mod discord;
mod donorbox;
mod err_responses;
mod icons;
mod send_email;
mod webconnex;

#[derive(Clone)]
struct AppState {
    db_pool: sqlx::PgPool,
    secret_store: HashMap<String, String>,
    google_oauth: oauth2::basic::BasicClient,
    http_client: reqwest::Client,
    discord_verifier: serenity::interactions_endpoint::Verifier,
    discord_http: Arc<serenity::http::Http>,
    discord_guild: serenity::model::id::GuildId,
}

async fn home(cookies: CookieJar) -> Response {
    match cookies.get("jwt") {
        None => components::layout(
            html! {
                a ."btn" href="/signin" {"Sign In"}
            },
            None,
        )
        .into_response(),
        Some(_) => Redirect::to("/admin").into_response(),
    }
}

#[tokio::main]
async fn main() {
    let secret_store = std::env::vars().collect::<HashMap<_, _>>();
    let db_pool = PgPool::connect(std::env::var("POSTGRES_DATABASE").as_ref().unwrap())
        .await
        .unwrap();
    sqlx::migrate!().run(&db_pool).await.unwrap();

    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let google_oauth = auth::oauth_client(
        secret_store.get("GOOGLE_OAUTH_CLIENT_ID").unwrap().clone(),
        secret_store
            .get("GOOGLE_OAUTH_CLIENT_SECRET")
            .unwrap()
            .clone(),
        secret_store.get("GOOGLE_OAUTH_REDIRECT").unwrap().clone(),
    );

    let discord_verifier = serenity::interactions_endpoint::Verifier::new(
        &secret_store.get("DISCORD_API_KEY").unwrap(),
    );

    let http_client = reqwest::Client::new();

    let discord_http = Arc::new(serenity::http::Http::new(
        &secret_store.get("DISCORD_BOT_TOKEN").unwrap(),
    ));
    discord_http.set_application_id(
        serenity::model::id::ApplicationId::from_str(
            &secret_store.get("DISCORD_APPLICATION_ID").unwrap(),
        )
        .unwrap(),
    );

    let discord_guild =
        serenity::model::id::GuildId::from_str(&secret_store.get("DISCORD_GUILD_ID").unwrap())
            .unwrap();

    let state = AppState {
        db_pool,
        secret_store,
        google_oauth,
        http_client,
        discord_verifier,
        discord_http,
        discord_guild,
    };

    discord::create_commands(&state).await;

    let router = Router::new()
        .route("/", get(home))
        .route("/signin", get(auth::signin_redirect))
        .route("/signout", get(auth::signout))
        .route("/callback-google", get(auth::oauth_callback))
        .route("/.discord/interaction", post(discord::handle_request))
        .with_state(state.clone())
        .nest("/admin", admin::router(state.clone()))
        .nest("/.webconnex", webconnex::router(state.clone()))
        .nest("/.donorbox", donorbox::router(state.clone()))
        .nest_service("/assets", ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
