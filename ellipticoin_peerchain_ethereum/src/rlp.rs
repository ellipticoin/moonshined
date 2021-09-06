use num_bigint::BigUint;
use num_traits::{FromPrimitive, ToPrimitive};

pub fn encode(input: Vec<Vec<u8>>) -> Vec<u8> {
    let mut result = input
        .iter()
        .map(encode_item)
        .collect::<Vec<Vec<u8>>>()
        .concat();
    if result.len() > 55 {
        let length = BigUint::from_usize(result.len()).unwrap();
        let length_length = length.to_bytes_be().len();
        result = vec![
            [vec![length_length as u8 + 0xf7u8], length.to_bytes_be()].concat(),
            result,
        ]
        .concat();
    } else {
        result.insert(0, result.len() as u8 + 0xc0u8);
    }
    result
}

pub fn encode_item(input: &Vec<u8>) -> Vec<u8> {
    if input.len() == 0 {
        vec![0x80]
    } else if input.len() == 1 && input[0] < 0x7f {
        input.to_vec()
    } else if input.len() < 55 {
        [vec![input.len() as u8 + 0x80u8], input.to_vec()].concat()
    } else {
        let length = BigUint::from_usize(input.len()).unwrap();
        let length_length = length.to_bytes_be().len();
        vec![
            [vec![length_length as u8 + 0xb7u8], length.to_bytes_be()].concat(),
            input.to_vec(),
        ]
        .concat()
    }
}

pub fn decode(input: &[u8]) -> Vec<Vec<u8>> {
    if input[0] == 0xf8 && input[1] as usize == input.len() - 2 {
        decode_next(&input[2..], vec![])
    } else {
        panic!("not a simple list")
    }
}

fn decode_next(input: &[u8], mut output: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let length = match input[0] {
        0x00..=0x7f => {
            output.push(vec![input[0]]);
            0usize
        }
        0x80 => {
            output.push(vec![]);
            0usize
        }
        0x81..=0xb7 => {
            let length = input[0] as usize - 0x80 as usize;
            output.push(input[1..length + 1].to_vec());
            length
        }
        0xb8..=0xbf => {
            let length_length = input[0] as usize - 0xb7 as usize;
            let length = BigUint::from_bytes_be(&input[1..1 + length_length])
                .to_usize()
                .unwrap();
            output.push(input[1 + length_length..1 + length_length + length].to_vec());
            length + length_length
        }

        header => {
            panic!("unknown rlp header: {}", header)
        }
    };
    if length == input.len() - 1 {
        output
    } else {
        decode_next(&input[length + 1..input.len()], output)
    }
}
