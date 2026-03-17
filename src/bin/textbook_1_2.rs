const LENGTH_MAX: usize = 6;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <string>", args[0]);
        std::process::exit(1);
    }
    let string = args[1].clone();
    let pre = gen_pre();
    println!("{}", solve(&pre, string));
}

fn gen_pre() -> Vec<Vec<usize>> {
    // dp[i][j] 长度为 i 的递增字符串，以字符 j 开始的数量
    // pre[i][j] 长度为 i 的递增字符串，以字符 [1, j] 开始的数量（做了个前缀和）
    let mut dp = vec![vec![0; 27]; LENGTH_MAX + 1];
    // dp[1][i] = 1
    dp[1].iter_mut().skip(1).for_each(|x| *x = 1);
    for i in 2..=LENGTH_MAX {
        for j in 1..=25 {
            dp[i][j] = dp[i - 1][j + 1..].iter().sum();
        }
    }
    let mut pre = vec![vec![0; 27]; LENGTH_MAX + 1];
    for i in 1..=LENGTH_MAX {
        for j in 1..=26 {
            pre[i][j] = pre[i][j - 1] + dp[i][j];
        }
    }
    pre
}

fn solve(dp: &[Vec<usize>], string: String) -> usize {
    let len = string.len();
    let mut low_bound = 0;
    // low_bound += dp[i][26] for i in [1, len)
    dp.iter().take(len).skip(1).for_each(|x| low_bound += x[26]);
    let mut last_base = 0;
    for (i, c) in string.chars().enumerate() {
        let l = last_base as usize + 1;
        let r = (c as u8 - b'a') as usize;
        // [l, r]
        low_bound += dp[len - i][r] - dp[len - i][l - 1];
        last_base = (c as u8 - b'a') + 1;
    }
    low_bound + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    fn gen_list_item(string: String, list: &mut Vec<String>) {
        if string.len() == LENGTH_MAX {
            return;
        }
        let base = string
            .chars()
            .last()
            .map(|c| c as u8 - 'a' as u8 + 1)
            .unwrap_or(0);
        for i in base + 1..=26 {
            let mut new_string = string.clone();
            new_string.push(('a' as u8 + i - 1) as char);
            list.push(new_string.clone());
            gen_list_item(new_string, list);
        }
    }

    fn cmp(a: &String, b: &String) -> Ordering {
        if a.len() == b.len() {
            a.cmp(b)
        } else {
            a.len().cmp(&b.len())
        }
    }

    fn gen_list() -> Vec<String> {
        let mut list = Vec::new();
        gen_list_item(String::new(), &mut list);
        list.sort_by(cmp);
        list
    }

    #[test]
    #[ignore]
    fn test() {
        let pre = gen_pre();
        let list = gen_list();
        for (i, s) in list.iter().enumerate() {
            assert_eq!(solve(&pre, s.clone()), i + 1);
        }
    }
}
