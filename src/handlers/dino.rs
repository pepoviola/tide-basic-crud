use super::*;
use crate::Dino;
use sqlx::{query, query_as, PgPool};

pub async fn create(dino: Dino, db_pool: &PgPool) -> tide::Result<Dino> {
    let row: Dino = query_as!(
        Dino,
        r#"
        INSERT INTO dinos (id, name, weight, diet, user_id) VALUES
        ($1, $2, $3, $4, $5) returning id as "id!", name, weight, diet, user_id
        "#,
        dino.id,
        dino.name,
        dino.weight,
        dino.diet,
        dino.user_id
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| Error::new(409, e))?;

    Ok(row)
}
pub async fn list(db_pool: &PgPool) -> tide::Result<Vec<Dino>> {
    let rows = query_as!(
        Dino,
        r#"
        SELECT id, name, weight, diet, user_id from dinos
        "#
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| Error::new(409, e))?;

    Ok(rows)
}

pub async fn get(id: Uuid, db_pool: &PgPool) -> tide::Result<Option<Dino>> {
    let row = query_as!(
        Dino,
        r#"
        SELECT  id, name, weight, diet, user_id from dinos
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| Error::new(409, e))?;

    Ok(row)
}
pub async fn delete(id: Uuid, db_pool: &PgPool) -> tide::Result<Option<()>> {
    let row = query!(
        r#"
        delete from dinos
        WHERE id = $1
        returning id
        "#,
        id
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| Error::new(409, e))?;

    let r = match row {
        None => None,
        Some(_) => Some(()),
    };

    Ok(r)
}

pub async fn update(id: Uuid, dino: Dino, db_pool: &PgPool) -> tide::Result<Option<Dino>> {
    let row = query_as!(
        Dino,
        r#"
        UPDATE dinos SET name = $2, weight = $3, diet = $4, user_id = $5
        WHERE id = $1
        returning id, name, weight, diet, user_id
        "#,
        id,
        dino.name,
        dino.weight,
        dino.diet,
        dino.user_id
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| Error::new(409, e))?;

    Ok(row)
}
