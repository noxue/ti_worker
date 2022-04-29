use rand::Rng;
use ti_protocol::{
    get_header_size, PackType, Packet, PacketHeader, Task, TaskResult, TiPack, TiUnPack,
};
use tokio::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    let len: usize = get_header_size();

    let mut ts = vec![];

    for _ in 0..3 {
        ts.push(tokio::spawn(async move {
            // 连接服务端
            let mut stream = TcpStream::connect("127.0.0.1:8000").await.unwrap();
            loop {
                // 获取任务
                let packet = Packet::new_without_data(PackType::GetTask);
                let packet = packet.pack().unwrap();
                stream.write(&packet).await.unwrap();
                stream.flush().await.unwrap();

                // 获取任务返回
                let mut header = vec![0; len];
                stream.read(&mut header).await.unwrap();
                let header = PacketHeader::unpack(&header).unwrap();

                // 检查标志位，不对就跳过
                if !header.check_flag() {
                    continue;
                }

                match header.pack_type {
                    // 处理返回的任务
                    PackType::Task => {
                        // 根据包头长度读取数据
                        let mut body = vec![0; header.body_size as usize];
                        stream.read(&mut body).await.unwrap();

                        // 解包
                        let task = Task::unpack(&body).unwrap();

                        // 如果没有任务，就等待一段时间
                        if !task.has_task() {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            continue;
                        }

                        println!("获取到任务：{:?}", task);
                        // 执行任务
                        //  暂时不实现

                        // 随机睡眠一段时间
                        // let sleep_time = rand::thread_rng().gen_range(1..5);
                        // println!("睡眠 {} 秒", sleep_time);
                        // tokio::time::sleep(Duration::from_millis(100)).await;

                        // 创建 TaskResult
                        let task_result = TaskResult::new(task.task_id, Ok(task.task_id + 1));
                        // 创建Packet
                        let packet = Packet::new(PackType::TaskResult, task_result).unwrap();
                        // 将packet序列化
                        let packet = packet.pack().unwrap();
                        // 发送
                        stream.write(&packet).await.unwrap();
                        stream.flush().await.unwrap();
                    }
                    _ => {}
                }

                // thread::sleep(Duration::from_millis(500));
            }
        }));
    }

    for v in ts {
        v.await.unwrap();
    }
}
