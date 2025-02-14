use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process,
};

use libp2p::{
    futures::StreamExt,
    identity::{self},
    request_response,
    swarm::SwarmEvent,
    Multiaddr,
};
use qs::{
    behaviour::{ChatBehaviourEvent, FileRequest, FileResponse},
    shared_state::SharedState,
    swarm::build_swarm,
    varifcation,
};
use tokio::io::AsyncBufReadExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    let mut swarm = build_swarm(local_key)?;
    // 监听本地所有地址和随机端口
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut shared_state = SharedState::default();
    let mut local_addrs = Vec::<Multiaddr>::new();

    loop {
        tokio::select! {
            event = swarm.select_next_some()=>match  event{
                SwarmEvent::NewListenAddr {address,  .. } => {
                    println!("address:{}",address);
                    local_addrs.push(address);
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(event)) => match event {

                    libp2p::mdns::Event::Discovered(vec) => {
                        for (peer_id, _) in vec {
                            shared_state.discovered_perrs.insert(peer_id);
                        }
                    }
                    libp2p::mdns::Event::Expired(vec) => {
                        for (peer_id, _) in vec {
                            println!("Expired peer :{}", peer_id);
                            shared_state.discovered_perrs.remove(&peer_id);
                        }
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::RequestReponse(
                    request_response::Event::Message { message, .. },
                )) => match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        if let Some((path,addrs)) = shared_state.active_passwords.get(&request.password) {
                            println!("开始传输文件");
                            let content = fs::read(path)?;
                            let response = FileResponse { content,addrs:addrs.clone() };
                            swarm
                                .behaviour_mut()
                                .request_reponse
                                .send_response(channel, response)
                                .unwrap();
                            println!("发送成功");
                            //接收到对方接收完成信号后退出程序
                            if let SwarmEvent::Behaviour(ChatBehaviourEvent::Ping(_)) = swarm.select_next_some().await {
                                break;
                            }
                        }
                    }
                    request_response::Message::Response { response, .. } => {
                        fs::write("./received_file", response.content).unwrap();
                        println!("接收成功");
                        //发送接收成功通知
                        for addr in response.addrs{
                        swarm.dial(addr)?;
                        }
                        break;
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::RequestReponse(
                    request_response::Event::OutboundFailure { error, .. },
                )) => {
                    match error{
                        request_response::OutboundFailure::DialFailure => {
                            println!("与发送方连接失败,exiting...");
                            Err(error)?;
                        },
                        request_response::OutboundFailure::Timeout => {
                            println!("连接超时，重新尝试接收");
                            if let  SwarmEvent::Behaviour(ChatBehaviourEvent::RequestReponse(request_response::Event::Message {  message:request_response::Message::Response {  response,.. },.. })) = swarm.select_next_some().await{
                                fs::write("./received_file", response.content).unwrap();
                                println!("接收成功");
                                //发送接收成功通知
                                for addr in response.addrs{
                                swarm.dial(addr)?;
                                }
                                break;
                            }

                        },
                        request_response::OutboundFailure::ConnectionClosed => {
                            println!("对方已关闭连接,exiting...");
                            Err(error)?;
                        },
                        request_response::OutboundFailure::UnsupportedProtocols => {
                            println!("协议不被支持,exiting...");
                            Err(error)?;
                        },
                        request_response::OutboundFailure::Io(error) => {
                            println!("文件传输异常 ,exiting...");
                            Err(error)?;
                        },
                    }
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::RequestReponse(
                    request_response::Event::ResponseSent { .. },
                )) => {}
                _ => {}
            },
            stdin = stdin.next_line() =>  {
                let line =stdin.expect("can not  get line").expect("can not read line from stdin");
                match line.split_whitespace().collect::<Vec<&str>>().as_slice() {
                    ["send", file_path] => {
                        let varifcation_code = varifcation::gen_varif(varifcation::VARIFCATION_LEN);
                        println!("receive {}",varifcation_code);
                        let path_buf=check_file_path(file_path)?;

                        shared_state
                            .active_passwords
                            .insert(varifcation_code, (path_buf,local_addrs.clone()));
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
}
