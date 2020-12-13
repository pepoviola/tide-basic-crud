use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::{query, query_as, PgPool};
use tera::Tera;
use tide::prelude::*;
use tide::{Body, Error, Request, Response, Server};
use tide_tera::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct State {
    db_pool: PgPool,
    tera: Tera,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Dino {
    id: Uuid,
    name: String,
    weight: i32,
    diet: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Dinos {
    dinos: Vec<Dino>,
}

struct RestEntity {
    base_path: String,
}

impl RestEntity {
    async fn create(mut req: Request<State>) -> tide::Result {
        let dino: Dino = req.body_json().await?;
        let db_pool = req.state().db_pool.clone();
        let row: Dino = match query_as!(
            Dino,
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
        .await
        {
            Ok(r) => r,
            Err(e) => {
                // TODO: we may want to cast the error here.
                let err = Error::new(409, e);
                return Err(err);
            }
        };

        let mut res = Response::new(201);
        res.set_body(Body::from_json(&row)?);
        Ok(res)
    }

    async fn list(req: tide::Request<State>) -> tide::Result {
        let db_pool = req.state().db_pool.clone();
        let rows = query_as!(
            Dino,
            r#"
            SELECT id, name, weight, diet from dinos
            "#
        )
        .fetch_all(&db_pool)
        .await?;
        let mut res = Response::new(200);
        res.set_body(Body::from_json(&rows)?);
        Ok(res)
    }

    async fn get(req: tide::Request<State>) -> tide::Result {
        let db_pool = req.state().db_pool.clone();
        let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
        let row = query_as!(
            Dino,
            r#"
            SELECT  id, name, weight, diet from dinos
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&db_pool)
        .await?;

        let res = match row {
            None => Response::new(404),
            Some(row) => {
                let mut r = Response::new(200);
                r.set_body(Body::from_json(&row)?);
                r
            }
        };

        Ok(res)
    }

    async fn update(mut req: tide::Request<State>) -> tide::Result {
        let dino: Dino = req.body_json().await?;
        let db_pool = req.state().db_pool.clone();
        let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
        let row = query_as!(
            Dino,
            r#"
            UPDATE dinos SET name = $2, weight = $3, diet = $4
            WHERE id = $1
            returning id, name, weight, diet
            "#,
            id,
            dino.name,
            dino.weight,
            dino.diet
        )
        .fetch_optional(&db_pool)
        .await?;

        let res = match row {
            None => Response::new(404),
            Some(row) => {
                let mut r = Response::new(200);
                r.set_body(Body::from_json(&row)?);
                r
            }
        };

        Ok(res)
    }

    async fn delete(req: tide::Request<State>) -> tide::Result {
        let db_pool = req.state().db_pool.clone();
        let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
        let row = query!(
            r#"
            delete from dinos
            WHERE id = $1
            returning id
            "#,
            id
        )
        .fetch_optional(&db_pool)
        .await?;

        let res = match row {
            None => Response::new(404),
            Some(_) => Response::new(204),
        };

        Ok(res)
    }
}
#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();

    tide::log::start();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db_pool = make_db_pool(&db_url).await;

    let app = server(db_pool).await;
    let mut listener = app
        .bind("127.0.0.1:8080")
        .await
        .expect("can't bind the port");

    for info in listener.info().iter() {
        println!("Server listening on {}", info);
    }
    listener.accept().await.unwrap();
}

fn register_rest_entity(app: &mut Server<State>, entity: RestEntity) {
    app.at(&entity.base_path)
        .get(RestEntity::list)
        .post(RestEntity::create);

    app.at(&format!("{}/:id", entity.base_path))
        .get(RestEntity::get)
        .put(RestEntity::update)
        .delete(RestEntity::delete);
}

pub async fn make_db_pool(db_url: &str) -> PgPool {
    Pool::new(db_url).await.unwrap()
}

async fn server(db_pool: PgPool) -> Server<State> {
    let mut tera = Tera::new("templates/**/*").expect("Error parsing templates directory");
    tera.autoescape_on(vec!["html"]);

    let state = State { db_pool, tera };

    let mut app = tide::with_state(state);

    // index page
    app.at("/").get(|req: tide::Request<State>| async move {
        let tera = req.state().tera.clone();
        let db_pool = req.state().db_pool.clone();
        let rows = query_as!(
            Dino,
            r#"
            SELECT id, name, weight, diet from dinos
            "#
        )
        .fetch_all(&db_pool)
        .await?;
        tera.render_response(
            "index.html",
            &context! {
               "title" => String::from("Tide basic CRUD"),
               "dinos" => rows
            },
        )
    });

    // new dino
    app.at("/dinos/new")
        .get(|req: tide::Request<State>| async move {
            let tera = req.state().tera.clone();

            tera.render_response(
                "form.html",
                &context! {
                   "title" => String::from("Create new dino")
                },
            )
        });
    // edit dino
    app.at("/dinos/:id/edit")
        .get(|req: tide::Request<State>| async move {
            let tera = req.state().tera.clone();
            let db_pool = req.state().db_pool.clone();
            let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
            let row = query_as!(
                Dino,
                r#"
            SELECT  id, name, weight, diet from dinos
            WHERE id = $1
            "#,
                id
            )
            .fetch_optional(&db_pool)
            .await?;

            let res = match row {
                None => Response::new(404),
                Some(row) => {
                    let mut r = Response::new(200);
                    let b = tera.render_body(
                        "form.html",
                        &context! {
                            "title" => String::from("Edit dino"),
                            "dino" => row
                        },
                    )?;
                    r.set_body(b);
                    r
                }
            };

            Ok(res)
        });

    let dinos_endpoint = RestEntity {
        base_path: String::from("/dinos"),
    };

    register_rest_entity(&mut app, dinos_endpoint);

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
}
