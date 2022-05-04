use std::env;

use log::{debug, error, info};
use rand::Rng;
use ti_protocol::{
    get_header_size, PackType, Packet, PacketHeader, Task, TaskResult, TaskResultError, TiPack,
    TiUnPack,
};

use ti_worker::worker::Worker;
use tokio::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let server = args.get(1).expect("请提供服务器地址和端口");

    log4rs::init_file("log.yml", Default::default()).unwrap();

    let len: usize = get_header_size();

    loop {
        let mut ts = vec![];

        for _ in 0..3 {
            let server = server.clone();
            ts.push(tokio::spawn(async move {
                // 连接服务端
                let mut stream = match TcpStream::connect(server).await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("服务器断开连接: {}", e);
                        return;
                    }
                };
                let worker = Worker::new();

                loop {
                    // 获取任务
                    let packet = Packet::new_without_data(PackType::GetTask);
                    let packet = packet.pack().unwrap();
                    if let Err(e) = stream.write(&packet).await {
                        error!("发送任务失败: {}", e);
                        return;
                    }
                    if let Err(e) = stream.flush().await {
                        error!("发送任务失败: {}", e);
                        return;
                    }

                    // 获取任务返回
                    let mut header = vec![0; len];
                    if let Err(e) = stream.read(&mut header).await {
                        error!("接收任务失败: {}", e);
                        return;
                    }
                    let header = PacketHeader::unpack(&header).unwrap();

                    // 检查标志位，不对就跳过
                    if !header.check_flag() {
                        continue;
                    }

                    // 表示出错，下面根据他来判断是否休眠一段时间
                    let mut is_err = false;

                    match header.pack_type {
                        // 处理返回的任务
                        PackType::Task => {
                            // 根据包头长度读取数据
                            let mut body = vec![0; header.body_size as usize];
                            if let Err(e) = stream.read(&mut body).await {
                                error!("接收任务失败: {}", e);
                                return;
                            }

                            // 解包
                            let task = Task::unpack(&body).unwrap();

                            // 如果没有任务，就等待一段时间
                            if !task.has_task() {
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                continue;
                            }

                            debug!("获取到任务：{:?}", task);

                            // 执行任务
                            let res = worker
                                .get_store_by_product_name(task.product_name.as_str())
                                .await;

                            // 封装返回结果
                            let task_result = match res {
                                Ok(v) => TaskResult::new(task.task_id, Ok(v as i32)),
                                Err(e) => {
                                    info!("执行任务出错：{}", e);

                                    let err_type = if e.contains("operation timed out") {
                                        TaskResultError::Timeout
                                    } else if e.contains("403 Forbidden") {
                                        is_err = true; // 如果请求太频繁，就停止一段时间
                                        TaskResultError::Banned
                                    } else if e.contains("No Content") {
                                        TaskResultError::ProductNotFound
                                    } else {
                                        TaskResultError::AccessDenied
                                    };

                                    TaskResult::new(task.task_id, Err(err_type))
                                }
                            };

                            // 创建Packet
                            let packet = Packet::new(PackType::TaskResult, task_result).unwrap();
                            // 将packet序列化
                            let packet = packet.pack().unwrap();

                            // 发送
                            if let Err(e) = stream.write(&packet).await {
                                error!("发送任务失败: {}", e);
                                return;
                            }
                            if let Err(e) = stream.flush().await {
                                error!("发送任务失败: {}", e);
                                return;
                            }
                        }
                        _ => {}
                    }

                    if is_err {
                        // 随机数1-3秒

                        let sleep_time = {
                            let mut rng = rand::thread_rng();
                            rng.gen_range(60 * 2..60 * 5)
                        };
                        info!("休眠{}秒", sleep_time);
                        tokio::time::sleep(Duration::from_secs(sleep_time)).await;
                    }
                }
            }));
        }

        for v in ts {
            v.await.unwrap();
        }
    }
}
