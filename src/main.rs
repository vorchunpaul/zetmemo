use teloxide::{payloads::SendMessageSetters, prelude::Requester, Bot};
use tracing::info;
use anyhow;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_name="tg_bot_tock")]
    tg: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // log enable
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(tracing::Level::TRACE)
            .finish()
    )?;
    info!("zetmemo bot starting ...");

    // parce args 
    let conf = Args::parse();

    // init bot    
    let bot = Bot::new(conf.tg);
    Ok(teloxide::repl(bot,save).await)
}

async fn save(bot: Bot, msg: teloxide::types::Message) -> teloxide::prelude::ResponseResult<()> 
{
    let chat_id = msg.chat.id;
    let message_id = msg.id;
    let message_text = if let Some(text) = msg.text() {
        text
    } else {
        info!("no text");
        bot.send_message(chat_id, "plz text only").await?;
        return Ok(())
    };

    info!("new message text: {:#?}", message_text);

    let time = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let message_text = message_text.replace("#", "\\#");
    let resp = format!("`{}`\n{}", time, message_text);


    bot.delete_message(chat_id, message_id).await?;
    bot.send_message(chat_id, resp)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;

    Ok(())
}