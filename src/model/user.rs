use sqlb::{Fields, HasFields};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

use crate::ctx::Ctx;
use crate::model::{
    base::{self, DbBmc},
    Error, ModelManager, Result,
};

#[derive(FromRow, Fields)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Fields)]
pub struct UserForCreate {
    pub username: String,
    pub pwd_clear: String,
}

pub struct UserForInsert {
    pub username: String,
}

pub struct UserForLogin {
    pub id: i64,
    pub username: String,

    pub pwd: Option<String>,
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}

pub struct UserForAuth {
    pub id: i64,
    pub username: String,

    pub token_salt: Uuid,
}

/// marker trait
pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Send + Unpin {}

impl UserBy for User {}

pub struct UserBmc;

impl DbBmc for UserBmc {
    const TABLE: &'static str = "user";
}

impl UserBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }
    // User标记了Fields，所以可以不用使用HasFields
    /*  pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<User> {
        base::get::<Self, _>(ctx, mm, id).await
    } */

    pub async fn first_by_username<E>(
        ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        let db = mm.db();

        let user = sqlb::select()
            .table(Self::TABLE)
            .and_where("username", "=", username)
            .fetch_optional::<_, E>(db)
            .await?;

        Ok(user)
    }
}

// region:    ---Tests
#[cfg(test)]
mod tests {
    #![allow(unused)]
    use crate::_dev_utils;

    use super::*;
    use anyhow::{Context, Result};
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_get_ok() {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let username = "demo1";

        let user: User = UserBmc::get(&ctx, &mm, 1000).await.unwrap();
        assert_eq!(username, user.username);
    }

    #[serial]
    #[tokio::test]
    async fn test_first_ok_demo1() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let username = "demo1";

        // Exec
        let user: User = UserBmc::first_by_username(&ctx, &mm, username)
            .await?
            .context("Should have demo1")?;

        // Check
        assert_eq!(username, user.username);

        Ok(())
    }
}
// endregion: ---Tests
