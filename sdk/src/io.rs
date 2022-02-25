use std::io::Seek;

use crate::{types, Error};
use crossbeam_channel as cc;

const RPC_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub(crate) enum OpCode {
    Handshake = 0,
    Frame = 1,
    Close = 2,
    Ping = 3,
    Pong = 4,
}

/// Message immediately sent to Discord upon establishing a connection
#[derive(serde::Serialize)]
pub(crate) struct Handshake {
    /// The RPC version we support
    #[serde(rename = "v")]
    version: u32,
    /// The unique identifier for this application
    client_id: String,
}

/// Parses the frame header for a message from Discord, which just consists
/// of a 4 byte opcode and a 4 byte length of the actual message payload
fn parse_frame_header(header: [u8; 8]) -> Result<(OpCode, u32), Error> {
    let op_code = {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(&header[..4]);

        u32::from_le_bytes(bytes)
    };

    let op_code = match op_code {
        0 => OpCode::Handshake,
        1 => OpCode::Frame,
        2 => OpCode::Close,
        3 => OpCode::Ping,
        4 => OpCode::Pong,
        unknown => {
            return Err(Error::UnknownVariant {
                kind: "OpCode",
                value: unknown,
            })
        }
    };

    let len = {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(&header[4..8]);

        u32::from_le_bytes(bytes)
    };

    Ok((op_code, len))
}

pub(crate) fn serialize_message(
    op_code: OpCode,
    data: &impl serde::Serialize,
    buffer: &mut Vec<u8>,
) -> Result<(), Error> {
    let start = buffer.len();

    buffer.extend_from_slice(&(op_code as u32).to_le_bytes());
    buffer.extend_from_slice(&[0; 4]);

    // We have to pass the whole vec, but since serde_json::to_writer doesn't
    // give us the Write back we have to wrap it in a cursor, but then we need
    // to advance it to point in the buffer we actually want to write the JSON
    // to, otherwise it will overwrite the beginning and make everyone sad
    let mut cursor = std::io::Cursor::new(buffer);
    cursor
        .seek(std::io::SeekFrom::Start(start as u64 + 8))
        .unwrap();

    match serde_json::to_writer(&mut cursor, data) {
        Ok(_) => {
            let buffer = cursor.into_inner();
            let data_len = (buffer.len() - start - 8) as u32;
            buffer[start + 4..start + 8].copy_from_slice(&data_len.to_le_bytes());
        }
        Err(e) => {
            let buffer = cursor.into_inner();
            buffer.truncate(start);
            return Err(e.into());
        }
    }

    Ok(())
}

fn make_message(op_code: OpCode, data: &[u8]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(data.len() + 8);
    msg.extend_from_slice(&(op_code as u32).to_le_bytes());
    msg.extend_from_slice(&(data.len() as u32).to_le_bytes());
    msg.extend_from_slice(data);

    msg
}

pub(crate) struct IoTask {
    /// The queue of messages to send to Discord
    pub(crate) stx: cc::Sender<Option<Vec<u8>>>,
    /// The queue of RPCs sent from Discord
    pub(crate) rrx: tokio::sync::mpsc::Receiver<IoMsg>,
    /// The handle to the task
    pub(crate) handle: tokio::task::JoinHandle<()>,
}

pub(crate) enum IoMsg {
    Disconnected(Error),
    Frame(Vec<u8>),
}

#[cfg(unix)]
type Pipe = tokio::net::UnixStream;
#[cfg(windows)]
type Pipe = tokio::net::windows::named_pipe::NamedPipeClient;

