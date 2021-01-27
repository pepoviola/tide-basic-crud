use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::Pool;
use tera::Tera;
use tide::http::cookies::SameSite;
use tide::prelude::*;
use tide::{Error, Server};
use tide_tera::prelude::*;
use uuid::Uuid;

mod controllers;
mod handlers;

use controllers::auth;
use controllers::dino;
use controllers::views;

// OAuth deps and const
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

static AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
static TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v3/token";

#[derive(Clone, Debug)]
pub struct State {
    db_pool: PgPool,
    tera: Tera,
    oauth_google_client: BasicClient,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dino {
    id: Uuid,
    name: String,
    weight: i32,
    diet: String,
    user_id: Option<String>,
}

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct Dinos {
//     dinos: Vec<Dino>,
// }

#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();

    tide::log::start();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let port = std::env::var("APP_PORT").unwrap_or("8080".to_string());
    let db_pool = make_db_pool(&db_url).await;

    let app = server(db_pool).await;
    let mut listener = app
        .bind(format!("0.0.0.0:{}", port))
        .await
        .expect("can't bind the port");

    for info in listener.info().iter() {
        println!("Server listening on {}", info);
    }
    listener.accept().await.unwrap();
}

pub async fn make_db_pool(db_url: &str) -> PgPool {
    Pool::connect(db_url).await.unwrap()
}

fn make_oauth_google_client() -> tide::Result<BasicClient> {
    let client = BasicClient::new(
        ClientId::new(
            std::env::var("OAUTH_GOOGLE_CLIENT_ID")
                .expect("missing env var OAUTH_GOOGLE_CLIENT_ID"),
        ),
        Some(ClientSecret::new(
            std::env::var("OAUTH_GOOGLE_CLIENT_SECRET")
                .expect("missing env var OAUTH_GOOGLE_CLIENT_SECRET"),
        )),
        AuthUrl::new(AUTH_URL.to_string())?,
        Some(TokenUrl::new(TOKEN_URL.to_string())?),
    )
    .set_redirect_url(RedirectUrl::new(
        std::env::var("OAUTH_GOOGLE_REDIRECT_URL")
            .expect("missing env var OAUTH_GOOGLE_REDIRECT_URL"),
    )?);

    Ok(client)
}

async fn server(db_pool: PgPool) -> Server<State> {
    let mut tera = Tera::new("templates/**/*").expect("Error parsing templates directory");
    tera.autoescape_on(vec!["html"]);

    let oauth_google_client = make_oauth_google_client().unwrap();

    let state = State {
        db_pool,
        tera,
        oauth_google_client,
    };

    let mut app = tide::with_state(state);

    app.with(
        tide::sessions::SessionMiddleware::new(
            tide::sessions::MemoryStore::new(),
            std::env::var("TIDE_SECRET")
                .expect("Please provide a TIDE_SECRET value of at least 32 bytes")
                .as_bytes(),
        )
        .with_same_site_policy(SameSite::Lax),
    );

    // views
    app.at("/").get(views::index);
    app.at("/dinos/new").get(views::new);
    app.at("/dinos/:id/edit").get(views::edit);

    // auth
    app.at("/auth/google")
        .get(auth::auth_google)
        .at("/authorized")
        .get(auth::auth_google_authorized);

    app.at("/logout").get(auth::logout);

    // api
    app.at("/dinos").get(dino::list).post(dino::create);

    app.at("/dinos/:id")
        .get(dino::get)
        .put(dino::update)
        .delete(dino::delete);

    app.at("/public")
        .serve_dir("./public/")
        .expect("Invalid static file directory");

    app
}

// #[async_std::test]
// async fn index_page() -> tide::Result<()> {
//     use tide::http::{Method, Request as httpRequest, Response, Url};

//     // let dinos_store = Default::default();
//     let db_pool = make_db_pool().await;
//     let app = server(db_pool).await;
//     let url = Url::parse("https://example.com").unwrap();
//     let req = httpRequest::new(Method::Get, url);
//     let mut res: Response = app.respond(req).await?;
//     assert_eq!("ok", res.body_string().await?);
//     Ok(())
// }

// #[async_std::test]
// async fn list_dinos() -> tide::Result<()> {
//     dotenv::dotenv().ok();

//     let db_url = std::env::var("DATABASE_URL").unwrap();
//     let db_pool = make_db_pool(db_url).await;
//     let app = server(db_pool).await;

//     let res = surf::Client::with_http_client(app)
//         .get("https://example.com/dinos")
//         .await?;

//     assert_eq!(200, res.status());
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use sqlx::query;

