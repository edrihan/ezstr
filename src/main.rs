use ezstr::*;
use regex::Regex;

fn main() {
    // Your actual program logic goes here (if any)
    println!("Run `cargo test` to execute tests.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_and_len() {
        let sample = EzStr::new(
            "*  Thé - Nicotine Dreams   ♩≈117BPM   page 1/2
 By: Édrihan Lévesque   /   Alin Rogoz  ©2024*
[4/4 Pickup]          𝄽  𝄽 𝆔♪  ♪ 𝆔♪  ♪
                     |N.C   A1 C1 A1 G1|");

        assert_eq!(sample.slice(0, -1), sample);
    }

    #[test]
    fn test_regex_and_contains() {
        let sample = EzStr::new("𝆔♪ 𝆔♪");
        let reg = Regex::new("𝆔♪").unwrap();

        assert!(sample.contains("𝆔♪"));

        let captures: Vec<_> = sample.find_iter(&reg).collect();
        assert_eq!(captures.len(), 2); // Expecting two matches
    }
}