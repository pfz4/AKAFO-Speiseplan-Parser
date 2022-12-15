use std::env;

use serenity::{
    async_trait,
    framework::StandardFramework,
    model::prelude::{
        command::Command,
        interaction::{Interaction, InteractionResponseType},
        GuildId, Ready,
    },
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};

mod menu_command;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "menu" => menu_command::run(&command.data.options).await,
                _ => "not implemented :(".to_string(),
            };

            if let Err(err) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", err);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let commands = Command::create_global_application_command(&ctx.http, |command| {
            menu_command::register(command)
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN env variable");

    let mut client = Client::builder(
        token,
        GatewayIntents::non_privileged()
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler(Handler)
    .framework(StandardFramework::new())
    .await
    .expect("working discord client");

    if let Err(err) = client.start().await {
        println!("Client error: {:?}", err);
    }
}
