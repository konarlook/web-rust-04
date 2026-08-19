pub mod token;

use anyhow::{Context, Result};
use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};
use std::process::ExitCode;

const DEFAULT_HTTP_SERVER: &str = "http://localhost:8080";
const DEFAULT_GRPC_SERVER: &str = "http://localhost:50051";

#[derive(Debug, Subcommand)]
enum Commands {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },

    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },

    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },

    Get {
        #[arg(long)]
        id: i64,
    },

    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },

    Delete {
        #[arg(long)]
        id: i64,
    },

    List {
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
}

#[derive(Debug, Parser)]
#[command(name = "blog-cli", version, about = "Клиент блога: HTTP и gRPC")]
struct Cli {
    #[arg(long, global = true)]
    grpc: bool,

    #[arg(long, global = true)]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(error) = run().await {
        eprintln!("{}", error);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let endpoint = cli.server.unwrap_or_else(|| {
        if cli.grpc {
            DEFAULT_GRPC_SERVER
        } else {
            DEFAULT_HTTP_SERVER
        }
        .to_owned()
    });

    let transport = if cli.grpc {
        Transport::Grpc(endpoint)
    } else {
        Transport::Http(endpoint)
    };

    let mut client = BlogClient::new(transport)
        .await
        .context("Failed connect to server")?;

    if let Some(saved) = token::load()? {
        client.set_token(saved);
    }

    match cli.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let auth = client.register(&username, &email, &password).await?;
            token::save(&auth.token)?;
            println!(
                "Пользователь {} зарегистрирован (id {})",
                auth.user.username, auth.user.id,
            );
        }
        Commands::Login { username, password } => {
            let auth = client.login(&username, &password).await?;
            token::save(&auth.token)?;
            println!("Вход выполнен: {}", auth.user.username);
        }
        Commands::Create { title, content } => {
            let post = client.create_post(&title, &content).await?;
            println!("Пост создан (id {})", post.id);
        }
        Commands::Get { id } => {
            let post = client.get_post(id).await?;
            println!("#{} {}", post.id, post.title);
            println!("Автор: {}", post.author_id);
            println!("Создан: {}", post.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("Изменён: {}", post.updated_at.format("%Y-%m-%d %H:%M:%S"));
            println!();
            println!("{}", post.content);
        }
        Commands::Update { id, title, content } => {
            let post = client
                .update_post(id, title.as_deref(), content.as_deref())
                .await?;
            println!("Пост {} обновлён", post.id);
        }
        Commands::Delete { id } => {
            client.delete_post(id).await?;
            println!("Пост {id} удалён");
        }
        Commands::List { limit, offset } => {
            let page = client.list_posts(limit, offset).await?;

            if page.posts.is_empty() {
                println!("Постов нет");
            } else {
                println!(
                    "Показаны {}–{} из {}",
                    page.offset + 1,
                    page.offset + page.posts.len() as i64,
                    page.total
                );
                for post in &page.posts {
                    println!(
                        "  #{:<4} {:<40} {}",
                        post.id,
                        post.title,
                        post.created_at.format("%Y-%m-%d %H:%M")
                    );
                }
            }
        }
    }

    Ok(())
}
