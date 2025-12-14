use azalea::prelude::*;
use anyhow::Result;
use regex::Regex;
use reqwest::Client as HttpClient;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

#[tokio::main]
async fn main() -> AppExit {
    let config = load_config().unwrap();

    let account = if config.bot.is_offline {
        Account::offline(&config.bot.username)
    } else {
        // 这里可以添加在线登录的逻辑
        Account::offline(&config.bot.username)
    };

    // 解析服务器地址
    let address_parts: Vec<&str> = config.bot.server_address.split(':').collect();
    let host = address_parts[0];
    let port = if address_parts.len() > 1 {
        address_parts[1].parse().unwrap_or(25565)
    } else {
        25565
    };

    ClientBuilder::new()
        .set_handler(handle)
        .start(account, (host, port))
        .await
}

#[derive(Deserialize, Debug, Clone)]
struct Config {
    bot: BotConfig,
    bluemap: BluemapConfig,
}

#[derive(Deserialize, Debug, Clone)]
struct BotConfig {
    username: String,
    server_address: String,
    is_offline: bool,
}

#[derive(Deserialize, Debug, Clone)]
struct BluemapConfig {
    api_url: String,
}

use std::sync::{Arc, Mutex};

#[derive(Clone, Component)]
pub struct State {
    super_ops: Vec<String>,
    ops: Arc<Mutex<Vec<String>>>,
    http_client: HttpClient,
    config: Config,
}

impl Default for State {
    fn default() -> Self {
        let config = load_config().unwrap();
        // 默认超级超管
        let super_ops = vec![
            "NOI_zl".to_string(),
            "Mc＿MintyCool".to_string()
        ];
        
        Self {
            super_ops,
            ops: Arc::new(Mutex::new(Vec::new())),
            http_client: HttpClient::new(),
            config,
        }
    }
}

impl State {
    // 检查是否为超级超管
    fn is_super_op(&self, player: &str) -> bool {
        self.super_ops.contains(&player.to_string())
    }
    
    // 检查是否为超管（包括超级超管）
    fn is_op(&self, player: &str) -> bool {
        let ops = self.ops.lock().unwrap();
        self.is_super_op(player) || ops.contains(&player.to_string())
    }
    
    // 添加超管
    fn add_op(&self, player: &str) {
        let mut ops = self.ops.lock().unwrap();
        if !self.is_super_op(player) && !ops.contains(&player.to_string()) {
            ops.push(player.to_string());
        }
    }
    
    // 移除超管
    fn remove_op(&self, player: &str) {
        let mut ops = self.ops.lock().unwrap();
        if let Some(index) = ops.iter().position(|p| p == player) {
            ops.remove(index);
        }
    }
    
    // 获取ops列表
    fn get_ops(&self) -> Vec<String> {
        let ops = self.ops.lock().unwrap();
        ops.clone()
    }
}

fn load_config() -> Result<Config> {
    let config_path = Path::new("config.toml");
    let config_content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

// 允许中文等非空白字符作为指令名
static COMMAND_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^%([^\s]+)\s*(.*)$").unwrap()
});

