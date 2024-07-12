use std::env;

use lazy_static::lazy_static;
use serenity::{async_trait, model::channel::Message, prelude::*};
use thecocktaildb_rs::{Client as CocktailClient, Cocktails};
use tracing::error;

struct Handler;

lazy_static! {
    static ref COCKTAIL: CocktailClient =
        CocktailClient::new(&env::var("THECOCKTAILDB_TOKEN").expect("Expected THECOCKTAILDB_TOKEN in the environment")).unwrap();
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!drink" {
            let cocktail = match Cocktails::random(&COCKTAIL).await {
                Ok(cocktails) => cocktails,
                Err(err) => {
                    error!("{}", err);
                    return;
                }
            };

            let message = cocktail.first().map_or("no drink found".into(), |cocktail| {
                if cocktail.drink.is_none() {
                    return "no drink found".into();
                }

                if cocktail.ingredients.is_empty() {
                    return "no drink found".into();
                }

                let ingredients = cocktail
                    .ingredients
                    .iter()
                    .filter(|i| i.ingredient.is_some())
                    .filter(|i| i.measure.is_some())
                    .map(|i| format!("{} ({})", i.ingredient.clone().unwrap().trim(), i.measure.clone().unwrap().trim()))
                    .collect::<Vec<_>>();

                if cocktail.instructions.en.is_none() {
                    return "no drink found".into();
                }

                if cocktail.glass.is_none() {
                    return "no drink found".into();
                }

                format!(
                    "**Name**: {}\n**Ingredients**: {}\n**Instructions**: {}\n**Glass**: {}",
                    cocktail.drink.clone().unwrap().trim(),
                    ingredients.join(", ").trim(),
                    cocktail.instructions.en.clone().unwrap().trim(),
                    cocktail.glass.clone().unwrap().trim(),
                )
            });

            if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                println!("Error sending message: {why:?}");
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv()?;

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}
