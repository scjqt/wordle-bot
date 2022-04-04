use std::{env, sync::Arc, time::Duration};

use serenity::{
    async_trait,
    builder::{CreateComponents, CreateEmbed},
    futures::stream::StreamExt,
    http::Http,
    model::{interactions::message_component::ButtonStyle, prelude::*},
    prelude::*,
    utils,
};
use wordle_bot::{pattern_builder::PatternBuilder, wordle::*};

const GREEN_SQUARE: &str = "🟩";
const YELLOW_SQUARE: &str = "🟨";
const BLACK_SQUARE: &str = "⬛";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let http = &ctx.http;
        if msg.mentions_me(http).await.unwrap() {
            let mut wordle = Wordle::new();
            let mut pattern = PatternBuilder::new();
            let mut history: Vec<(String, String, bool)> = Vec::new();

            let mut reply = msg
                .channel_id
                .send_message(http, |m| {
                    m.embed(|e| guess(e, &wordle, &pattern, &history))
                        .components(|c| create_buttons(c, 0))
                })
                .await
                .unwrap();

            let mut builder = reply
                .await_component_interactions(&ctx)
                .timeout(Duration::from_secs(300))
                .await;

            while let Some(interaction) = builder.next().await {
                interaction
                    .create_interaction_response(http, |r| {
                        r.kind(InteractionResponseType::UpdateMessage)
                    })
                    .await
                    .unwrap();

                if interaction.user.id == msg.author.id {
                    let button = interaction.as_ref().data.custom_id.as_str();

                    if pattern.count() < 5 {
                        if match button {
                            "Black" => pattern.append(Colour::Black),
                            "Yellow" => pattern.append(Colour::Yellow),
                            "Green" => pattern.append(Colour::Green),
                            "Back" => pattern.remove(),
                            _ => false,
                        } {
                            edit(
                                &mut reply,
                                http,
                                |e| guess(e, &wordle, &pattern, &history),
                                pattern.count(),
                            )
                            .await;
                        }
                    } else {
                        match button {
                            "Back" => {
                                pattern.remove();
                                edit(
                                    &mut reply,
                                    http,
                                    |e| guess(e, &wordle, &pattern, &history),
                                    pattern.count(),
                                )
                                .await;
                            }
                            "Confirm" => {
                                history.push((
                                    format!(
                                        "Guess {}:    {}",
                                        history.len() + 1,
                                        &wordle.guess().to_string()
                                    ),
                                    pattern_to_string(&pattern),
                                    false,
                                ));
                                wordle.update(pattern.get_pattern().unwrap());
                                pattern.clear();
                                match wordle.options() {
                                    1 => {
                                        edit(
                                            &mut reply,
                                            http,
                                            |e| {
                                                title(footer(e)).fields(history.clone()).field(
                                                    format!(
                                                        "The word is {}",
                                                        wordle.guess().to_string()
                                                    ),
                                                    "Solved",
                                                    false,
                                                )
                                            },
                                            0,
                                        )
                                        .await;
                                        break;
                                    }
                                    0 => {
                                        edit(
                                            &mut reply,
                                            http,
                                            |e| {
                                                title(footer(e)).fields(history.clone()).field(
                                                    "No options remain",
                                                    "Unsolvable",
                                                    false,
                                                )
                                            },
                                            0,
                                        )
                                        .await;
                                        break;
                                    }
                                    _ => {
                                        edit(
                                            &mut reply,
                                            http,
                                            |e| guess(e, &wordle, &pattern, &history),
                                            pattern.count(),
                                        )
                                        .await;
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }

            if wordle.options() > 1 {
                reply
                    .edit(http, |m| {
                        m.embed(|e| {
                            title(footer(e))
                                .fields(history)
                                .field("Timed out.", "-", false)
                        })
                    })
                    .await
                    .unwrap();
            }
            reply.edit(http, |m| m.components(|c| c)).await.unwrap();
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn edit(
    msg: &mut Message,
    http: &Arc<Http>,
    f: impl FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    count: usize,
) {
    msg.edit(http, |m| {
        m.embed(f).components(|c| create_buttons(c, count))
    })
    .await
    .unwrap();
}

fn title(e: &mut CreateEmbed) -> &mut CreateEmbed {
    e.title("<--- Wordle Solver --->")
        .color(utils::Colour::LIGHT_GREY)
}

fn footer(e: &mut CreateEmbed) -> &mut CreateEmbed {
    e.footer(|f| {
        f.text("Created by Dis#3386")
            .icon_url("https://i.imgur.com/DbQTOdc.png")
    })
}

fn guess<'a>(
    e: &'a mut CreateEmbed,
    wordle: &Wordle,
    pattern: &PatternBuilder,
    history: &Vec<(String, String, bool)>,
) -> &'a mut CreateEmbed {
    title(e).fields(history.clone()).field(
        format!("Guess     {}", &wordle.guess().to_string()),
        format!("Pattern: {}", &pattern_to_string(pattern)),
        false,
    )
}

fn create_buttons(c: &mut CreateComponents, count: usize) -> &mut CreateComponents {
    let enabled = match count {
        0 => [true, true, true, false, false],
        5 => [false, false, false, true, true],
        _ => [true, true, true, true, false],
    };
    c.create_action_row(|r| {
        r.create_button(|b| {
            b.custom_id("Black")
                .style(ButtonStyle::Secondary)
                .emoji(str_to_reaction(BLACK_SQUARE))
                .disabled(!enabled[0])
        })
        .create_button(|b| {
            b.custom_id("Yellow")
                .style(ButtonStyle::Secondary)
                .emoji(str_to_reaction(YELLOW_SQUARE))
                .disabled(!enabled[1])
        })
        .create_button(|b| {
            b.custom_id("Green")
                .style(ButtonStyle::Secondary)
                .emoji(str_to_reaction(GREEN_SQUARE))
                .disabled(!enabled[2])
        })
        .create_button(|b| {
            b.custom_id("Back")
                .style(ButtonStyle::Danger)
                .label("Back")
                .disabled(!enabled[3])
        })
        .create_button(|b| {
            b.custom_id("Confirm")
                .style(ButtonStyle::Success)
                .label("Confirm")
                .disabled(!enabled[4])
        })
    })
}

fn str_to_reaction(unicode: &str) -> ReactionType {
    ReactionType::Unicode(String::from(unicode))
}

fn pattern_to_string(pattern: &PatternBuilder) -> String {
    let mut s = String::new();
    for &colour in pattern.get() {
        s.push_str(match colour {
            Colour::Black => BLACK_SQUARE,
            Colour::Yellow => YELLOW_SQUARE,
            Colour::Green => GREEN_SQUARE,
        });
    }
    s
}

#[tokio::main]
async fn main() {
    println!("success 1");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    println!("success 2");

    let id = env::var("APPLICATION_ID")
        .expect("Expected an application ID in the environment")
        .parse()
        .unwrap();

    println!("success 3");

    let mut client = Client::builder(&token)
        .application_id(id)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    println!("success 4");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    println!("success 5");

}
