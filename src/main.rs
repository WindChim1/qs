use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process,
};

use libp2p::{
    futures::StreamExt,
    identity::{self, Keypair},
    noise, request_response,
    swarm::SwarmEvent,
    yamux, Swarm, SwarmBuilder,
};
use qs::{
    behaviour::{ChatBehaviour, ChatBehaviourEvent, FileRequest, FileResponse},
    SharedState,
};
use rand::Rng;
use tokio::io::AsyncBufReadExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    println!("local peer_id {}", local_peer_id);
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    let mut swarm = build_swarm(local_key)?;
    // 监听本地所有地址和随机端口
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut shared_state = SharedState::default();

    loop {
        tokio::select! {
            event = swarm.select_next_some()=>match  event{
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("address   {}", address);
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(event)) => match event {
                    libp2p::mdns::Event::Discovered(vec) => {
                        for (peer_id, _) in vec {
                            println!("Discovered peer  :{}", peer_id);
                            shared_state.discovered_perrs.insert(peer_id);
                        }
                    }
                    libp2p::mdns::Event::Expired(vec) => {
                        for (peer_id, _) in vec {
                            println!("Expired peer :{}", peer_id);
                            shared_state.discovered_perrs.insert(peer_id);
                        }
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::RequestReponse(
                    request_response::Event::Message { message, .. },
                )) => match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        if let Some(path) = shared_state.active_passwords.get(&request.password) {
                            println!("开始传输文件");
                            let content = fs::read(path)?;
                            let response = FileResponse { content };
                            swarm
                                .behaviour_mut()
                                .request_reponse
                                .send_response(channel, response)
                                .unwrap();
                            println!("发送成功");
                            break;
                        }
                    }
                    request_response::Message::Response { response, .. } => {
                        fs::write("./received_file", response.content).unwrap();
                        println!("接收成功");
                        break;
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::RequestReponse(
                    request_response::Event::OutboundFailure { error, .. },
                )) => {
                    println!("接收失败:{:?}", error)
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::RequestReponse(
                    request_response::Event::ResponseSent { .. },
                )) => {}
                _ => {}
            },
            stdin = stdin.next_line() =>  {
                let line =stdin.expect("can not  get line").expect("can not read line from stdin");
                match line.split_whitespace().collect::<Vec<&str>>().as_slice() {
                    // ["send","c", content] => {
                    //     let password: u32 = rand::rng().random();
                    //     println!("password {}",password);
                    //     shared_state
                    //         .active_passwords
                    //         .insert(password.to_string(), content.to_string());
                    //     // let content = read_file(file_path)?;
                    // }
                    ["send", file_path] => {
                        let password: u32 = rand::rng().random();
                        println!("password {}",password);
                        let path_buf=check_file_path(file_path)?;

                        shared_state
                            .active_passwords
                            .insert(password.to_string(), path_buf);
                    }
                    ["receive", password] => {
                        for perr_id  in &shared_state.discovered_perrs {
                            let request = FileRequest::new(password.to_string());
                            swarm.behaviour_mut().request_reponse.send_request(
                                    perr_id,
                                request,
                            );
                        }


                    }
                    _ => {}
                }
                println!("{line}")
            },
            _ = ctrl_c()=>{
                process::exit(0)
            }
        };
    }
    Ok(())
}

async fn ctrl_c() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler")
}

fn check_file_path(file_path: &str) -> Result<PathBuf, Box<dyn Error>> {
    // 解析文件路径
    let file_path = Path::new(file_path);
    let absolute_path = if file_path.is_relative() {
        // 将相对路径转换为绝对路径
        std::env::current_dir()?.join(file_path)
    } else {
        file_path.to_path_buf()
    };

    // 检查文件是否存在
    if !absolute_path.exists() {
        println!("File not found: {:?}", absolute_path);
        Err(format!("文件不存在:{:?}", absolute_path).into())
    } else {
        Ok(absolute_path)
    }

    // 读取文件内容
    // let content = fs::read(&absolute_path)?;
    // println!("Sending file: {:?}", absolute_path);
    // Ok(content)
}

fn build_swarm(local_key: Keypair) -> Result<Swarm<ChatBehaviour>, Box<dyn Error>> {
    let swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            Default::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|b| ChatBehaviour::new(local_key).unwrap())?
        .build();

    Ok(swarm)
}
