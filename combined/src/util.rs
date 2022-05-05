#[macro_export]
macro_rules! add_substring {
    () => {
        trait StringUtils {
            fn substring(&self, start: usize, len: usize) -> Self;
        }
        impl StringUtils for String {
            fn substring(&self, start: usize, len: usize) -> Self {
                self.chars().skip(start).take(len - start).collect()
            }
        }
    };
}
