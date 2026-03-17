use hnu_algo::algorithms::misc::{cantor_expansion, inverse_cantor_expansion};

fn next_permutation(a: &mut [usize]) -> bool {
    if a.len() < 2 {
        return false;
    }
    let mut i = a.len() - 2;
    while a[i] >= a[i + 1] {
        if i == 0 {
            a.reverse();
            return false;
        }
        i -= 1;
    }
    let mut j = a.len() - 1;
    while a[j] <= a[i] {
        j -= 1;
    }
    a.swap(i, j);
    a[i + 1..].reverse();
    true
}

fn factorial(n: usize) -> usize {
    (1..=n).fold(1, |acc, x| acc * x)
}

#[test]
#[ignore]
fn test_cantor_expansion() {
    for n in 1..=10 {
        let mut perm: Vec<usize> = (1..=n).collect();
        let mut index: usize = 0;

        loop {
            index += 1;

            let rank = cantor_expansion(&perm);
            let restored = inverse_cantor_expansion(rank, n);

            assert_eq!(
                restored, perm,
                "inverse_cantor_expansion failed for n = {n}, rank = {rank}, perm = {:?}",
                perm
            );
            assert_eq!(
                rank, index,
                "cantor_expansion rank mismatch for n = {n}, perm = {:?}",
                perm
            );
            if !next_permutation(&mut perm) {
                break;
            }
        }

        assert_eq!(
            index,
            factorial(n),
            "did not enumerate all permutations for n = {n}"
        );

        println!("test_cantor_expansion passed for n = {n}");
    }
}
