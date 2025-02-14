use rand::Rng;

#[test]
fn random_test() {
    let mut verification = [' '; 4];
    let char_array: [char; 62] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    verification.iter_mut().for_each(|x| {
        let append = char_array[rand::rng().random_range(0..char_array.len())];
        *x = append
    });
    let varif = String::from_iter(&verification);
    println!("varif {:?}", &varif);

    assert_eq!(1, 2)
}
