use unix_named_pipe;
use std::path::Path;
use std::io::{BufRead, stdin, self};
use std::{env, sync::{Arc, atomic::{Ordering, AtomicBool}}, time::Duration};
use std::fs;
use serenity::{
    async_trait,
    model::{id::{GuildId, ChannelId}, channel::Message, gateway::{Ready, Activity}},
    prelude::*,
};
use std::fs::File;

use chrono::offset::Utc;

struct Handler {
    is_loop_running: AtomicBool,
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!ping") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    // We use the cache_ready event just in case some cache operation is required in whatever use
    // case you have for this.
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache built successfully!");

        // it's safe to clone Context, but Arc is cheaper for this use case.
        // Untested claim, just theoretically. :P
        let ctx = Arc::new(ctx);

        // We need to check that the loop is not already running when this event triggers,
        // as this event triggers every time the bot enters or leaves a guild, along every time the
        // ready shard event triggers.
        //
        // An AtomicBool is used because it doesn't require a mutable reference to be changed, as
        // we don't have one due to self being an immutable reference.
        if !self.is_loop_running.load(Ordering::Relaxed) {

            // We have to clone the Arc, as it gets moved into the new thread.
            let ctx1 = Arc::clone(&ctx);
            // tokio::spawn creates a new green thread that can run in parallel with the rest of
            // the application.
            /*
            tokio::spawn(async move {
                loop {
                    // We clone Context again here, because Arc is owned, so it moves to the
                    // new function.
                    log_system_load(Arc::clone(&ctx1)).await;
                    tokio::time::delay_for(Duration::from_secs(120)).await;
                }
            });
                    */
            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    let fifo = "/tmp/oabot";
                    if !Path::new(fifo).exists() {
                        unix_named_pipe::create(fifo, None);
                    }
                    if let Ok(lines) = read_lines(fifo) {
                        // Consumes the iterator, returns an (Optional) String
                        for line in lines {
                            if let Ok(command) = line {
                                println!("{}", command);
                                if command == "afk" {
                                    change_chan(Arc::clone(&ctx2), String::from("AFK")).await;     
                                }
                                if command == "core" {
                                    change_chan(Arc::clone(&ctx2), String::from("Core")).await;     
                                }
                            }
                        }
                    }
                }
            });

            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

async fn change_chan(ctx: Arc<Context>, chan: String) {
    let cache = &ctx.cache;
    let http = &ctx.http;
    let guild = cache.guilds().await[0];
    let guild = cache.guild(guild).await.unwrap();
    let member = guild.member_named("oab").unwrap();
    let id = guild.channel_id_from_name(cache, chan).await.unwrap();
    let _ = member.move_to_voice_channel(http, id).await;
}

async fn log_system_load(ctx: Arc<Context>) {
    let cpu_load = sys_info::loadavg().unwrap();
    let mem_use = sys_info::mem_info().unwrap();

    // We can use ChannelId directly to send a message to a specific channel; in this case, the
    // message would be sent to the #testing channel on the discord server.
    if let Err(why) = ChannelId(772066296165826591).send_message(&ctx, |m| m.embed(|e| {
        e.title("System Resource Load");
        e.field(
            "CPU Load Average",
            format!("{:.2}%", cpu_load.one * 10.0),
            false,
        );
        e.field(
            "Memory Usage",
            format!("{:.2} MB Free out of {:.2} MB", mem_use.free as f32 / 1000.0, mem_use.total as f32 / 1000.0),
            false,
        );
        e
    })).await {
        eprintln!("Error sending message: {:?}", why);
    };
}

async fn set_status_to_current_time(ctx: Arc<Context>) {
    let current_time = Utc::now();
    let formatted_time = current_time.to_rfc2822();

    ctx.set_activity(Activity::playing(&formatted_time)).await;
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}
