use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::{comments, posts, users, pages};
use super::{Post, User, Page};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = comments)]
#[diesel(belongs_to(Post, foreign_key = post_id))]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(belongs_to(Page, foreign_key = page_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Comment {
    pub id: i32,
    pub post_id: Option<i32>,
    pub user_id: Option<i32>,
    pub content: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub page_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = comments)]
pub struct NewComment {
    pub post_id: Option<i32>,
    pub user_id: Option<i32>,
    pub content: String,
    pub page_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = comments)]
pub struct UpdateComment {
    pub content: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct CommentWithRelations {
    pub id: i32,
    pub post_id: Option<i32>,
    pub user_id: Option<i32>,
    pub content: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub author_username: Option<String>,
    pub post_title: Option<String>,
    pub page_id: Option<i32>,
    pub page_title: Option<String>,
}

impl Comment {
    pub fn find_by_id(conn: &mut PgConnection, comment_id: i32) -> Result<Option<Self>, diesel::result::Error> {
        comments::table
            .find(comment_id)
            .first::<Comment>(conn)
            .optional()
    }

    pub fn find_with_relations(conn: &mut PgConnection, comment_id: i32) -> Result<Option<CommentWithRelations>, diesel::result::Error> {
        comments::table
            .left_join(users::table.on(comments::user_id.eq(users::id.nullable())))
            .left_join(posts::table.on(comments::post_id.eq(posts::id.nullable())))
            .left_join(pages::table.on(comments::page_id.eq(pages::id.nullable())))
            .filter(comments::id.eq(comment_id))
            .select((
                comments::id,
                comments::post_id,
                comments::user_id,
                comments::content,
                comments::created_at,
                comments::updated_at,
                users::username.nullable(),
                posts::title.nullable(),
                comments::page_id,
                pages::title.nullable(),
            ))
            .first::<CommentWithRelations>(conn)
            .optional()
    }

    pub fn create(conn: &mut PgConnection, new_comment: NewComment) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(comments::table)
            .values(&new_comment)
            .get_result(conn)
    }

    pub fn update(conn: &mut PgConnection, comment_id: i32, mut update_comment: UpdateComment) -> Result<Self, diesel::result::Error> {
        update_comment.updated_at = Some(chrono::Utc::now().naive_utc());
        diesel::update(comments::table.find(comment_id))
            .set(update_comment)
            .get_result(conn)
    }

    pub fn delete(conn: &mut PgConnection, comment_id: i32) -> Result<usize, diesel::result::Error> {
        diesel::delete(comments::table.find(comment_id))
            .execute(conn)
    }

    pub fn list(conn: &mut PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        comments::table
            .order(comments::created_at.desc())
            .load::<Comment>(conn)
    }

    pub fn list_with_relations(conn: &mut PgConnection) -> Result<Vec<CommentWithRelations>, diesel::result::Error> {
        comments::table
            .left_join(users::table.on(comments::user_id.eq(users::id.nullable())))
            .left_join(posts::table.on(comments::post_id.eq(posts::id.nullable())))
            .left_join(pages::table.on(comments::page_id.eq(pages::id.nullable())))
            .order(comments::created_at.desc())
            .select((
                comments::id,
                comments::post_id,
                comments::user_id,
                comments::content,
                comments::created_at,
                comments::updated_at,
                users::username.nullable(),
                posts::title.nullable(),
                comments::page_id,
                pages::title.nullable(),
            ))
            .load::<CommentWithRelations>(conn)
    }

    pub fn find_by_post(conn: &mut PgConnection, post_id: i32) -> Result<Vec<Self>, diesel::result::Error> {
        comments::table
            .filter(comments::post_id.eq(post_id))
            .order(comments::created_at.asc())
            .load::<Comment>(conn)
    }

    pub fn find_by_page(conn: &mut PgConnection, page_id: i32) -> Result<Vec<Self>, diesel::result::Error> {
        comments::table
            .filter(comments::page_id.eq(page_id))
            .order(comments::created_at.asc())
            .load::<Comment>(conn)
    }

    pub fn find_by_user(conn: &mut PgConnection, user_id: i32) -> Result<Vec<Self>, diesel::result::Error> {
        comments::table
            .filter(comments::user_id.eq(user_id))
            .order(comments::created_at.desc())
            .load::<Comment>(conn)
    }
} 