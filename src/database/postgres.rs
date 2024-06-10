use deadpool_postgres::{Client, GenericClient};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::errors::pg_errors::MyError;
use crate::model::user::{User, UserLoginRequest, UserSignUpRequest};

pub async fn get_users(client: &Client) -> Result<Vec<User>, MyError> {
    let stmt = include_str!("../../sql/get_users.sql");
    let stmt = stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>();

    Ok(results)
}

pub async fn check_email_and_password_valid(
    client: &Client,
    user_info: UserLoginRequest,
) -> Option<User> {
    let stmt = include_str!("../../sql/check_user_password_valid.sql");
    println!("statement: {}", stmt);
    let stmt = client.prepare(&stmt).await.unwrap();

    client
        .query_one(&stmt, &[&user_info.email, &user_info.password])
        .await
        .map_or(None, |row| Some(User::from_row_ref(&row).unwrap()))
}

pub async fn add_user(client: &Client, user_info: UserSignUpRequest) {
    let stmt = include_str!("../../sql/add_user.sql");
    let stmt = stmt.replace("$table_fields", &User::sql_table_fields());
    println!("statement: {}", stmt);
    let stmt = client.prepare(&stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[&user_info.email, &user_info.username, &user_info.password],
        )
        .await
        .unwrap();
}

pub async fn get_user_by_email(client: &Client, email: String) -> Result<User, MyError> {
    let stmt = include_str!("../../sql/get_user_by_email.sql");
    let stmt = stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&stmt).await?;

    let rows = client.query(&stmt, &[&email]).await?;
    if let Some(row) = rows.iter().next() {
        let user = User::from_row_ref(&row)?;
        Ok(user)
    } else {
        Err(MyError::NotFound)
    }
}
