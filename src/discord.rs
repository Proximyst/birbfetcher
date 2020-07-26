// birbfetcher - Collect bird images with ease.
// Copyright (C) 2020 Mariell Hoversholm
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::prelude::*;
use once_cell::sync::Lazy;
use serenity::framework::standard::{
    help_commands,
    macros::{command, group, help},
    Args, CommandGroup, CommandResult, HelpOptions,
};
use serenity::model::{
    channel::{Message, Reaction, ReactionType},
    id::{EmojiId, MessageId, UserId},
    misc::EmojiIdentifier,
};
use serenity::prelude::*;
use std::collections::{HashMap, HashSet};

static REACTIONS: Lazy<ReactionIds> = Lazy::new(|| {
    use std::env::var;
    let verify = var("DISCORD_REACTION_VERIFY")
        .expect("`DISCORD_REACTION_VERIFY`")
        .parse()
        .expect("`DISCORD_REACTION_VERIFY`");
    let ban = var("DISCORD_REACTION_BAN")
        .expect("`DISCORD_REACTION_BAN`")
        .parse()
        .expect("`DISCORD_REACTION_BAN`");
    ReactionIds {
        verify,
        ban,
        verify_id: EmojiIdentifier {
            name: var("DISCORD_REACTION_VERIFY_NAME").expect("`DISCORD_REACTION_VERIFY_NAME`"),
            id: EmojiId(verify),
        },
        ban_id: EmojiIdentifier {
            name: var("DISCORD_REACTION_BAN_NAME").expect("`DISCORD_REACTION_BAN_NAME`"),
            id: EmojiId(ban),
        },
    }
});

pub struct Handler;

impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if *reaction.user_id.as_u64() != 181470050039889920 {
            return;
        }

        let mut data = ctx.data.write();
        let map = data
            .get_mut::<ImagesContainer>()
            .expect("images map must exist");
        let img = match map.remove(&reaction.message_id) {
            None => return,
            Some(id) => id,
        };
        let db = data
            .get::<DatabaseContainer>()
            .expect("database must exist");

        match reaction.emoji {
            ReactionType::Custom { id, .. } if *id.as_u64() == REACTIONS.ban => {
                let err = sqlx::query("UPDATE birbs SET banned = true WHERE id = ?")
                    .bind(img)
                    .execute(db);
                let err = futures::executor::block_on(err);

                if let Err(e) = err {
                    if let Err(e) = reaction
                        .channel_id
                        .say(&ctx.http, format!("Could not ban ID {}: {:?}", img, e))
                    {
                        warn!("Could not send message: {:?}", e);
                    }
                } else if let Err(e) = reaction
                    .channel_id
                    .say(&ctx.http, &format!("Banned ID {}", img))
                {
                    warn!("Could not send message: {:?}", e);
                }
            }

            ReactionType::Custom { id, .. } if *id.as_u64() == REACTIONS.verify => {
                let err = sqlx::query("UPDATE birbs SET verified = true WHERE id = ?")
                    .bind(img)
                    .execute(db);
                let err = futures::executor::block_on(err);

                if let Err(e) = err {
                    if let Err(e) = reaction
                        .channel_id
                        .say(&ctx.http, format!("Could not verify ID {}: {:?}", img, e))
                    {
                        warn!("Could not send message: {:?}", e);
                    }
                } else if let Err(e) = reaction
                    .channel_id
                    .say(&ctx.http, &format!("Verified ID {}", img))
                {
                    warn!("Could not send message: {:?}", e);
                }
            }

            _ => (),
        }
    }
}

#[derive(Clone)]
struct ReactionIds {
    verify: u64,
    ban: u64,
    verify_id: EmojiIdentifier,
    ban_id: EmojiIdentifier,
}

pub struct DatabaseContainer;

impl TypeMapKey for DatabaseContainer {
    type Value = MySqlPool;
}

pub struct ImagesContainer;

impl TypeMapKey for ImagesContainer {
    type Value = HashMap<MessageId, u32>;
}

#[group]
#[owners_only]
#[commands(ban, verify, image)]
pub struct Owner;

#[help]
#[command_not_found_text = "No such command: `{}`."]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Strike"]
#[lacking_role = "Strike"]
#[wrong_channel = "Strike"]
pub fn help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

#[command]
pub fn ban(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let id = args.single::<u32>()?;
    let data = ctx.data.read();
    let db = data
        .get::<DatabaseContainer>()
        .expect("database must exist");

    let err = sqlx::query("UPDATE birbs SET banned = true WHERE id = ?")
        .bind(id)
        .execute(db);
    let err = futures::executor::block_on(err);

    if let Err(e) = err {
        if let Err(e) = msg
            .channel_id
            .say(&ctx.http, format!("Could not ban ID {}: {:?}", id, e))
        {
            warn!("Could not send message: {:?}", e);
        }
    } else if let Err(e) = msg.channel_id.say(&ctx.http, &format!("Banned ID {}", id)) {
        warn!("Could not send message: {:?}", e);
    }

    Ok(())
}

#[command]
pub fn verify(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let id = args.single::<u32>()?;
    let data = ctx.data.read();
    let db = data
        .get::<DatabaseContainer>()
        .expect("database must exist");

    let err = sqlx::query("UPDATE birbs SET verified = true WHERE id = ?")
        .bind(id)
        .execute(db);
    let err = futures::executor::block_on(err);

    if let Err(e) = err {
        if let Err(e) = msg
            .channel_id
            .say(&ctx.http, format!("Could not verify ID {}: {:?}", id, e))
        {
            warn!("Could not send message: {:?}", e);
        }
    } else if let Err(e) = msg
        .channel_id
        .say(&ctx.http, &format!("Verified ID {}", id))
    {
        warn!("Could not send message: {:?}", e);
    }

    Ok(())
}

#[command]
pub fn image(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let db = data
        .get::<DatabaseContainer>()
        .expect("database must exist");

    let res = sqlx::query_as(
        "SELECT id FROM birbs WHERE banned = false AND verified = false ORDER BY RAND() LIMIT 1",
    )
    .fetch_one(db);
    let res = futures::executor::block_on(res);
    let (imgid,): (u32,) = match res {
        Err(e) => {
            if let Err(e) = msg
                .channel_id
                .say(&ctx.http, format!("Could not fetch an image: {:?}", e))
            {
                warn!("Could not send message: {:?}", e);
            }
            return Ok(());
        }
        Ok(id) => id,
    };

    let id = match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.image(format!("https://birb.proximy.st/id/{}", imgid));
            e.field("ID", format!("{}", imgid), true);
            e
        });
        m.reactions(vec![
            REACTIONS.verify_id.clone(),
            REACTIONS.ban_id.clone(),
        ]);

        m
    }) {
        Err(e) => {
            warn!("Could not send message: {:?}", e);
            return Ok(());
        }
        Ok(id) => id,
    };

    // We need a mutable borrow now.
    drop(db);

    let map = data
        .get_mut::<ImagesContainer>()
        .expect("images map must exist");
    map.insert(id.id, imgid);

    Ok(())
}
