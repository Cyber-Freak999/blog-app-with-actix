use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Article {
    id: i32,
    title: String,
    content: String,
}

fn init_db() -> Result<()> {
    let conn = Connection::open("blog.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS articles (
            id      INTEGER PRIMARY KEY,
            title   TEXT NOT NULL,
            content TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

async fn create_post(article: web::Json<Article>) -> impl Responder {
    let conn = Connection::open("blog.db").unwrap();
    conn.execute(
        "INSERT INTO articles (title, content) VALUES (?1, ?2)",
        (&article.title, &article.content),
    )
    .unwrap();

    HttpResponse::Ok().json(article.into_inner())
}

async fn get_article(web::Path(id): web::Path<i32>) -> impl Responder {
    let conn = Connection::open("blog.db").unwrap();
    let mut stmt = conn
        .prepare("SELECT id, title, content FROM articles WHERE id = ?1")
        .unwrap();
    let post = stmt
        .query_row([id], |row| {
            Ok(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
            })
        })
        .unwrap();

    HttpResponse::Ok().json(post)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
