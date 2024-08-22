use std::sync::Arc;

use serde_json::{json, Value};
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

    oai_endpoint: String,
    oai_token: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // enable log 
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(tracing::Level::INFO)
            .finish()
    )?;
    info!("zetmemo bot starting ...");

    // parce args 
    let conf = Arc::new(Args::parse());

    // init storege adapter
    let store = Arc::new(object_store::aws::AmazonS3Builder::new()
        .with_bucket_name(conf.s3_bucket.clone())
        .with_access_key_id(conf.s3_key_id.clone())
        .with_secret_access_key(conf.s3_access_key.clone())
        .with_endpoint(conf.s3_endpoint.clone())
        .with_allow_http(true)
        .build()?
    );
    
    // init bot api
    let bot = Bot::new(conf.tg.clone());
    
    // init bot handler
    let handler = dptree::entry()
        .inspect(|u: Update| debug!("{u:?}"))
        .branch(Update::filter_message().endpoint(save));
    
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![store, conf])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Err(anyhow::Error::msg("bot close"))
}

fn html_safe(input: &str) -> String
{
    input
        .replace("<", "&lt")
        .replace(">", "&gt")
        .replace("&", "&amp")
}


async fn oai_query(url: String, token: String, system: String, user: String) -> String
{
    let rq = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "messages": [
            {
                "role": "system",
                "content": system.clone()
            },
            {
                "role": "user",
                "content": user.clone()
            }]
        }));
    
    let result = rq.send().await;
    let mut data = result.unwrap().json::<Value>().await.unwrap();
    let ans = data["answer"].take().to_string().replace("\"", "");
    return ans;
}

fn tags_parse(input: String) -> Vec<String> {
    let mut array: Vec<String> = vec![];
    let tags = input.split(",");
    
    for tag in tags {
        let tag = tag.trim();
        let tag = tag.replace(" ", "_");
        let tag = tag.replace(".", "");
        
        array.push(
            format!("#{tag}")
        );
    }

    return array;
}

async fn save(bot: Bot, msg: teloxide::types::Message, store: Arc<AmazonS3>, args: Arc<Args>) -> teloxide::prelude::ResponseResult<()> 
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

    let tldr = oai_query(
        args.oai_endpoint.clone(),
        args.oai_token.clone(),
        "Напиши очень краткий заголовок заметки от лица автора не более 80 символов не используя двойные кавычки".to_string(),
        message_text.to_string()
    ).await;

    let tags = oai_query(
        args.oai_endpoint.clone(),
        args.oai_token.clone(),
        "Составь список тегов описывающие ключевые элементы текста. Теги должны быть разделенными запятыми. Важно не используй \" и \'. Невкоем случае не добавляй #. Невкоем случае не добавляй пробелы в тег, используй вместо него нижнее подчеркивание".to_string(),
        message_text.to_string()
    ).await;
    let tags = tags_parse(tags).join(" ");

    let resp = format!(
        "<b>{}</b>\n\n{}\n\n{}",
        tldr,
        html_safe(message_text),
        tags
    );

    let stat = bot.send_message(chat_id, &resp)
        .parse_mode(teloxide::types::ParseMode::Html)
        //.reply_parameters(ReplyParameters::new(message_id))
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
    //attr.insert(Attribute::ContentType, "text/plain".into()).unwrap();
    
    let err = store.put_opts(&path, payload, PutOptions::from(attr)).await;
    if let Err(e) = err {
        error!("save file error, {e}");
        bot.send_message(chat_id, "file not save ☹️").await?;
    }

    let name = msg.chat.username().unwrap();    
    info!("memnote @{}:\n{}\n{}\n{}", name, tldr, message_text, tags);

    Ok(())
}