async fn handle(bot: Client, event: Event, state: State) -> Result<()> {
    match event {
        Event::Chat(m) => {
            let (sender, content) = m.split_sender_and_content();
            if sender.is_none() {
                return Ok(());
            }

            let sender_name = sender.unwrap();
            if let Some(captures) = COMMAND_REGEX.captures(&content) {
                let command = captures.get(1).unwrap().as_str();
                let args = captures.get(2).unwrap().as_str().trim();

                match command {
                    "开盒" => handle_open_box(&bot, sender_name, args, &state).await?,
                    "tpa" => handle_tpa(&bot, sender_name, args, &state).await?,
                    "设置传送点" => handle_set_home(&bot, sender_name, args, &state).await?,
                    "挖矿" => {
                        if !state.is_op(sender_name) {
                            bot.chat("&#f877f8[&#df8af0樱&#c79ee7花&#aeb1df雪&#95c5d7机&#7cd8cf器&#64ecc6人&#4bffbe] &#47fac5您&#44f5cc暂&#40f0d3无&#3decdb管&#39e7e2理&#36e2e9权&#32ddf0限");
                            return Ok(());
                        }
                        bot.chat("/tpa here");
                        bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 收到指令，正在tpa you请接受");
                    },
                    "op" => handle_op(&bot, sender_name, args, state.clone()).await?,
                    "deop" => handle_deop(&bot, sender_name, args, state.clone()).await?,
                    "op查询" => handle_op_query(&bot, sender_name, &state).await?,
                    "指令" => {
                        bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 可用指令列表：");
                        bot.chat("1. %开盒 [玩家名字] - 查询玩家位置 (管理员)");
                        bot.chat("2. %tpa me - 让机器人tp到你这里 (管理员)");
                        bot.chat("3. %tpa you - 让你tp到机器人那里 (管理员)");
                        bot.chat("4. %挖矿 - 机器人开始挖矿 (管理员)");
                        bot.chat("5. %设置传送点 [名字] - 传送并设置家 (管理员)");
                        bot.chat("6. %op [玩家名字] - 添加管理员 (超级管理员)");
                        bot.chat("7. %deop [玩家名字] - 移除管理员 (超级管理员)");
                        bot.chat("8. %op查询 - 查询管理员列表 (管理员)");
                    },
                    _ => bot.chat(format!("未知命令: {}", command)),
                }
            }
        },
        _ => {}
    }

    Ok(())
}

async fn handle_open_box(bot: &Client, sender: &str, args: &str, state: &State) -> Result<()> {
    if !state.is_op(sender) {
        bot.chat("&#f877f8[&#df8af0樱&#c79ee7花&#aeb1df雪&#95c5d7机&#7cd8cf器&#64ecc6人&#4bffbe] &#47fac5您&#44f5cc暂&#40f0d3无&#3decdb管&#39e7e2理&#36e2e9权&#32ddf0限");
        return Ok(());
    }

    if args.is_empty() {
        bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 请输入玩家名字，格式: %开盒 [名字]");
        return Ok(());
    }

    let player_name = args;
    // 调用BlueMap API获取玩家位置
    match get_player_position(state, player_name).await {
        Ok((x, y, z)) => {
            bot.chat(format!("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] {}玩家目前位置在 {} {} {}", player_name, x, y, z));
        },
        Err(e) => {
            bot.chat(format!("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 获取玩家位置失败: {:?}", e));
        }
    }
    Ok(())
}

