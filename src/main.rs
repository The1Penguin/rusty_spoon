mod commands;

use commands::{general::*, sentinel::*};

use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    sync::Arc,
};

use serenity::{
    async_trait,
    framework::standard::{
        buckets::{LimitedFor, RevertBucket},
        help_commands,
        macros::{check, command, group, help, hook},
        Args, CommandGroup, CommandOptions, CommandResult, DispatchError, HelpOptions, Reason,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        guild::*,
        id::{GuildId, RoleId, UserId},
        permissions::Permissions,
    },
    prelude::*,
};

struct Handler;

#[group]
#[commands(temp)]
struct Sentinel;

#[group]
#[commands(down)]
struct General;

fn get_roleId(map: HashMap<RoleId, Role>, name: String) -> Option<RoleId> {
    map.iter()
        .find_map(|(key, val)| if val.name == name { Some(*key) } else { None })
}

#[async_trait]
impl EventHandler for Handler {
    // Adds a role when a memeber joins the server
    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        println!("Person has joined");
        let roles = guild_id
            .roles(ctx.http.as_ref())
            .await
            .expect("Expected roles");
        let role = get_roleId(roles, "Friends".to_string()).expect("Couldn't find role");

        println!("{:#?}", role);
        new_member
            .add_role(ctx.http.as_ref(), role)
            .await
            .expect("Adding role didn't work");
    }

    // Prints successfully connected
    async fn ready(&self, _: Context, ready: Ready) {
        println!("connected!");
    }
}

#[tokio::main]
async fn main() {
    // Extract token from env variable
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("<").owners(owners))
        .group(&GENERAL_GROUP)
        .group(&SENTINEL_GROUP);

    // Creates a client and a handler
    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Starts the client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
