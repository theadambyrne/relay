use sys_info::{cpu_speed, loadavg, mem_info, proc_total};
use config::{Config, File};
use lapin::options::BasicPublishOptions;
use lapin::{BasicProperties, Connection, ConnectionProperties};
use serde::Deserialize;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct AmqpConfig {
    host: String,
    port: u16,
    virtual_host: String,
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    amqp: AmqpConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = Config::default();
    settings.merge(File::with_name("../config")).unwrap();
    let app_config: AppConfig = settings.try_into().unwrap();

    let addr = format!(
        "amqp://{}:{}@{}:{}/{}",
        app_config.amqp.username,
        app_config.amqp.password,
        app_config.amqp.host,
        app_config.amqp.port,
        app_config.amqp.virtual_host
    );

    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;

    let channel = conn.create_channel().await?;
    let queue_name = "healthcheck";
    channel
        .queue_declare(queue_name, Default::default(), Default::default())
        .await?;

    let duration = Duration::from_secs(60);
    loop {
        let cpu_speed = cpu_speed().unwrap();
        let loadavg = loadavg().unwrap();
        let mem_info = mem_info().unwrap();
        let proc_total = proc_total().unwrap();


        let payload = format!(
            "CPU Speed: {} MHz\nLoad Average: {:.2}, {:.2}, {:.2}\nMemory Info: Total: {} KB, Free: {} KB, Available: {} KB, Buffers: {} KB, Cached: {} KB\nTotal Processes: {}",
            cpu_speed,
            loadavg.one, loadavg.five, loadavg.fifteen,
            mem_info.total, mem_info.free, mem_info.avail, mem_info.buffers, mem_info.cached,
            proc_total
        );

        channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                payload.as_bytes(),
                BasicProperties::default(),
            )
            .await?;

        sleep(duration).await;
    }
}

