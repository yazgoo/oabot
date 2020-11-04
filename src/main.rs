use serenity::async_trait;
use serenity::model::guild::Member;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::{Message, GuildChannel};
use serenity::framework::standard::{
    Args,
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

use std::env;

#[group]
#[commands(mva, mc, umc)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

// move user to audio channel
// mva <channel> <optional=user>
#[command]
async fn mva(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let args = args.raw().collect::<Vec<&str>>();
    let caller = msg.member(ctx).await?;
    let member = if args.len() <= 1 { 
        &caller
    }
    else {
        guild.member_named(args[1]).unwrap()
    };
    if args.len() >= 1 {
        let name = args[0];
        let id = guild.channel_id_from_name(ctx, name).await.unwrap();
        let res = member.move_to_voice_channel(ctx, id).await;
        res?;
    }
    Ok(())
}

// mute audio channel
// mc channel
#[command]
async fn mc(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() >= 1 {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let name = args.rest();
        let id = guild.channel_id_from_name(ctx, name).await.unwrap();
        let channel = &ctx.cache
            .guild_channel(id)
            .await.unwrap();
        let members = channel.members(&ctx.cache).await.unwrap();
        for member in members {
            let _ = guild.edit_member(&ctx.http, member.user.id, |m| m.mute(true)).await;
        }
    }
    Ok(())
}

// mute audio channel
// mc channel
#[command]
async fn umc(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() >= 1 {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let name = args.rest();
        let id = guild.channel_id_from_name(ctx, name).await.unwrap();
        let channel = &ctx.cache
            .guild_channel(id)
            .await.unwrap();
        let members = channel.members(&ctx.cache).await.unwrap();
        for member in members {
            let _ = guild.edit_member(&ctx.http, member.user.id, |m| m.mute(false)).await;
        }
    }
    Ok(())
}
