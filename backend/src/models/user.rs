use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub role: String,
    pub status: String,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub role: String,
    pub status: String,
    pub email_verified: Option<bool>,
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub email_verified: Option<bool>,
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<NaiveDateTime>,
}

impl User {
    pub fn find_by_id(conn: &mut PgConnection, user_id: i32) -> Result<Option<Self>, diesel::result::Error> {
        users::table
            .find(user_id)
            .first::<User>(conn)
            .optional()
    }

    pub fn find_by_username(conn: &mut PgConnection, username: &str) -> Result<Option<Self>, diesel::result::Error> {
        users::table
            .filter(users::username.eq(username))
            .first::<User>(conn)
            .optional()
    }

    pub fn find_by_email(conn: &mut PgConnection, email: &str) -> Result<Option<Self>, diesel::result::Error> {
        users::table
            .filter(users::email.eq(email))
            .first::<User>(conn)
            .optional()
    }

    pub fn find_by_verification_token(conn: &mut PgConnection, token: &str) -> Result<Option<Self>, diesel::result::Error> {
        users::table
            .filter(users::email_verification_token.eq(token))
            .first::<User>(conn)
            .optional()
    }

    pub fn create(conn: &mut PgConnection, new_user: NewUser) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(conn)
    }

    pub fn update(conn: &mut PgConnection, user_id: i32, update_user: UpdateUser) -> Result<Self, diesel::result::Error> {
        diesel::update(users::table.find(user_id))
            .set(update_user)
            .get_result(conn)
    }

    pub fn delete(conn: &mut PgConnection, user_id: i32) -> Result<usize, diesel::result::Error> {
        diesel::delete(users::table.find(user_id))
            .execute(conn)
    }

    pub fn list(conn: &mut PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        users::table
            .order(users::created_at.desc())
            .load::<User>(conn)
    }
} 