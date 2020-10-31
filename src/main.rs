use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::{query, query_as, PgPool};
use tide::{Body, Request, Response, Server};
use uuid::Uuid;

#[derive(Clone, Debug)]
struct State {
    // dinos: Arc<RwLock<HashMap<String, Dino>>>,
    db_pool: PgPool,
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
        let row = query_as!(
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
        .await?;

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
    let db_pool = make_db_pool().await;
    // let dinos_store = Default::default();
    let app = server(db_pool).await;

    app.listen("127.0.0.1:8080").await.unwrap();
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

pub async fn make_db_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    Pool::new(&db_url).await.unwrap()
}

async fn server(db_pool: PgPool) -> Server<State> {
    let state = State { db_pool };

    let mut app = tide::with_state(state);
    app.at("/").get(|_| async { Ok("ok") });

    let dinos_endpoint = RestEntity {
        base_path: String::from("/dinos"),
    };

    register_rest_entity(&mut app, dinos_endpoint);

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

#[async_std::test]
async fn list_dinos() -> tide::Result<()> {
    use tide::http::{Method, Request, Response, Url};

    let dino = Dino {
        id: Uuid::new_v4(),
        name: String::from("test"),
        weight: 50,
        diet: String::from("carnivorous"),
    };

    let db_pool = make_db_pool().await;
    let app = server(db_pool).await;

    let url = Url::parse("https://example.com/dinos").unwrap();
    let req = Request::new(Method::Get, url);
    let mut res: Response = app.respond(req).await?;
    // TODO: assert response body
    // let v = res.body_string().await?;
    // println!("{:?}", v);
    // assert_eq!(serde_json::to_string(&dino)?, v);

    assert_eq!(200, res.status());
    Ok(())
}

#[async_std::test]
async fn create_dino() -> tide::Result<()> {
    dotenv::dotenv().ok();
    use tide::http::{Method, Request, Response, Url};

    let dino = Dino {
        id: Uuid::new_v4(),
        name: String::from("test"),
        weight: 50,
        diet: String::from("carnivorous"),
    };

    let db_pool = make_db_pool().await;
    let app = server(db_pool).await;

    let url = Url::parse("https://example.com/dinos").unwrap();
    let mut req = Request::new(Method::Post, url);
    req.set_body(serde_json::to_string(&dino)?);
    let res: Response = app.respond(req).await?;
    assert_eq!(201, res.status());
    Ok(())
}

#[async_std::test]
async fn get_dino() -> tide::Result<()> {
    dotenv::dotenv().ok();
    use tide::http::{Method, Request, Response, Url};

    let dino = Dino {
        id: Uuid::new_v4(),
        name: String::from("test_get"),
        weight: 500,
        diet: String::from("carnivorous"),
    };

    let db_pool = make_db_pool().await;

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

    let url = Url::parse(format!("https://example.com/dinos/{}", &dino.id).as_str()).unwrap();
    let req = Request::new(Method::Get, url);

    let res: Response = app.respond(req).await?;
    assert_eq!(200, res.status());
    Ok(())
}

#[async_std::test]
async fn update_dino() -> tide::Result<()> {
    dotenv::dotenv().ok();
    use tide::http::{Method, Request, Response, Url};

    let mut dino = Dino {
        id: Uuid::new_v4(),
        name: String::from("test_update"),
        weight: 500,
        diet: String::from("carnivorous"),
    };

    let db_pool = make_db_pool().await;

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

    let url = Url::parse(format!("https://example.com/dinos/{}", &dino.id).as_str()).unwrap();
    let mut req = Request::new(Method::Put, url);
    let dinos_as_json_string = serde_json::to_string(&dino)?;
    req.set_body(dinos_as_json_string);
    let res: Response = app.respond(req).await?;
    assert_eq!(200, res.status());
    Ok(())
}

#[async_std::test]
async fn delete_dino() -> tide::Result<()> {
    dotenv::dotenv().ok();
    use tide::http::{Method, Request, Response, Url};

    let dino = Dino {
        id: Uuid::new_v4(),
        name: String::from("test_delete"),
        weight: 500,
        diet: String::from("carnivorous"),
    };

    let db_pool = make_db_pool().await;

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

    let url = Url::parse(format!("https://example.com/dinos/{}", &dino.id).as_str()).unwrap();
    let req = Request::new(Method::Delete, url);
    let res: Response = app.respond(req).await?;
    assert_eq!(204, res.status());
    Ok(())
}
