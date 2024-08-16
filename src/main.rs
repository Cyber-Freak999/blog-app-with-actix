use std::fmt::format;

use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Article {
    id: Option<i32>,
    title: String,
    content: String,
}

#[derive(Debug, serde::Serialize)]
struct MyError {
    message: String,
}

impl From<rusqlite::Error> for MyError {
    fn from(err: rusqlite::Error) -> MyError {
        MyError {
            message: format!("{}", err),
        }
    }
}

fn init_db() -> Result<()> {
    let conn = Connection::open("blog.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS articles (
            ID      INTEGER NOT NULL,
            TITLE   TEXT NOT NULL,
            CONTENT TEXT NOT NULL,
            PRIMARY KEY (ID)
        )",
        [],
    )?;
    Ok(())
}

#[post("/articles")]
async fn create_article(article: web::Json<Article>) -> impl Responder {
    let conn = Connection::open("blog.db").unwrap();
    conn.execute(
        "INSERT INTO articles (TITLE, CONTENT) VALUES (?1, ?2)",
        (&article.title, &article.content),
    )
    .unwrap();

    HttpResponse::Ok().json(article.into_inner())
}

#[get("/articles/{id}")]
async fn get_article(id: web::Path<i32>) -> impl Responder {
    let conn = Connection::open("blog.db").unwrap();
    let id = id.into_inner();
    let article: Result<Article, MyError> = async {
        let mut stmt = conn
            .prepare("SELECT ID, TITLE, CONTENT FROM articles WHERE ID = ?1")
            .map_err(MyError::from)?;
        stmt.query_row([id], |row| {
            Ok(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
            })
        })
        .map_err(MyError::from)
    }
    .await;

    HttpResponse::Ok().json(article)
}

#[get("/articles")]
async fn get_articles() -> impl Responder {
    let conn = Connection::open("blog.db").unwrap();
    let mut stmt = conn.prepare("SELECT * FROM articles").unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
            })
        })
        .unwrap();

    let articles: Vec<Article> = rows.filter_map(|result| result.ok()).collect();
    HttpResponse::Ok().json(articles)
}

#[put("/articles/{id}")]
async fn update_article(id: web::Path<i32>, article: web::Json<Article>) -> impl Responder {
    let id = id.into_inner();
    let conn = Connection::open("blog.db").unwrap();
    let updated_row = conn
        .execute(
            "UPDATE articles SET TITLE=?1, CONTENT=?2 WHERE ID = ?3",
            (&article.title, &article.content, &id),
        )
        .unwrap();

    if updated_row == 0 {
        return HttpResponse::NotFound().json(MyError {
            message: format!("Article with id {} not found", id),
        });
    }

    HttpResponse::Ok().json(article.into_inner())
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the blog")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = init_db() {
        eprintln!("Failed to initialize the database: {}", e);
        std::process::exit(1);
    }

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(create_article)
            .service(get_article)
            .service(get_articles)
            .service(update_article)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
