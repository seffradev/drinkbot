use std::env;

use lazy_static::lazy_static;
use poise::serenity_prelude as serenity;
use thecocktaildb_rs::{Client as CocktailClient, Cocktails};
use tracing::warn;

struct Data;
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

lazy_static! {
    static ref COCKTAIL: CocktailClient =
        CocktailClient::new(&env::var("THECOCKTAILDB_TOKEN").expect("Expected THECOCKTAILDB_TOKEN in the environment")).unwrap();
}

/// Fetches a random drink
#[poise::command(slash_command, prefix_command)]
async fn random(ctx: Context<'_>) -> Result<(), anyhow::Error> {
    let cocktail = Cocktails::random(&COCKTAIL).await?;

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

    ctx.say(message).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    if let Err(err) = dotenvy::dotenv() {
        warn!("Failed to read .env-file: {}", err);
    }

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![random()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data)
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot.
    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}
