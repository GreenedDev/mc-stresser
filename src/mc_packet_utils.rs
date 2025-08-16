use tokio::{io::AsyncWriteExt, net::TcpStream};

// Send handshake packet to initiate connection
pub async fn send_handshake(
    stream: &mut TcpStream,
    proto: u16,
    hostname: &str,
    srv_port: u16,
    next_state: u8,
) {
    _ = write_u8_varint(stream, 0x00).await;

    _ = write_u16_varint(stream, proto).await;
    _ = write_string(stream, hostname).await;
    _ = write_unsigned_short(stream, srv_port).await;
    _ = write_u8_varint(stream, next_state).await;
    _ = stream.flush().await;
}
pub async fn write_u8_varint(stream: &mut TcpStream, val: u8) {
    let mut value = val;
    loop {
        if (value & !0x7F) == 0 {
            if stream.write_all(&[value]).await.is_err() {
                break;
            }
            return;
        }

        if stream.write_all(&[((value & 0x7F) | 0x80)]).await.is_err() {
            break;
        }

        value >>= 7;
    }
}
pub async fn write_u16_varint(stream: &mut TcpStream, val: u16) {
    let mut value = val;
    loop {
        if (value & !0x7F) == 0 {
            if stream.write_all(&[value as u8]).await.is_err() {
                break;
            }
            return;
        }

        if stream
            .write_all(&[((value & 0x7F) | 0x80) as u8])
            .await
            .is_err()
        {
            break;
        }

        value >>= 7;
    }
}
pub async fn write_usize_varint(stream: &mut TcpStream, val: usize) {
    let mut value = val;
    loop {
        if (value & !0x7F) == 0 {
            if stream.write_all(&[value as u8]).await.is_err() {
                break;
            }
            return;
        }

        if stream
            .write_all(&[((value & 0x7F) | 0x80) as u8])
            .await
            .is_err()
        {
            break;
        }

        value >>= 7;
    }
}
pub async fn write_string(stream: &mut TcpStream, val: &str) {
    let bytes = val.as_bytes();
    write_usize_varint(stream, bytes.len()).await;
    let _ = stream.write_all(bytes).await;
}
pub async fn write_unsigned_short(stream: &mut TcpStream, val: u16) {
    let _ = stream.write_all(&val.to_be_bytes()).await;
}
