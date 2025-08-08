use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::pages;
use super::User;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = pages)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Page {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub user_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub slug: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = pages)]
pub struct NewPage {
    pub title: String,
    pub content: String,
    pub user_id: Option<i32>,
    pub slug: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = pages)]
pub struct UpdatePage {
    pub title: Option<String>,
    pub content: Option<String>,
    pub user_id: Option<i32>,
    pub updated_at: Option<NaiveDateTime>,
    pub slug: Option<String>,
    pub status: Option<String>,
}

impl Page {
    pub fn find_by_id(conn: &mut PgConnection, page_id: i32) -> Result<Option<Self>, diesel::result::Error> {
        pages::table
            .find(page_id)
            .first::<Page>(conn)
            .optional()
    }

    pub fn find_by_slug(conn: &mut PgConnection, slug_value: &str) -> Result<Option<Self>, diesel::result::Error> {
        pages::table
            .filter(pages::slug.eq(slug_value))
            .first::<Page>(conn)
            .optional()
    }

    pub fn create(conn: &mut PgConnection, new_page: NewPage) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(pages::table)
            .values(&new_page)
            .get_result(conn)
    }

    pub fn update(conn: &mut PgConnection, page_id: i32, mut update_page: UpdatePage) -> Result<Self, diesel::result::Error> {
        update_page.updated_at = Some(chrono::Utc::now().naive_utc());
        diesel::update(pages::table.find(page_id))
            .set(update_page)
            .get_result(conn)
    }

    pub fn delete(conn: &mut PgConnection, page_id: i32) -> Result<usize, diesel::result::Error> {
        diesel::delete(pages::table.find(page_id))
            .execute(conn)
    }

    pub fn list(conn: &mut PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        pages::table
            .order(pages::updated_at.desc())
            .load::<Page>(conn)
    }

    pub fn find_by_user(conn: &mut PgConnection, user_id: i32) -> Result<Vec<Self>, diesel::result::Error> {
        pages::table
            .filter(pages::user_id.eq(user_id))
            .order(pages::updated_at.desc())
            .load::<Page>(conn)
    }
} 