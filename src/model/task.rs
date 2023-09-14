use crate::ctx::Ctx;
use crate::model::{Error, Result};

use super::ModelManager;

#[derive(Debug)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

pub struct TaskForCreate {
    pub title: String,
}

pub struct TaskUpdate {
    pub title: Option<String>,
}

pub struct TaskBmc;

impl TaskBmc {
    pub async fn create(_ctx: Ctx, mm: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        let db = mm.db();

        // query_as返回一个QueryAs的结构体，
        // 其中的O类型为实现了FromRow trait的类型 O: for<'r> FromRow<'r, DB::Row>,
        // 单独的i64是没有实现的，而元组的用宏实现了FromRow
        // implement FromRow for tuples of types that implement Decode
        // up to tuples of 9 values
        /* macro_rules! impl_from_row_for_tuple {
            ($( ($idx:tt) -> $T:ident );+;) => {
                impl<'r, R, $($T,)+> FromRow<'r, R> for ($($T,)+)
                where
                    R: Row,
                    usize: crate::column::ColumnIndex<R>,
                    $($T: crate::decode::Decode<'r, R::Database> + crate::types::Type<R::Database>,)+
                {
                    #[inline]
                    fn from_row(row: &'r R) -> Result<Self, Error> {
                        Ok(($(row.try_get($idx as usize)?,)+))
                    }
                }
            };
        }

        impl_from_row_for_tuple!(
            (0) -> T1;
        )
        */
        let (id,) = sqlx::query_as("INSERT INTO task (title) VALUES ($1) RETURNING id")
            .bind(task_c.title)
            .fetch_one(db)
            .await?; // 在这里可以使用？的原因是在model::Error中已经实现了From<sqlx::Error>

        Ok(id)
    }
}

// region:    ---Tests
#[cfg(test)]
mod tests {
    #![allow(unused)]
    use crate::_dev_utils;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        // Setup && Features
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok Title";

        // --Exec
        let task_c = TaskForCreate {
            title: fx_title.to_string(),
        };
        let id = TaskBmc::create(ctx, &mm, task_c).await?;

        // Check
        let (title,): (String,) = sqlx::query_as("SELECT title FROM task WHERE id = $1")
            .bind(id)
            .fetch_one(mm.db())
            .await?;
        assert_eq!(title, fx_title);

        // --Cleanup
        let count = sqlx::query("DELETE FROM task WHERE id = $1")
            .bind(id)
            .execute(mm.db())
            .await?
            .rows_affected();
        assert_eq!(count, 1);

        Ok(())
    }
}
// endregion: ---Tests