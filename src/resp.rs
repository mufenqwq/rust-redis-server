use anyhow::Result;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone, Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
}

impl Value {
    pub fn serialize(self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            _ => panic!("Unsupported value for serialize"),
        }
    }
}

pub struct RespHandler {
    stream: TcpStream,
    buffer: BytesMut,
}
impl RespHandler {
    pub fn new(stream: TcpStream) -> Self {
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let (v, _) = parse_message(self.buffer.split())?;
        Ok(Some(v))
    }
    
    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        self.stream.write(value.serialize().as_bytes()).await?;
        
        Ok(())
    }  
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        // todo 这是什么问题
        _ => Err(anyhow::anyhow!("Not a known value type {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    // +5mufen/r/n
    if let Some((line, len)) = read_until_ctrl(&buffer[1..]) {
        // 5mufen 6
        let string = String::from_utf8(line.to_vec())?;

        return Ok((Value::SimpleString(string), len + 1));
    }
    Err(anyhow::anyhow!("Invalid string {:?}", buffer))
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    //todo *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n
    let (array_length, mut bytes_consumed) =
        if let Some((line, len)) = read_until_ctrl(&buffer[1..]) {
            let array_length = parse_int(line)?;

            (array_length, len + 1)
        } else {
            return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
        };
    let mut items = vec![];
    for _ in 0..array_length {
        let (array_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;

        items.push(array_item);
        bytes_consumed += len;
    }
    Ok((Value::Array(items), bytes_consumed))
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    //todo $6\r\nfoobar\r\n
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_until_ctrl(&buffer[1..]) {
        let bulk_str_len = parse_int(line)?;

        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format{:?}", buffer));
    };
    let end_of_bulk_str = bytes_consumed + bulk_str_len as usize;
    let total_parsed = end_of_bulk_str + 2;
    Ok((
        Value::BulkString(String::from_utf8(
            buffer[bytes_consumed..end_of_bulk_str].to_vec(),
        )?),
        total_parsed,
    ))
}
fn read_until_ctrl(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        // mufen/r/n
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            //
            return Some((&buffer[0..(i - 1)], i + 1)); // (mufen, 7)
        }
    }
    None
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}

