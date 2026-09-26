pub mod chat;
pub mod models;
pub mod ollama;
pub mod platform;

pub use chat::commands as chat_commands;
pub use chat::contracts as chat_contracts;
pub use models::commands as model_commands;
pub use models::contracts as model_contracts;
pub use models::queries as model_queries;
pub use ollama::commands as ollama_commands;
pub use ollama::contracts as ollama_contracts;
pub use ollama::queries as ollama_queries;
pub use platform::commands as platform_commands;
pub use platform::contracts as platform_contracts;
pub use platform::queries as platform_queries;
