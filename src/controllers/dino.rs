use super::*;

use tide::{Body, Request, Response};

use crate::handlers;

pub async fn create(mut req: Request<State>) -> tide::Result {
    let mut dino: Dino = req.body_json().await?;
    let db_pool = req.state().db_pool.clone();

    let session = req.session();
    match session.get("user_id") {
        Some(id) => dino.user_id = Some(id),
        None => dino.user_id = None,
    };

    let row = handlers::dino::create(dino, &db_pool).await?;

    let mut res = Response::new(201);
    res.set_body(Body::from_json(&row)?);
    Ok(res)
}

pub async fn list(req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let rows = handlers::dino::list(&db_pool).await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&rows)?);
    Ok(res)
}

pub async fn get(req: tide::Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
    let row = handlers::dino::get(id, &db_pool).await?;

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

pub async fn update(mut req: tide::Request<State>) -> tide::Result {
    let dino: Dino = req.body_json().await?;
    let db_pool = req.state().db_pool.clone();
    let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();

    // auth operation
    let session = req.session();
    let user_id: String = session.get("user_id").unwrap_or_else(|| "".to_string());
    let row = handlers::dino::get(id, &db_pool).await?;
    if let Some(dino) = row {
        if dino.user_id.is_some() && dino.user_id.unwrap() != user_id {
            // 401
            return Ok(Response::new(401));
        }
    }

    let row = handlers::dino::update(id, dino, &db_pool).await?;

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

pub async fn delete(req: tide::Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();

    // auth operation
    let session = req.session();
    let user_id: String = session.get("user_id").unwrap_or_else(|| "".to_string());
    let row = handlers::dino::get(id, &db_pool).await?;
    if let Some(dino) = row {
        if dino.user_id.is_some() && dino.user_id.unwrap() != user_id {
            // 401
            return Ok(Response::new(401));
        }
    }

    let row = handlers::dino::delete(id, &db_pool).await?;

    let res = match row {
        None => Response::new(404),
        Some(_) => Response::new(204),
    };

    Ok(res)
}
