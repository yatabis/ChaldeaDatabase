use actix_web::{get, web, Responder};
use serde::{Serialize, Deserialize, Serializer};
use std::collections::{HashMap, HashSet};
use sqlx::{Connection, Row};
use actix_web::http::StatusCode;

#[derive(Deserialize)]
pub struct Targets {
    targets: String,
}

#[derive(Serialize)]
#[derive(Debug, Clone)]
pub struct Table {
    pub id:    i32,
    pub name:  String,
    pub table: Vec<i32>
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String
}

pub enum ResponseBody {
    Ok(Vec<Table>),
    Err(ErrorResponse)
}

impl Serialize for ResponseBody {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        match self {
            ResponseBody::Ok(table) => table.serialize(serializer),
            ResponseBody::Err(e) => e.serialize(serializer)
        }
    }
}

fn validate_targets(targets: &str) -> Vec<i32> {
    let mut ids =  Vec::new();
    let mut seen = HashSet::new();
    for t in targets.split(",") {
        if let Ok(id) = t.trim().parse() {
            if !seen.contains(&id) {
                ids.push(id);
                seen.insert(id);
            }
        }
    }
    ids
}

async fn fetch_table(ids: &Vec<i32>) -> Result<Vec<Table>, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let mut conn = sqlx::PgConnection::connect(&database_url).await?;
    let mut stmt = "select * from tables where id in (".to_string();
    stmt += &(1..=ids.len()).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
    stmt += ");";
    let mut query = sqlx::query(&stmt);
    for id in ids {
        query = query.bind(id);
    }
    let mut table_map = HashMap::new();
    for row in query.fetch_all(&mut conn).await? {
        let mut table = Vec::new();
        let id: i32 = row.get("id");
        let name = row.get("name");
        for i in 1..=15 {
            table.push(row.get(&*format!("lv{}", i)))
        }
        table_map.insert(id, Table {
            id,
            name,
            table
        });
    }
    let mut tables = Vec::new();
    for id in ids {
        if let Some(table) = table_map.remove(id) {
            tables.push(table);
        }
    }
    Ok(tables)
}

#[get("/tables")]
pub async fn get_tables(query: web::Query<Targets>) -> impl Responder {
    let ids = validate_targets(&query.targets);
    if ids.is_empty() {
        return web::Json(ResponseBody::Ok(Vec::new())).with_status(StatusCode::OK);
    }
    match fetch_table(&ids).await {
        Ok(tables) => web::Json(ResponseBody::Ok(tables)).with_status(StatusCode::OK),
        Err(e) => web::Json(ResponseBody::Err(
            ErrorResponse{
                message: e.to_string()
            })).with_status(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
