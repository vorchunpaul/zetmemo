use std::sync::Arc;

use teloxide::{dispatching::UpdateFilterExt, dptree, payloads::SendMessageSetters, prelude::{Dispatcher, Requester}, types::{ReplyParameters, Update}, Bot};
use tracing::{debug, error, info};
use anyhow;
use clap::Parser;
use object_store::{self, aws::AmazonS3, path::Path, Attribute, Attributes, ObjectStore, PutOptions, PutPayload};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_name="tg_bot_tock")]
    tg: String,

    s3_bucket: String,
    s3_key_id: String,
    s3_access_key: String,
    s3_endpoint: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // enable log 
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(tracing::Level::TRACE)
            .finish()
    )?;
    info!("zetmemo bot starting ...");

    // parce args 
    let conf = Args::parse();

    // init storege adapter
    let store = Arc::new(object_store::aws::AmazonS3Builder::new()
        .with_bucket_name(conf.s3_bucket)
        .with_access_key_id(conf.s3_key_id)
        .with_secret_access_key(conf.s3_access_key)
        .with_endpoint(conf.s3_endpoint)
        .with_allow_http(true)
        .build()?
    );
    
    // init bot api
    let bot = Bot::new(conf.tg);
    
    // init bot handler
    let handler = dptree::entry()
        .inspect(|u: Update| debug!("{u:?}"))
        .branch(Update::filter_message().endpoint(save));
    
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![store])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Err(anyhow::Error::msg("bot close"))
}

fn html_save(input: &str) -> String
{
    input
        .replace("<", "&lt")
        .replace(">", "&gt")
        .replace("&", "&amp")
}

async fn save(bot: Bot, msg: teloxide::types::Message, store: Arc<AmazonS3>) -> teloxide::prelude::ResponseResult<()> 
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
    let time = chrono::Utc::now();

    let resp = format!(
        "<code>{}</code>\n<b>SUPER TEST</b>\n\n{}\n\n#test #test #test",
        time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true), 
        html_save(message_text)
    );

    let stat = bot.send_message(chat_id, &resp)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_parameters(ReplyParameters::new(message_id))
        .await;

    if let Err(e) = stat {
        error!("msg send error: {}", e);
        bot.send_message(chat_id, "fuck you! adun fish").await?;
    }

    // save memnote to store
    let memnote_name = format!("/{}/{}.md", 
        chat_id,
        time.format("%Y%m%d%H%M%S%.3fZ")
    );
    let path = Path::from(memnote_name);
    let payload = PutPayload::from(resp);

    let mut attr = Attributes::new();
    attr.insert(Attribute::ContentType, "text/plain".into()).unwrap();
    
    let err = store.put_opts(&path, payload, PutOptions::from(attr)).await;
    if let Err(e) = err {
        error!("save file error, {e}");
        bot.send_message(chat_id, "file not save ☹️").await?;
    }
    Ok(())
}