pub(crate) fn start_io_task(app_id: i64) -> IoTask {
    #[cfg(unix)]
    async fn connect() -> Result<Pipe, Error> {
        let tmp_path = std::env::var("XDG_RUNTIME_DIR")
            .or_else(|_| std::env::var("TMPDIR"))
            .or_else(|_| std::env::var("TMP"))
            .or_else(|_| std::env::var("TEMP"))
            .unwrap_or_else(|_| "/tmp".to_owned());

        #[cfg(feature = "local-testing")]
        if let Ok(id) = std::env::var("DISCORD_INSTANCE_ID") {
            let socket_path = format!("{}/discord-ipc-{}", tmp_path, id);

            return match Pipe::connect(&socket_path).await {
                Ok(stream) => {
                    tracing::debug!("connected to {}!", socket_path);
                    Ok(stream)
                }
                Err(e) => {
                    tracing::error!("Unable to connect to {}: {}", socket_path, e);
                    Err(Error::io("connecting to socket", e))
                }
            };
        }

        // Discord just uses a simple round robin approach to finding a socket to use
        let mut socket_path = format!("{}/discord-ipc-0", tmp_path);
        for seq in 0..10i32 {
            socket_path.pop();

            use std::fmt::Write;
            write!(&mut socket_path, "{}", seq).unwrap();

            match Pipe::connect(&socket_path).await {
                Ok(stream) => {
                    tracing::debug!("connected to {}!", socket_path);
                    return Ok(stream);
                }
                Err(e) => {
                    tracing::trace!("Unable to connect to {}: {}", socket_path, e);
                }
            }
        }

        Err(Error::NoConnection)
    }

    #[cfg(windows)]
    async fn connect() -> Result<Pipe, Error> {
        use tokio::net::windows::named_pipe::ClientOptions;

        #[cfg(feature = "local-testing")]
        if let Ok(id) = std::env::var("DISCORD_INSTANCE_ID") {
            let socket_path = format!("\\\\?\\pipe\\discord-ipc-{}", id);

            return match ClientOptions::new().open(&socket_path) {
                Ok(stream) => {
                    tracing::debug!("connected to {}!", socket_path);
                    Ok(stream)
                }
                Err(e) => {
                    tracing::error!("Unable to connect to {}: {}", socket_path, e);
                    Err(Error::io("connecting to socket", e))
                }
            };
        }

        // Discord just uses a simple round robin approach to finding a socket to use
        let mut socket_path = "\\\\?\\pipe\\discord-ipc-0".to_owned();
        for seq in 0..10i32 {
            socket_path.pop();
            use std::fmt::Write;
            write!(&mut socket_path, "{}", seq).unwrap();

            match ClientOptions::new().open(&socket_path) {
                Ok(stream) => {
                    tracing::debug!("connected to {}!", socket_path);
                    return Ok(stream);
                }
                Err(e) => {
                    tracing::trace!("Unable to connect to {}: {}", socket_path, e);
                }
            }
        }

        Err(Error::NoConnection)
    }

    // Send queue
    let (stx, srx) = cc::bounded::<Option<Vec<u8>>>(100);
    // Receive queue
    let (rtx, rrx) = tokio::sync::mpsc::channel(100);

    // The io thread also sends messages
    let io_stx = stx.clone();

    let handle = tokio::task::spawn(async move {
        async fn io_loop(
            stream: impl SocketStream,
            app_id: i64,
            stx: &cc::Sender<Option<Vec<u8>>>,
            srx: &cc::Receiver<Option<Vec<u8>>>,
            rtx: &tokio::sync::mpsc::Sender<IoMsg>,
        ) -> Result<(), Error> {
            // We always send the handshake immediately on establishing a connection,
            // Discord should then respond with a `Ready` RPC
            let mut handshake = Vec::with_capacity(128);
            serialize_message(
                OpCode::Handshake,
                &Handshake {
                    version: RPC_VERSION,
                    client_id: app_id.to_string(),
                },
                &mut handshake,
            )?;

            stx.send(Some(handshake))?;

            struct ReadBuf<const N: usize> {
                buf: [u8; N],
                cursor: usize,
            }

            impl<const N: usize> ReadBuf<N> {
                fn new() -> Self {
                    Self {
                        buf: [0u8; N],
                        cursor: 0,
                    }
                }
            }

            let mut header_buf = ReadBuf::<8>::new();
            let mut data_buf = Vec::with_capacity(1024);
            let mut data_cursor = 0;
            let mut valid_header: Option<(OpCode, u32)> = None;
            let mut top_message: Option<(Vec<u8>, usize)> = None;

            let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));

            loop {
                // We use crossbeam channels for sending messages to this I/O
                // task as they provide a little more functionality compared to
                // tokio mpsc channels, but that means we need some way to sleep
                // this task, as otherwise the stream.ready() is basically always
                // going to immediately return and report it is writable which
                // causes this task to peg a core and actually cause tokio to
                // fail to wake other tasks, however, we do try and read all data
                // that is pending on the pipe each tick, so it's essentially
                // just the write that is limited to a maximum of 1 per tick
                // which is fine since the tick is quite small relative to the
                // amount of messages we actually send to Discord
                interval.tick().await;

                let ready = stream
                    .ready(tokio::io::Interest::READABLE | tokio::io::Interest::WRITABLE)
                    .await
                    .map_err(|e| Error::io("polling socket readiness", e))?;

                if ready.is_readable() {
                    'read: loop {
                        let buf = match &valid_header {
                            Some((_, len)) => &mut data_buf[data_cursor..*len as usize],
                            None => &mut header_buf.buf[header_buf.cursor..],
                        };

                        match stream.try_read(buf) {
                            Ok(n) => {
                                if n == 0 {
                                    return Err(Error::NoConnection);
                                }

                                if let Some((op, len)) = valid_header {
                                    data_cursor += n;
                                    let len = len as usize;
                                    if data_cursor == len {
                                        match op {
                                            OpCode::Close => {
                                                let close: types::CloseFrame<'_> =
                                                    serde_json::from_slice(&data_buf)?;

                                                tracing::debug!("Received close request from Discord: {:?} - {:?}", close.code, close.message);
                                                return Err(Error::Close(
                                                    close
                                                        .message
                                                        .unwrap_or("unknown reason")
                                                        .to_owned(),
                                                ));
                                            }
                                            OpCode::Frame => {
                                                if rtx
                                                    .send(IoMsg::Frame(data_buf.clone()))
                                                    .await
                                                    .is_err()
                                                {
                                                    tracing::error!(
                                                        "Dropped RPC as queue is too full"
                                                    );
                                                }
                                            }
                                            OpCode::Ping => {
                                                let pong_response =
                                                    make_message(OpCode::Pong, &data_buf);
                                                tracing::debug!(
                                                    "Responding to PING request from Discord"
                                                );
                                                stx.send(Some(pong_response))?;
                                            }
                                            OpCode::Pong => {
                                                tracing::debug!(
                                                    "Received PONG response from Discord"
                                                );
                                            }
                                            OpCode::Handshake => {
                                                tracing::error!("Received a HANDSHAKE request from Discord, the stream is likely corrupt");
                                                return Err(Error::CorruptConnection);
                                            }
                                        }

                                        valid_header = None;
                                        header_buf.cursor = 0;
                                        data_buf.clear();
                                        data_cursor = 0;
                                    }
                                } else {
                                    header_buf.cursor += n;
                                    if header_buf.cursor == header_buf.buf.len() {
                                        let header = parse_frame_header(header_buf.buf)?;

                                        // Ensure the data buffer has enough space
                                        data_buf.resize(header.1 as usize, 0);

                                        valid_header = Some(header);
                                    }
                                }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                break 'read;
                            }
                            Err(e) => {
                                return Err(Error::io("reading socket", e));
                            }
                        }
                    }
                }

                if ready.is_writable() {
                    if top_message.is_none() {
                        if let Ok(msg) = srx.try_recv() {
                            top_message = if let Some(msg) = msg {
                                Some((msg, 0))
                            } else {
                                tracing::debug!("Discord I/O thread received shutdown signal");
                                return Ok(());
                            };
                        }
                    }

                    if let Some((message, cursor)) = &mut top_message {
                        let to_write = message.len() - *cursor;
                        match stream.try_write(&message[*cursor..]) {
                            Ok(n) => {
                                if n == to_write {
                                    top_message = None;
                                } else {
                                    *cursor += n;
                                }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                continue;
                            }
                            Err(e) => {
                                return Err(Error::io("writing socket", e));
                            }
                        }
                    }
                }
            }
        }

        let mut reconnect_dur = std::time::Duration::from_millis(500);

        loop {
            match connect().await {
                Err(e) => {
                    tracing::debug!("Failed to connect to Discord: {}", e);

                    reconnect_dur *= 2;
                    if reconnect_dur.as_secs() > 60 {
                        reconnect_dur = std::time::Duration::from_secs(60);
                    }

                    tokio::time::sleep(reconnect_dur).await;
                }
                Ok(stream) => {
                    reconnect_dur = std::time::Duration::from_millis(500);
                    match io_loop(stream, app_id, &io_stx, &srx, &rtx).await {
                        Err(e) => {
                            tracing::debug!("I/O loop failed: {:#}", e);

                            if let Error::Close(e) = &e {
                                tracing::warn!(
                                    reason = %e,
                                    "Shutting down I/O loop due to Discord close request"
                                );
                                return;
                            }

                            if rtx.try_send(IoMsg::Disconnected(e)).is_err() {
                                tracing::error!("Dropped disconnect message as queue is too full");
                            }

                            // Drain the send queue so we don't confuse Discord
                            while let Ok(msg) = srx.try_recv() {
                                // Also while we're here, check if we actually want
                                // to exit altogether
                                //
                                // TODO: also need to check this when we're not
                                // connected to Discord at all
                                if msg.is_none() {
                                    return;
                                }
                            }

                            tokio::time::sleep(reconnect_dur).await;
                        }
                        Ok(_) => return,
                    }
                }
            }
        }
    });

    IoTask { stx, rrx, handle }
}

// UnixStream and NamedPipe both have the same high level interface, but those
// aren't traits, just regular methods, so we unify them in our own trait
#[async_trait::async_trait]
trait SocketStream {
    async fn ready(&self, interest: tokio::io::Interest) -> std::io::Result<tokio::io::Ready>;
    fn try_read(&self, buf: &mut [u8]) -> std::io::Result<usize>;
    fn try_write(&self, buf: &[u8]) -> std::io::Result<usize>;
}

#[cfg(unix)]
#[async_trait::async_trait]
impl SocketStream for Pipe {
    async fn ready(&self, interest: tokio::io::Interest) -> std::io::Result<tokio::io::Ready> {
        self.ready(interest).await
    }
    #[inline]
    fn try_read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.try_read(buf)
    }
    #[inline]
    fn try_write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.try_write(buf)
    }
}

#[cfg(windows)]
#[async_trait::async_trait]
impl SocketStream for Pipe {
    async fn ready(&self, interest: tokio::io::Interest) -> std::io::Result<tokio::io::Ready> {
        self.ready(interest).await
    }
    #[inline]
    fn try_read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.try_read(buf)
    }
    #[inline]
    fn try_write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.try_write(buf)
    }
}