async fn get_player_position(state: &State, player_name: &str) -> Result<(f64, f64, f64)> {
    let url = format!("{}/players/{}", state.config.bluemap.api_url, player_name);
    
    // 添加重试机制
    let mut retries = 3;
    while retries > 0 {
        match state.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let json: serde_json::Value = response.json().await?;
                    if let (Some(x), Some(y), Some(z)) = (
                        json["position"]["x"].as_f64(),
                        json["position"]["y"].as_f64(),
                        json["position"]["z"].as_f64()
                    ) {
                        return Ok((x, y, z));
                    } else {
                        return Err(anyhow::anyhow!("Invalid position data in response"));
                    }
                } else {
                    retries -= 1;
                    if retries == 0 {
                        return Err(anyhow::anyhow!("HTTP request failed with status: {}", response.status()));
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            },
            Err(e) => {
                retries -= 1;
                if retries == 0 {
                    return Err(anyhow::anyhow!("HTTP request error: {}", e));
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
    
    Err(anyhow::anyhow!("Failed to get player position after retries"))
}

async fn handle_tpa(bot: &Client, sender: &str, args: &str, state: &State) -> Result<()> {
    if !state.is_op(sender) {
        bot.chat("&#f877f8[&#df8af0樱&#c79ee7花&#aeb1df雪&#95c5d7机&#7cd8cf器&#64ecc6人&#4bffbe] &#47fac5您&#44f5cc暂&#40f0d3无&#3decdb管&#39e7e2理&#36e2e9权&#32ddf0限");
        return Ok(());
    }

    match args {
        "me" => {
            bot.chat("/tpa here");
            bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 收到指令，正在tpa you请接受");
        },
        "you" => {
            bot.chat("/tpa");
            bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 收到指令，正在tpa me请接受");
        },
        _ => {
            bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 用法: %tpa me 或 %tpa you");
        }
    }
    Ok(())
}

async fn handle_set_home(bot: &Client, sender: &str, args: &str, state: &State) -> Result<()> {
    if !state.is_op(sender) {
        bot.chat("&#f877f8[&#df8af0樱&#c79ee7花&#aeb1df雪&#95c5d7机&#7cd8cf器&#64ecc6人&#4bffbe] &#47fac5您&#44f5cc暂&#40f0d3无&#3decdb管&#39e7e2理&#36e2e9权&#32ddf0限");
        return Ok(());
    }

    if args.is_empty() {
        bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 请输入传送点名字，格式: %设置传送点 [名字]");
        return Ok(());
    }

    let home_name = args;
    bot.chat(format!("/sethome {}", home_name));
    bot.chat(format!("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 已设置传送点: {}", home_name));
    
    // 等待5秒后传送
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    bot.chat(format!("/home {}", home_name));
    bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 正在传送...");
    
    Ok(())
}

async fn handle_op(bot: &Client, sender: &str, args: &str, state: State) -> Result<()> {
    if !state.is_super_op(sender) {
        bot.chat("&#f877f8[&#df8af0樱&#c79ee7花&#aeb1df雪&#95c5d7机&#7cd8cf器&#64ecc6人&#4bffbe] &#47fac5您&#44f5cc暂&#40f0d3无&#3decdb管&#39e7e2理&#36e2e9权&#32ddf0限");
        return Ok(());
    }

    if args.is_empty() {
        bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 请输入玩家名字，格式: %op [名字]");
        return Ok(());
    }

    let player_name = args;
    state.add_op(player_name);
    bot.chat(format!("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 已添加管理员: {}", player_name));
    
    Ok(())
}

async fn handle_deop(bot: &Client, sender: &str, args: &str, state: State) -> Result<()> {
    if !state.is_super_op(sender) {
        bot.chat("&#f877f8[&#df8af0樱&#c79ee7花&#aeb1df雪&#95c5d7机&#7cd8cf器&#64ecc6人&#4bffbe] &#47fac5您&#44f5cc暂&#40f0d3无&#3decdb管&#39e7e2理&#36e2e9权&#32ddf0限");
        return Ok(());
    }

    if args.is_empty() {
        bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 请输入玩家名字，格式: %deop [名字]");
        return Ok(());
    }

    let player_name = args;
    state.remove_op(player_name);
    bot.chat(format!("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 已移除管理员: {}", player_name));
    
    Ok(())
}

async fn handle_op_query(bot: &Client, sender: &str, state: &State) -> Result<()> {
    if !state.is_op(sender) {
        bot.chat("&#f877f8[&#df8af0樱&#c79ee7花&#aeb1df雪&#95c5d7机&#7cd8cf器&#64ecc6人&#4bffbe] &#47fac5您&#44f5cc暂&#40f0d3无&#3decdb管&#39e7e2理&#36e2e9权&#32ddf0限");
        return Ok(());
    }

    let ops = state.get_ops();
    let super_ops = &state.super_ops;
    
    bot.chat("&#f877f8[&#e487f1樱&#cf97ea花&#bba7e4雪&#a7b7dd机&#92c7d6器&#7ed7cf人&#6ae7c8] 管理员列表：");
    bot.chat("超级管理员：");
    for op in super_ops {
        bot.chat(format!("- {}", op));
    }
    
    if !ops.is_empty() {
        bot.chat("普通管理员：");
        for op in ops {
            bot.chat(format!("- {}", op));
        }
    } else {
        bot.chat("暂无普通管理员");
    }
    
    Ok(())
}