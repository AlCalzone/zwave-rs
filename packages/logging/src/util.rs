use unicode_segmentation::UnicodeSegmentation;

pub fn str_width(string: &str) -> usize {
    string.graphemes(true).count()
}

