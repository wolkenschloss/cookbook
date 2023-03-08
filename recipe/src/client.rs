use cookbook::recipe_service_client::RecipeServiceClient;
use cookbook::ListTableOfContentsRequest;

pub mod cookbook {
    tonic::include_proto!("cookbook");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RecipeServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(ListTableOfContentsRequest {
        name: "Gandalf".into(),
        age: 18,
    });

    let response = client.list_table_of_contents(request).await?;
    println!("RESPONSE: {:?}", response);

    Ok(())
}
