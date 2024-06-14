use deadpool_postgres::{Client, GenericClient};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::errors::pg_errors::MyError;
use crate::models::ip::{AddIpBdd, Ip};
use crate::models::user::{
    AddUserBdd, User, UserClaims, UserLoginRequest, UserUpdatePasswordRequest,
};

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

pub async fn update_user_password(
    client: &Client,
    user_passwords: UserUpdatePasswordRequest,
    user_info: UserClaims,
) {
    let stmt = include_str!("../../sql/change_password.sql");
    println!("statement: {}", stmt);
    let stmt = client.prepare(&stmt).await.unwrap();

    let _ = client
        .query_one(&stmt, &[&user_passwords.new_password, &user_info.id])
        .await;
}

pub async fn add_user(client: &Client, user_info: AddUserBdd) {
    let stmt = include_str!("../../sql/add_user.sql");
    let stmt = stmt.replace("$table_fields", &User::sql_table_fields());
    println!("statement: {}", stmt);
    let stmt = client.prepare(&stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[
                &user_info.email,
                &user_info.username,
                &user_info.password,
                &user_info.public_key,
                &user_info.private_key,
            ],
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

pub async fn get_ips_from_user_id(client: &Client, user_id: i64) -> Result<Vec<Ip>, MyError> {
    let stmt = include_str!("../../sql/get_ips_from_user_id.sql");
    let stmt = stmt.replace("$table_fields", &Ip::sql_table_fields());
    let stmt = client.prepare(&stmt).await?;

    let ips = client
        .query(&stmt, &[&user_id])
        .await?
        .iter()
        .map(|row| Ip::from_row_ref(row).unwrap())
        .collect::<Vec<Ip>>();

    if ips.is_empty() {
        Err(MyError::NotFound)
    } else {
        Ok(ips)
    }
}

pub async fn add_ip(client: &Client, ip_info: AddIpBdd) {
    let stmt = include_str!("../../sql/add_ip.sql");
    let stmt = stmt.replace("$table_fields", &Ip::sql_table_fields());
    println!("statement: {}", stmt);
    let stmt = client.prepare(&stmt).await.unwrap();
    client
        .query(&stmt, &[&ip_info.ip, &ip_info.user_id])
        .await
        .unwrap();
}

pub async fn get_user_by_id(client: &Client, id: i64) -> Result<User, MyError> {
    let stmt = include_str!("../../sql/get_user_by_id.sql");
    let stmt = stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&stmt).await?;

    let rows = client.query(&stmt, &[&id]).await?;
    if let Some(row) = rows.iter().next() {
        let user = User::from_row_ref(&row)?;
        Ok(user)
    } else {
        Err(MyError::NotFound)
    }
}

pub async fn check_ip_existence(client: &Client, ip: &str) -> Result<bool, MyError> {
    let stmt = include_str!("../../sql/get_ip_by_ip.sql");
    let exists: bool = client.query_one(stmt, &[&ip]).await?.get(0);
    Ok(exists)
}

pub async fn get_ip_by_ip(client: &Client, ip: &str) -> Result<Option<Ip>, MyError> {
    let stmt = include_str!("../../sql/get_ip_by_ip.sql");
    let rows = client.query(stmt, &[&ip]).await?;

    if let Some(row) = rows.get(0) {
        let ip = Ip::from_row_ref(row)?;
        Ok(Some(ip))
    } else {
        Ok(None)
    }
}
