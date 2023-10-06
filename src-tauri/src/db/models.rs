use super::schema::user;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Queryable)]
pub struct User {
    pub id: String,
    pub org_id: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub alias: String,
    pub base_url: String,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = user)]
pub struct NewUser<'a> {
    #[diesel(column_name = id)]
    pub user_id: &'a str,
    pub org_id: &'a str,
    pub username: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub alias: &'a str,
    pub base_url: &'a str,
    pub access_token: &'a str,
    pub refresh_token: &'a str,
}
