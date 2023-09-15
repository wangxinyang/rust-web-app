use sqlx::postgres::PgRow;
use sqlx::FromRow;

use crate::ctx::Ctx;
use crate::model::error::{Error, Result};
use crate::model::ModelManager;

pub trait DbBmc {
    // associated constants关联常量
    const TABLE: &'static str;
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Send + Unpin,
{
    let db = mm.db();

    // let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);

    /*     let entity: E = sqlx::query_as(&sql)
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(Error::EntityNotFound {
        entity: MC::TABLE,
        id,
    })?; */
    let entity = sqlb::select()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}
