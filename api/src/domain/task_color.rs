#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskColorError {
    InvalidFormat,
}

impl TaskColorError {
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidFormat => "Farbe muss im Format #RRGGBB angegeben werden",
        }
    }
}

const MIN_PRESELECT_LUMINANCE: f64 = 0.18;
const MAX_PRESELECT_LUMINANCE: f64 = 0.75;
const MIN_SATURATION: f64 = 0.45;
const MAX_SATURATION: f64 = 0.80;
const MIN_LIGHTNESS: f64 = 0.38;
const MAX_LIGHTNESS: f64 = 0.62;

pub fn parse_hex_color(value: &str) -> Result<String, TaskColorError> {
    if value.len() != 7 || !value.starts_with('#') {
        return Err(TaskColorError::InvalidFormat);
    }
    let digits = &value[1..];
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(TaskColorError::InvalidFormat);
    }
    Ok(format!("#{}", digits.to_ascii_lowercase()))
}

pub fn is_allowed_preselect(hex: &str) -> bool {
    let Ok(normalized) = parse_hex_color(hex) else {
        return false;
    };
    let Some((r, g, b)) = rgb_from_hex(&normalized) else {
        return false;
    };
    let luminance = relative_luminance(r, g, b);
    (MIN_PRESELECT_LUMINANCE..=MAX_PRESELECT_LUMINANCE).contains(&luminance)
}

pub fn random_task_color<R: rand::Rng + ?Sized>(rng: &mut R) -> String {
    loop {
        let hue = rng.gen_range(0.0..360.0);
        let saturation = rng.gen_range(MIN_SATURATION..=MAX_SATURATION);
        let lightness = rng.gen_range(MIN_LIGHTNESS..=MAX_LIGHTNESS);
        let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);
        let color = format!("#{r:02x}{g:02x}{b:02x}");
        if is_allowed_preselect(&color) {
            return color;
        }
    }
}

fn rgb_from_hex(hex: &str) -> Option<(u8, u8, u8)> {
    if hex.len() != 7 {
        return None;
    }
    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
    Some((r, g, b))
}

fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    0.2126 * channel_to_linear(r) + 0.7152 * channel_to_linear(g) + 0.0722 * channel_to_linear(b)
}

fn channel_to_linear(channel: u8) -> f64 {
    let srgb = f64::from(channel) / 255.0;
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> (u8, u8, u8) {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_prime = hue / 60.0;
    let x = chroma * (1.0 - (hue_prime.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match hue_prime {
        value if value < 1.0 => (chroma, x, 0.0),
        value if value < 2.0 => (x, chroma, 0.0),
        value if value < 3.0 => (0.0, chroma, x),
        value if value < 4.0 => (0.0, x, chroma),
        value if value < 5.0 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let match_lightness = lightness - chroma / 2.0;
    (
        to_u8(r1 + match_lightness),
        to_u8(g1 + match_lightness),
        to_u8(b1 + match_lightness),
    )
}

fn to_u8(channel: f64) -> u8 {
    (channel * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn parse_accepts_user_black_and_normalizes_case() {
        assert_eq!(parse_hex_color("#000000").expect("black"), "#000000");
        assert_eq!(parse_hex_color("#FFFFFF").expect("white"), "#ffffff");
        assert_eq!(parse_hex_color("#3F9D6C").expect("mid"), "#3f9d6c");
    }

    #[test]
    fn parse_rejects_invalid_hex() {
        for value in [
            "", "3f9d6c", "#fff", "#3f9d6", "#3f9d6cc", "#gg0000", " #3f9d6c",
        ] {
            assert_eq!(parse_hex_color(value), Err(TaskColorError::InvalidFormat));
        }
    }

    #[test]
    fn preselect_rejects_black_white_and_extremes() {
        assert!(!is_allowed_preselect("#000000"));
        assert!(!is_allowed_preselect("#ffffff"));
        assert!(!is_allowed_preselect("#0a0a0a"));
        assert!(!is_allowed_preselect("#f5f5f5"));
    }

    #[test]
    fn preselect_accepts_mid_tones() {
        assert!(is_allowed_preselect("#3f9d6c"));
        assert!(is_allowed_preselect("#e4b04a"));
        assert!(is_allowed_preselect("#6b8cae"));
    }

    #[test]
    fn random_color_stays_in_preselect_band() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..64 {
            let color = random_task_color(&mut rng);
            assert!(
                is_allowed_preselect(&color),
                "generated {color} is outside the preselect band"
            );
            assert_eq!(parse_hex_color(&color).as_deref(), Ok(color.as_str()));
        }
    }
}