    lazy_static! {
        static ref DB_URL: String =
            std::env::var("DATABASE_URL").expect("missing env var DATABASE_URL");
    }

    async fn clear_dinos() -> Result<(), Box<dyn std::error::Error>> {
        let db_pool = make_db_pool(&DB_URL).await;

        sqlx::query("DELETE FROM dinos").execute(&db_pool).await?;
        Ok(())
    }

    #[test]
    fn clear() {
        dotenv::dotenv().ok();
        async_std::task::block_on(async {
            clear_dinos().await.unwrap();
            ()
        })
    }

    #[async_std::test]
    async fn list_dinos() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        let db_pool = make_db_pool(&DB_URL).await;
        let app = server(db_pool).await;

        let res = surf::Client::with_http_client(app)
            .get("https://example.com/dinos")
            .await?;

        assert_eq!(200, res.status());
        Ok(())
    }

    #[async_std::test]
    async fn create_dino() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        use assert_json_diff::assert_json_eq;

        let dino = Dino {
            id: Uuid::new_v4(),
            name: String::from("test"),
            weight: 50,
            diet: String::from("carnivorous"),
            user_id: None,
        };

        let db_pool = make_db_pool(&DB_URL).await;
        let app = server(db_pool).await;

        let mut res = surf::Client::with_http_client(app)
            .post("https://example.com/dinos")
            .body(serde_json::to_string(&dino)?)
            .await?;

        assert_eq!(201, res.status());

        let d: Dino = res.body_json().await?;
        assert_json_eq!(dino, d);
        Ok(())
    }

    #[async_std::test]
    async fn create_dino_with_existing_key() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        let dino = Dino {
            id: Uuid::new_v4(),
            name: String::from("test_get"),
            weight: 500,
            diet: String::from("carnivorous"),
            user_id: None,
        };

        let db_pool = make_db_pool(&DB_URL).await;

        // create the dino
        query!(
            r#"
            INSERT INTO dinos (id, name, weight, diet) VALUES
            ($1, $2, $3, $4) returning id, name, weight, diet
            "#,
            dino.id,
            dino.name,
            dino.weight,
            dino.diet
        )
        .fetch_one(&db_pool)
        .await?;

        // start the server
        let app = server(db_pool).await;

        let res = surf::Client::with_http_client(app)
            .post("https://example.com/dinos")
            .body(serde_json::to_string(&dino)?)
            .await?;

        assert_eq!(409, res.status());

        Ok(())
    }

    #[async_std::test]
    async fn get_dino() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        use assert_json_diff::assert_json_eq;

        let dino = Dino {
            id: Uuid::new_v4(),
            name: String::from("test_get"),
            weight: 500,
            diet: String::from("carnivorous"),
            user_id: None,
        };

        let db_pool = make_db_pool(&DB_URL).await;

        // create the dino for get
        query!(
            r#"
            INSERT INTO dinos (id, name, weight, diet) VALUES
            ($1, $2, $3, $4) returning id, name, weight, diet
            "#,
            dino.id,
            dino.name,
            dino.weight,
            dino.diet
        )
        .fetch_one(&db_pool)
        .await?;

        // start the server
        let app = server(db_pool).await;

        let mut res = surf::Client::with_http_client(app)
            .get(format!("https://example.com/dinos/{}", &dino.id))
            .await?;

        assert_eq!(200, res.status());

        let d: Dino = res.body_json().await?;
        assert_json_eq!(dino, d);

        Ok(())
    }

    #[async_std::test]
    async fn get_dino_non_existing_key() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        // start the server
        let db_pool = make_db_pool(&DB_URL).await;
        let app = server(db_pool).await;

        let res = surf::Client::with_http_client(app)
            .get(format!("https://example.com/dinos/{}", &Uuid::new_v4()))
            .await?;

        assert_eq!(404, res.status());

        Ok(())
    }

    #[async_std::test]
    async fn update_dino() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        use assert_json_diff::assert_json_eq;

        let mut dino = Dino {
            id: Uuid::new_v4(),
            name: String::from("test_update"),
            weight: 500,
            diet: String::from("carnivorous"),
            user_id: None,
        };

        let db_pool = make_db_pool(&DB_URL).await;

        // create the dino for update
        query!(
            r#"
            INSERT INTO dinos (id, name, weight, diet) VALUES
            ($1, $2, $3, $4) returning id, name, weight, diet
            "#,
            dino.id,
            dino.name,
            dino.weight,
            dino.diet
        )
        .fetch_one(&db_pool)
        .await?;

        // change the dino
        dino.name = String::from("updated from test");

        // start the server
        let app = server(db_pool).await;

        let mut res = surf::Client::with_http_client(app)
            .put(format!("https://example.com/dinos/{}", &dino.id))
            .body(serde_json::to_string(&dino)?)
            .await?;

        assert_eq!(200, res.status());

        let d: Dino = res.body_json().await?;
        assert_json_eq!(dino, d);

        Ok(())
    }

    #[async_std::test]
    async fn updatet_dino_non_existing_key() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        let dino = Dino {
            id: Uuid::new_v4(),
            name: String::from("test_update"),
            weight: 500,
            diet: String::from("carnivorous"),
            user_id: None,
        };

        // start the server
        let db_pool = make_db_pool(&DB_URL).await;
        let app = server(db_pool).await;

        let res = surf::Client::with_http_client(app)
            .put(format!("https://example.com/dinos/{}", &dino.id))
            .body(serde_json::to_string(&dino)?)
            .await?;

        assert_eq!(404, res.status());

        Ok(())
    }

    #[async_std::test]
    async fn updatet_dino_create_by_another_user_should_reject_with_401() -> tide::Result<()> {
        dotenv::dotenv().ok();

        let mut dino = Dino {
            id: Uuid::new_v4(),
            name: String::from("test_update"),
            weight: 500,
            diet: String::from("carnivorous"),
            user_id: Some(String::from("123")),
        };

        let db_pool = make_db_pool(&DB_URL).await;

        // create the dino for update
        query!(
            r#"
            INSERT INTO dinos (id, name, weight, diet, user_id) VALUES
            ($1, $2, $3, $4, $5) returning id
            "#,
            dino.id,
            dino.name,
            dino.weight,
            dino.diet,
            dino.user_id
        )
        .fetch_one(&db_pool)
        .await?;

        // change the dino
        dino.name = String::from("updated from test");

        // start the server
        let app = server(db_pool).await;

        let res = surf::Client::with_http_client(app)
            .put(format!("https://example.com/dinos/{}", &dino.id))
            .body(serde_json::to_string(&dino)?)
            .await?;

        assert_eq!(401, res.status());

        Ok(())
    }

    #[async_std::test]
    async fn delete_dino() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        let dino = Dino {
            id: Uuid::new_v4(),
            name: String::from("test_delete"),
            weight: 500,
            diet: String::from("carnivorous"),
            user_id: None,
        };

        let db_pool = make_db_pool(&DB_URL).await;

        // create the dino for delete
        query!(
            r#"
            INSERT INTO dinos (id, name, weight, diet) VALUES
            ($1, $2, $3, $4) returning id, name, weight, diet
            "#,
            dino.id,
            dino.name,
            dino.weight,
            dino.diet
        )
        .fetch_one(&db_pool)
        .await?;

        // start the server
        let app = server(db_pool).await;

        let res = surf::Client::with_http_client(app)
            .delete(format!("https://example.com/dinos/{}", &dino.id))
            .await?;

        assert_eq!(204, res.status());

        Ok(())
    }

    #[async_std::test]
    async fn delete_dino_non_existing_key() -> tide::Result<()> {
        dotenv::dotenv().ok();
        // clear_dinos()
        //     .await
        //     .expect("Failed to clear the dinos table");

        // start the server
        let db_pool = make_db_pool(&DB_URL).await;
        let app = server(db_pool).await;

        let res = surf::Client::with_http_client(app)
            .delete(format!("https://example.com/dinos/{}", &Uuid::new_v4()))
            .await?;

        assert_eq!(404, res.status());

        Ok(())
    }

    #[async_std::test]
    async fn delete_dino_create_by_another_user_should_reject_with_401() -> tide::Result<()> {
        dotenv::dotenv().ok();

        let dino = Dino {
            id: Uuid::new_v4(),
            name: String::from("test_delete"),
            weight: 500,
            diet: String::from("carnivorous"),
            user_id: Some(String::from("123")),
        };

        let db_pool = make_db_pool(&DB_URL).await;

        // create the dino for delete
        query!(
            r#"
            INSERT INTO dinos (id, name, weight, diet, user_id) VALUES
            ($1, $2, $3, $4, $5) returning id
            "#,
            dino.id,
            dino.name,
            dino.weight,
            dino.diet,
            dino.user_id
        )
        .fetch_one(&db_pool)
        .await?;

        // start the server
        let app = server(db_pool).await;

        let res = surf::Client::with_http_client(app)
            .delete(format!("https://example.com/dinos/{}", &dino.id))
            .await?;

        assert_eq!(401, res.status());

        Ok(())
    }
}
