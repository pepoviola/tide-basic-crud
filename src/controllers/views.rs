use super::*;
use tide::{Request, Response};

pub async fn index(req: Request<State>) -> tide::Result {
    let tera = req.state().tera.clone();
    let db_pool = req.state().db_pool.clone();
    let rows = handlers::dino::list(&db_pool).await?;

    let session = req.session();
    println!("{:?}", session);
    let user_id: String = session.get("user_id").unwrap_or_else(|| "".to_string());
    let user_name: String = session.get("user_name").unwrap_or_else(|| "".to_string());

    println!(" data {}, {}", user_id, user_name);

    tera.render_response(
        "index.html",
        &context! {
           "title" => String::from("Tide basic CRUD"),
           "dinos" => rows,
           "user_id" => user_id,
           "user_name" => user_name
        },
    )
}

pub async fn new(req: Request<State>) -> tide::Result {
    let tera = req.state().tera.clone();
    let session = req.session();
    let user_id: String = session.get("user_id").unwrap_or_else(|| "".to_string());

    tera.render_response(
        "form.html",
        &context! {
            "title" => String::from("Create new dino"),
            "user_id" => user_id,
        },
    )
}

pub async fn edit(req: Request<State>) -> tide::Result {
    let tera = req.state().tera.clone();
    let session = req.session();
    let user_id: String = session.get("user_id").unwrap_or_else(|| "".to_string());

    let db_pool = req.state().db_pool.clone();
    let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
    let row = handlers::dino::get(id, &db_pool).await?;

    let res = match row {
        None => Response::new(404),
        Some(row) => {
            let mut r = Response::new(200);
            let b = tera.render_body(
                "form.html",
                &context! {
                    "title" => String::from("Edit dino"),
                    "dino" => row,
                    "user_id" => user_id,
                },
            )?;
            r.set_body(b);
            r
        }
    };

    Ok(res)
}
