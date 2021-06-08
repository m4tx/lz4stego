use crate::numeral_coding::{Decoder, Encoder};

fn encode(values: &[u16], max_values: &[u16]) -> Vec<u8> {
    let mut encoder = Encoder::new();

    for (value, max_value) in values.iter().zip(max_values) {
        encoder.add_value(*value, *max_value);
    }

    encoder.finish()
}

fn decode(data: &[u8], max_values: &[u16]) -> Vec<u16> {
    let mut result = Vec::new();
    let mut decoder = Decoder::new(data);

    for max_val in max_values {
        result.push(decoder.decode_value(*max_val));
    }

    result
}

#[test]
fn encode_empty() {
    let values = vec![0];
    let max_values = vec![2];
    let expected = vec![];

    let result = encode(&values, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn decode_empty() {
    let data = vec![];
    let max_values = vec![2];
    let expected = vec![0];

    let result = decode(&data, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn encode_single() {
    let values = vec![1];
    let max_values = vec![2];
    let expected = vec![1];

    let result = encode(&values, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn decode_single() {
    let data = vec![1];
    let max_values = vec![2];
    let expected = vec![1];

    let result = decode(&data, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn encode_bytes_short() {
    let values = vec![15, 129];
    let max_values = vec![256, 256];
    let expected = vec![129, 15];

    let result = encode(&values, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn decode_bytes_short() {
    let data = vec![129, 15];
    let max_values = vec![256, 256];
    let expected = vec![15, 129];

    let result = decode(&data, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn encode_bytes() {
    let values = vec![15, 129, 215, 66, 100, 121, 5, 199];
    let max_values = vec![256, 256, 256, 256, 256, 256, 256, 256];
    let expected = vec![199, 5, 15, 129, 215, 66, 100, 121];

    let result = encode(&values, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn decode_bytes() {
    let data = vec![199, 5, 15, 129, 215, 66, 100, 121];
    let max_values = vec![256, 256, 256, 256, 256, 256, 256, 256];
    let expected = vec![15, 129, 215, 66, 100, 121, 5, 199];

    let result = decode(&data, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn encode_mixed_short() {
    let values = vec![3, 6, 10];
    let max_values = vec![10, 7, 53];
    let expected = vec![2, 251];

    let result = encode(&values, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn decode_mixed_short() {
    let data = vec![2, 251];
    let max_values = vec![10, 7, 53];
    let expected = vec![3, 6, 10];

    let result = decode(&data, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn encode_mixed_medium() {
    let values = vec![3, 6, 10, 100, 57, 42, 13, 20];
    let max_values = vec![10, 7, 53, 256, 2133, 100, 15, 256];
    let expected = vec![57, 191, 128, 100, 22, 191];

    let result = encode(&values, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn decode_mixed_medium() {
    let data = vec![57, 191, 128, 100, 22, 191];
    let max_values = vec![10, 7, 53, 256, 2133, 100, 15, 256];
    let expected = vec![3, 6, 10, 100, 57, 42, 13, 20];

    let result = decode(&data, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn encode_mixed_long() {
    let values = vec![
        709, 337, 145, 429, 945, 234, 267, 218, 259, 449, 596, 795, 377, 979, 407, 205, 769, 224,
        760, 545, 993, 467, 439, 55, 331, 767, 316, 463, 860, 56, 355, 679, 365, 725, 959, 444,
        669, 999, 549, 990, 738, 560, 951, 397, 748, 0, 125, 166, 214, 445,
    ];
    let max_values = vec![
        1366, 1160, 4796, 7814, 4199, 9361, 7434, 7265, 5424, 945, 7381, 2125, 8772, 3415, 8975,
        7452, 7106, 2832, 3473, 4004, 1300, 4373, 6373, 9408, 5104, 7354, 4471, 1195, 8586, 9304,
        8251, 3802, 5539, 8627, 7941, 1512, 7303, 5146, 559, 5481, 6868, 5058, 9318, 3976, 5698,
        696, 3358, 3568, 344, 9533,
    ];
    let expected = vec![
        36, 172, 175, 113, 189, 145, 9, 221, 117, 234, 135, 161, 96, 242, 83, 213, 47, 73, 89, 33,
        113, 156, 242, 11, 182, 213, 213, 163, 96, 68, 216, 33, 29, 203, 152, 232, 105, 183, 83,
        187, 127, 142, 72, 237, 197, 50, 8, 68, 54, 167, 36, 80, 49, 117, 124, 168, 108, 225, 241,
        151, 196, 243, 3, 206, 112, 198, 171, 137, 245, 252, 232, 20, 63, 70, 206,
    ];

    let result = encode(&values, &max_values);

    assert_eq!(result, expected);
}

#[test]
fn decode_mixed_long() {
    let data = vec![
        36, 172, 175, 113, 189, 145, 9, 221, 117, 234, 135, 161, 96, 242, 83, 213, 47, 73, 89, 33,
        113, 156, 242, 11, 182, 213, 213, 163, 96, 68, 216, 33, 29, 203, 152, 232, 105, 183, 83,
        187, 127, 142, 72, 237, 197, 50, 8, 68, 54, 167, 36, 80, 49, 117, 124, 168, 108, 225, 241,
        151, 196, 243, 3, 206, 112, 198, 171, 137, 245, 252, 232, 20, 63, 70, 206,
    ];
    let max_values = vec![
        1366, 1160, 4796, 7814, 4199, 9361, 7434, 7265, 5424, 945, 7381, 2125, 8772, 3415, 8975,
        7452, 7106, 2832, 3473, 4004, 1300, 4373, 6373, 9408, 5104, 7354, 4471, 1195, 8586, 9304,
        8251, 3802, 5539, 8627, 7941, 1512, 7303, 5146, 559, 5481, 6868, 5058, 9318, 3976, 5698,
        696, 3358, 3568, 344, 9533,
    ];
    let expected = vec![
        709, 337, 145, 429, 945, 234, 267, 218, 259, 449, 596, 795, 377, 979, 407, 205, 769, 224,
        760, 545, 993, 467, 439, 55, 331, 767, 316, 463, 860, 56, 355, 679, 365, 725, 959, 444,
        669, 999, 549, 990, 738, 560, 951, 397, 748, 0, 125, 166, 214, 445,
    ];

    let result = decode(&data, &max_values);

    assert_eq!(result, expected);
}
