use async_std::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::{Entry, HashMap};
use std::sync::Arc;
use tide::{Body, Request, Response, Server};

#[derive(Clone, Debug)]
struct State {
    dinos: Arc<RwLock<HashMap<String, Dino>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Dino {
    name: String,
    weight: u16,
    diet: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Dinos {
    dinos: Vec<Dino>,
}

#[async_std::main]
async fn main() {
    tide::log::start();
    let dinos_store = Default::default();
    let app = server(dinos_store).await;

    app.listen("127.0.0.1:8080").await.unwrap();
}

async fn server(dinos_store: Arc<RwLock<HashMap<String, Dino>>>) -> Server<State> {
    let state = State {
        dinos: dinos_store, //Default::default(),
    };

    let mut app = tide::with_state(state);
    app.at("/").get(|_| async { Ok("ok") });

    app.at("/dinos")
        .post(|mut req: Request<State>| async move {
            let dino: Dino = req.body_json().await?;
            // let get a mut ref of our store ( hashMap )
            let mut dinos = req.state().dinos.write().await;
            dinos.insert(String::from(&dino.name), dino.clone());
            println!("{:?}", dinos);
            let mut res = Response::new(201);
            res.set_body(Body::from_json(&dino)?);
            Ok(res)
        })
        .get(|req: Request<State>| async move {
            let dinos = req.state().dinos.read().await;
            // get all the dinos as a vector
            let dinos_vec: Vec<Dino> = dinos.values().cloned().collect();
            let mut res = Response::new(200);
            res.set_body(Body::from_json(&dinos_vec)?);
            Ok(res)
        });
    app.at("/dinos/:name")
        .get(|req: Request<State>| async move {
            let mut dinos = req.state().dinos.write().await;
            let key: String = req.param("name")?;
            let res = match dinos.entry(key) {
                Entry::Vacant(_entry) => Response::new(404),
                Entry::Occupied(entry) => {
                    let mut res = Response::new(200);
                    res.set_body(Body::from_json(&entry.get())?);
                    res
                }
            };
            Ok(res)
        })
        .put(|mut req: Request<State>| async move {
            let dino_update: Dino = req.body_json().await?;
            let mut dinos = req.state().dinos.write().await;
            let key: String = req.param("name")?;
            let res = match dinos.entry(key) {
                Entry::Vacant(_entry) => Response::new(404),
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = dino_update;
                    let mut res = Response::new(200);
                    res.set_body(Body::from_json(&entry.get())?);
                    res
                }
            };
            Ok(res)
        })
        .delete(|req: Request<State>| async move {
            let mut dinos = req.state().dinos.write().await;
            let key: String = req.param("name")?;
            let deleted = dinos.remove(&key);
            let res = match deleted {
                None => Response::new(404),
                Some(_) => Response::new(204),
            };
            Ok(res)
        });

    app
}

#[async_std::test]
async fn index_page() -> tide::Result<()> {
    use tide::http::{Method, Request, Response, Url};

    let dinos_store = Default::default();
    let app = server(dinos_store).await;
    let url = Url::parse("https://example.com").unwrap();
    let req = Request::new(Method::Get, url);
    let mut res: Response = app.respond(req).await?;
    assert_eq!("ok", res.body_string().await?);
    Ok(())
}

#[async_std::test]
async fn list_dinos() -> tide::Result<()> {
    use tide::http::{Method, Request, Response, Url};

    let dino = Dino {
        name: String::from("test"),
        weight: 50,
        diet: String::from("carnivorous"),
    };

    let mut dinos_store = HashMap::new();
    dinos_store.insert(dino.name.clone(), dino);
    let dinos: Vec<Dino> = dinos_store.values().cloned().collect();
    let dinos_as_json_string = serde_json::to_string(&dinos)?;

    let state = Arc::new(RwLock::new(dinos_store));
    let app = server(state).await;

    let url = Url::parse("https://example.com/dinos").unwrap();
    let req = Request::new(Method::Get, url);
    let mut res: Response = app.respond(req).await?;
    let v = res.body_string().await?;
    assert_eq!(dinos_as_json_string, v);
    Ok(())
}

#[async_std::test]
async fn create_dino() -> tide::Result<()> {
    use tide::http::{Method, Request, Response, Url};

    let dino = Dino {
        name: String::from("test"),
        weight: 50,
        diet: String::from("carnivorous"),
    };

    let dinos_store = HashMap::new();

    let state = Arc::new(RwLock::new(dinos_store));
    let app = server(state).await;

    let url = Url::parse("https://example.com/dinos").unwrap();
    let mut req = Request::new(Method::Post, url);
    req.set_body(serde_json::to_string(&dino)?);
    let res: Response = app.respond(req).await?;
    assert_eq!(201, res.status());
    Ok(())
}

#[async_std::test]
async fn update_dino() -> tide::Result<()> {
    use tide::http::{Method, Request, Response, Url};

    let dino = Dino {
        name: String::from("test"),
        weight: 500,
        diet: String::from("carnivorous"),
    };

    let dinos_as_json_string = serde_json::to_string(&dino)?;
    let mut dinos_store = HashMap::new();
    dinos_store.insert(dino.name.clone(), dino);

    let state = Arc::new(RwLock::new(dinos_store));
    let app = server(state).await;

    let url = Url::parse("https://example.com/dinos/test").unwrap();
    let mut req = Request::new(Method::Put, url);
    req.set_body(dinos_as_json_string);
    let res: Response = app.respond(req).await?;
    assert_eq!(200, res.status());
    Ok(())
}

#[async_std::test]
async fn delete_dino() -> tide::Result<()> {
    use tide::http::{Method, Request, Response, Url};

    let dino = Dino {
        name: String::from("test"),
        weight: 50,
        diet: String::from("carnivorous"),
    };

    let mut dinos_store = HashMap::new();
    dinos_store.insert(dino.name.clone(), dino);

    let state = Arc::new(RwLock::new(dinos_store));
    let app = server(state).await;

    let url = Url::parse("https://example.com/dinos/test").unwrap();
    let req = Request::new(Method::Delete, url);
    let res: Response = app.respond(req).await?;
    assert_eq!(204, res.status());
    Ok(())
}
