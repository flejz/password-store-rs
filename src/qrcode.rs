use qrcode::{EcLevel, QrCode};

/// Render data as a QR code to stdout using Unicode half-block characters.
/// Two rows of modules are packed into one terminal row with ▀/▄/█/space,
/// halving the vertical space needed.
pub fn print_qrcode(data: &str) -> anyhow::Result<()> {
    let code = QrCode::with_error_correction_level(data.as_bytes(), EcLevel::L)
        .map_err(|e| anyhow::anyhow!("QR encode failed: {}", e))?;

    let width = code.width();
    let modules: Vec<bool> = code
        .into_colors()
        .iter()
        .map(|c| *c == qrcode::Color::Dark)
        .collect();

    // Quiet zone: 2 empty rows top/bottom, 2 cols left/right
    let quiet = 2usize;
    let padded_w = width + quiet * 2;

    let row = |y: usize| -> Vec<bool> {
        let mut r = vec![false; padded_w];
        if y >= quiet && y < width + quiet {
            let src_y = y - quiet;
            for x in 0..width {
                r[x + quiet] = modules[src_y * width + x];
            }
        }
        r
    };

    let total_rows = width + quiet * 2;
    let pairs = (total_rows + 1) / 2;

    for pair in 0..pairs {
        let top = row(pair * 2);
        let bot = if pair * 2 + 1 < total_rows {
            row(pair * 2 + 1)
        } else {
            vec![false; padded_w]
        };

        let line: String = (0..padded_w)
            .map(|x| match (top[x], bot[x]) {
                (true, true)   => '█',
                (true, false)  => '▀',
                (false, true)  => '▄',
                (false, false) => ' ',
            })
            .collect();

        println!("{}", line);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qrcode_renders_without_error() {
        // Just verify it doesn't panic/error on valid input
        assert!(print_qrcode("https://example.com").is_ok());
    }

    #[test]
    fn qrcode_renders_short_string() {
        assert!(print_qrcode("pass").is_ok());
    }
}
