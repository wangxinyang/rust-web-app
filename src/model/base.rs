use sqlb::{HasFields, SqlBuilder};
use sqlx::postgres::PgRow;
use sqlx::FromRow;

use crate::ctx::Ctx;
use crate::model::error::{Error, Result};
use crate::model::ModelManager;

pub trait DbBmc {
    // associated constants关联常量
    const TABLE: &'static str;
}

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();
    let fields = data.not_none_fields();

    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
        .await?;

    Ok(id)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    E: HasFields,
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
        .columns(E::field_names())
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

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Send + Unpin,
    E: HasFields,
{
    let db = mm.db();
    let entity = sqlb::select()
        .columns(E::field_names())
        .table(MC::TABLE)
        .order_by("id")
        .fetch_all(db)
        .await?;

    Ok(entity)
}

pub async fn update<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();
    let fields = data.not_none_fields();

    let count = sqlb::update()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .data(fields)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
    MC: DbBmc,
{
    let db = mm.db();
    let entity = sqlb::delete()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .exec(db)
        .await?;

    if entity == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
