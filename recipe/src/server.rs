use cookbook::recipe_service_server::{RecipeService, RecipeServiceServer};
use cookbook::{ListTableOfContentsRequest, TableOfContentsResponse};

use tonic::{transport::Server, Request, Response, Status};

pub mod cookbook {
    tonic::include_proto!("cookbook");
}

#[derive(Default)]
pub struct MyService {}

#[tonic::async_trait]
impl RecipeService for MyService {
    async fn list_table_of_contents(
        &self,
        request: Request<ListTableOfContentsRequest>,
    ) -> Result<Response<TableOfContentsResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let reply = cookbook::TableOfContentsResponse {
            greeting: "Das Wars".to_string(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let service = MyService::default();

    println!("Service listening on {}", addr);
    Server::builder()
        .add_service(RecipeServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
