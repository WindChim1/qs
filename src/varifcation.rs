use rand::Rng;

const CHAR_ARRAY: [char; 62] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];
pub const VARIFCATION_LEN: usize = 4;

pub fn gen_varif(len: usize) -> String {
    let mut verification = vec![' '; len];
    verification.iter_mut().for_each(|x| {
        let append = CHAR_ARRAY[rand::rng().random_range(0..CHAR_ARRAY.len())];
        *x = append
    });
    String::from_iter(&verification)
}
