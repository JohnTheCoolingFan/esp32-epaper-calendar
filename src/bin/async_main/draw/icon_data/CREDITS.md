These icons were taken from https://github.com/AlexGyver/RipCalendar/ (the whole repository is MIT-licensed), decoded and reprocessed (rotated properly and scaled down to 32x32)

In addition, if anyone is curious, here's a rust code snippet used to decode the image data. It's not pretty or totally correct, but it works:

```rs
pub fn bytes_to_chunks<'a>(data: &'a [u8]) -> impl Iterator<Item = Chunk> + 'a {
    let length = u16::from_le_bytes(data[5..=6].try_into().unwrap()) as usize;
    data[7..(5 + length)].chunks(3).flat_map(|bytes| {
        let [a, b, c] = *bytes else {
            //panic!("Wrong chunking: {}", bytes.len());
            return [Chunk::empty(); 4];
        };
        three_bytes_to_chunks([a, b, c])
    })
}

fn three_bytes_to_chunks(bytechunk: [u8; 3]) -> [Chunk; 4] {
    let val0 = bytechunk[0] >> 2;
    let val1 = (bytechunk[0] << 4 & 0b00110000) | (bytechunk[1] >> 4);
    let val2 = (bytechunk[1] << 2 & 0b00111100) | (bytechunk[2] >> 6);
    let val3 = bytechunk[2] & 0b00111111;

    [val0, val1, val2, val3].map(Chunk::from_byte)
}

pub fn test_length(data: &[u8]) -> usize {
    bytes_to_chunks(data).map(|c| c.length as usize).sum()
}

#[derive(Clone, Copy)]
pub struct Chunk {
    pub value: bool,
    pub length: u8,
}

impl Chunk {
    fn from_byte(byte: u8) -> Self {
        Self {
            value: (byte & 1) != 0,
            length: (byte >> 1) & 0b00011111,
        }
    }

    fn empty() -> Self {
        Self {
            value: false,
            length: 0,
        }
    }
}
```
