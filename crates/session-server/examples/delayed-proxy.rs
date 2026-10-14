//! Development-only control transport impairment. Never carries media or logs payloads.
use tokio::{io::{AsyncRead,AsyncReadExt,AsyncWrite,AsyncWriteExt},net::{TcpListener,TcpStream}};
use std::time::Duration;
async fn relay(mut input:impl AsyncRead+Unpin,mut output:impl AsyncWrite+Unpin,base:u64,jitter:u64,mut seed:u64) -> std::io::Result<()> {
    let mut buffer=[0;16384];
    loop {
        let length=input.read(&mut buffer).await?;
        if length==0 {output.shutdown().await?;return Ok(());}
        seed=seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let offset=if jitter==0 {0} else {(seed>>32)%(2*jitter+1)};
        let delay=(base+offset).saturating_sub(jitter);
        tokio::time::sleep(Duration::from_millis(delay)).await;
        output.write_all(&buffer[..length]).await?;
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    anyhow::ensure!(cfg!(debug_assertions),"control impairment is debug-only");
    let mut args=std::env::args().skip(1);
    let rtt:u64=args.next().unwrap_or("0".into()).parse()?;
    let jitter:u64=args.next().unwrap_or("0".into()).parse()?;
    anyhow::ensure!(rtt<=1600 && jitter<=400,"test delay out of range");
    let listener=TcpListener::bind("127.0.0.1:47821").await?;
    let mut connection=0u64;
    loop {
        let (client,_)=listener.accept().await?;connection+=1;
        tokio::spawn(async move {
            let Ok(server)=TcpStream::connect("127.0.0.1:47820").await else {return};
            let _=client.set_nodelay(true);let _=server.set_nodelay(true);
            let (cr,cw)=client.into_split();let (sr,sw)=server.into_split();
            // First finished direction cancels the other, immediately propagating disconnect.
            tokio::select! { _=relay(cr,sw,rtt/2,jitter,connection)=>{}, _=relay(sr,cw,rtt/2,jitter,connection+17)=>{} }
        });
    }
}
