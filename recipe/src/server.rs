use std::sync::{Arc, RwLock};

use axum::{routing, Router};
use recipers::repository::Repository;

use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handler::{
    recipe_delete, recipe_get, recipe_put, recipe_share, recipes_get, recipes_post,
};
// use cookbook::recipe_service_server::{RecipeService, RecipeServiceServer};
// use cookbook::{ListTableOfContentsRequest, TableOfContentsResponse};
// use tonic::{transport::Server, Request, Response, Status};

// pub mod cookbook {
//     tonic::include_proto!("cookbook");
// }

// #[derive(Default)]
// pub struct MyService {}

// #[tonic::async_trait]
// impl RecipeService for MyService {
//     async fn list_table_of_contents(
//         &self,
//         request: Request<ListTableOfContentsRequest>,
//     ) -> Result<Response<TableOfContentsResponse>, Status> {
//         println!("Got a request from {:?}", request.remote_addr());
//         let reply = cookbook::TableOfContentsResponse {
//             greeting: "Das Wars".to_string(),
//         };

//         Ok(Response::new(reply))
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse().unwrap();
//     let service = MyService::default();

//     println!("Service listening on {}", addr);
//     Server::builder()
//         .add_service(RecipeServiceServer::new(service))
//         .serve(addr)
//         .await?;

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,recipers=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = router();
    tracing::debug!("listening to 0.0.0.0:8080");
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn router() -> Router {
    let repository = Arc::new(RwLock::new(Repository::new()));

    Router::new()
        .route("/", routing::get(|| async { "Hello World!" }))
        .route(
            "/cookbook/recipe",
            routing::get(recipes_get)
                .post(recipes_post)
                .with_state(repository.clone())
                .layer(TraceLayer::new_for_http()),
        )
        .route(
            "/cookbook/recipe/:id",
            routing::get(recipe_get)
                .put(recipe_put)
                .delete(recipe_delete)
                .with_state(repository.clone())
                .layer(TraceLayer::new_for_http()),
        )
        .route(
            "/cookbook/recipe/share",
            routing::get(recipe_share).with_state(repository.clone()),
        )
}

type AppState = Arc<RwLock<Repository>>;

mod handler;

#[cfg(test)]
mod test {

    use hyper::StatusCode;
    use hyper::{body::to_bytes, Body, Request};
    use recipers::TableOfContents;

    use crate::router;

    use tower::Service;
    use tower::ServiceExt;

    #[tokio::test]
    async fn get_toc() {
        let request = Request::builder().uri("/").body(Body::empty()).unwrap();

        let response = router().ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK)
    }

    #[tokio::test]
    async fn get_toc2() {
        let request = Request::builder()
            .uri("/cookbook/recipe")
            .header("Range", "bytes=0-9")
            .body(Body::empty())
            .unwrap();

        let response = router().ready().await.unwrap().call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let raw_body = to_bytes(response.into_body()).await.unwrap();
        let got: TableOfContents = serde_json::from_slice(&raw_body).unwrap();
        println!("{:?}", got);

        assert_eq!(got, TableOfContents::empty())
    }
}
