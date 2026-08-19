use blog_client::{BlogClient, Transport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for transport in [
        Transport::Http("http://localhost:8080".into()),
        Transport::Grpc("http://localhost:50051".into()),
    ] {
        let label = match &transport {
            Transport::Http(_) => "HTTP",
            Transport::Grpc(_) => "gRPC",
        };

        let mut client = BlogClient::new(transport).await?;

        assert!(client.create_post("t", "c").await.is_err());

        let page = client.list_posts(Some(5), None).await?;
        println!("{label}: постов всего {}", page.total);

        let auth = client
            .login("ivan", "secret123")
            .await
            .or_else(|_| Err("Нужно зарегистрировать пользователя"))?;
        println!("{label}: вошли как {}", auth.user.username);

        let post = client.create_post("Из клиента", "Проверка").await?;
        println!("{label}: создан пост {}", post.id);

        client.delete_post(post.id).await?;
        println!("{label}: пост удален")
    }

    Ok(())
}
