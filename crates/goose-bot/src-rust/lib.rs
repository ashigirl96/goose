// Goose Bot のRustコード
// これはGoose Bot のシェア可能なRustコードの部分です
// 将来的にはgoose-cliとの連携に使う可能性がある

pub fn version() -> &'static str {
    "0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.0");
    }
}