use rand::Rng;

use ti_protocol::{PackType, Packet, Task, TaskResult, TiPack};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[tokio::main]
async fn main() {
    // 连接服务端
    let mut stream = TcpStream::connect("192.168.1.6:8000").await.unwrap();
    // loop {
    let mut rng = rand::thread_rng();
    let x: i32 = rng.gen_range(10..1000);
    let y: i32 = rng.gen_range(10..1000);

    // 创建Task数据包
    let task = Task::new(x, format!("{}+{}", x, y));
    // 创建Packet
    let packet = Packet::new(PackType::Task, task).unwrap();

    // 将packet序列化
    let packet = packet.pack().unwrap();

    // 发送
    stream.write(&packet).await.unwrap();

    // 创建 TaskResult
    let task_result = TaskResult::new(x, Ok(x + y));
    // 创建Packet
    let packet = Packet::new(PackType::TaskResult, task_result).unwrap();
    // 将packet序列化
    let packet = packet.pack().unwrap();
    // 发送
    stream.write(&packet).await.unwrap();
    stream.flush().await.unwrap();
    // }

    // // 发送数据
    // stream.write(b"hello world!").unwrap();
    // // 接收数据
    // let mut buf = [0; 1024];
    // stream.read(&mut buf).unwrap();
    // println!("{}", String::from_utf8_lossy(&buf));
}
