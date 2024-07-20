use std::env;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

mod models;

#[derive(Deserialize)]
struct CreateItem {
    name: String,
    description: String,
}

#[derive(Serialize)]
struct Item {
    id: i32,
    name: String,
    description: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/items", web::post().to(create_item))
            .route("/items", web::get().to(get_items))
            .route("/items/{id}", web::get().to(get_item))
            .route("/items/{id}", web::put().to(update_item))
            .route("/items/{id}", web::delete().to(delete_item))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn create_item(pool: web::Data<PgPool>, item: web::Json<CreateItem>) -> impl Responder {
    let result = sqlx::query!(
        "INSERT INTO items (name, description) VALUES ($1, $2) RETURNING id, name, description",
        item.name,
        item.description
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(record) => {
            let new_item = Item {
                id: record.id,
                name: record.name,
                description: record.description,
            };
            HttpResponse::Ok().json(new_item)
        }
        Err(_) => HttpResponse::InternalServerError().body("Error to create item"),
    }
}

async fn get_items(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as!(Item, "SELECT id, name, description FROM items")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_item(pool: web::Data<PgPool>, item_id: web::Path<i32>) -> impl Responder {
    let result = sqlx::query_as!(
        Item,
        "SELECT id, name, description FROM items WHERE id = $1",
        *item_id
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

async fn update_item(
    pool: web::Data<PgPool>,
    item_id: web::Path<i32>,
    item: web::Json<CreateItem>,
) -> impl Responder {
    let result = sqlx::query!(
        "UPDATE items SET name = $1, description = $2 WHERE id = $3",
        item.name,
        item.description,
        *item_id,
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn delete_item(pool: web::Data<PgPool>, item_id: web::Path<i32>) -> impl Responder {
    let result = sqlx::query!("DELETE FROM items WHERE id = $1", *item_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
