use rand::Rng;
use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};
use ti_protocol::{get_header_size, PackType, Packet, Task, TaskResult, TiPack, TiUnPack};

fn main() {
    // 连接服务端
    let mut stream = TcpStream::connect("192.168.1.6:8000").unwrap();
    loop {
        let mut rng = rand::thread_rng();
        let x: i32 = rng.gen_range(10..1000);
        let y: i32 = rng.gen_range(10..1000);

        // 创建Task数据包
        let task = Task::new(x, format!("{}+{}", x, y));
        // 创建Packet
        let packet = Packet::new(PackType::Task, task).unwrap();

        // 将packet序列化
        let packet = packet.pack().unwrap();

        let t = Task::unpack(&packet[get_header_size()..]).unwrap();
        println!("{:?}", t);

        // 发送
        stream.write(&packet).unwrap();
        stream.flush().unwrap();
        // 创建 TaskResult
        let task_result = TaskResult::new(x, Ok(x + y));
        // 创建Packet
        let packet = Packet::new(PackType::TaskResult, task_result).unwrap();
        // 将packet序列化
        let packet = packet.pack().unwrap();
        // 发送
        stream.write(&packet).unwrap();
        stream.flush().unwrap();

        thread::sleep(Duration::from_millis(500));
    }
}
