use crate::srt::Subtitle;

use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};

pub fn serialise<P: AsRef<Path>>(subs: Vec<Subtitle>, output: P) -> Result<()> {
    let file = std::fs::File::create(output).context("Failed to create file!")?;
    let mut writer = BufWriter::new(file);
    write_subs(&mut writer, subs).context("Failed to write to output file.")?;
    writer.flush().context("Failed to write to output file.")?;
    Ok(())
}

fn write_subs<W: Write>(buf: &mut W, subs: Vec<Subtitle>) -> Result<()> {
    for sub in subs {
        write_sub(buf, sub)?;
    }
    Ok(())
}

fn write_sub<W: Write>(buf: &mut W, sub: Subtitle) -> Result<()> {
    writeln!(buf, "{}", sub.sequence_number.unwrap())?;
    write_ts(buf, sub.show_at)?;
    write!(buf, " --> ")?;
    write_ts(buf, sub.hide_at)?;
    writeln!(buf)?;
    for line in sub.text {
        writeln!(buf, "{}", line)?;
    }
    writeln!(buf)?;
    Ok(())
}

fn write_ts<W: Write>(buf: &mut W, timestamp: Duration) -> Result<()> {
    let total_secs = timestamp.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = timestamp.as_millis() % 1000;
    write!(
        buf,
        "{:02}:{:02}:{:02},{:03}",
        hours, minutes, seconds, millis
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::Duration;

    macro_rules! test_write_ts {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;

                let ts = Duration::from_millis(input);
                let mut buf = Cursor::new(vec![]);

                write_ts(&mut buf, ts).expect("Failed to write to buffer");

                assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), expected);
            }
        )*
        }
    }

    test_write_ts! {
        test_write_ts_0: (0, "00:00:00,000"),
        test_write_ts_1: (1, "00:00:00,001"),
        test_write_ts_2: (999, "00:00:00,999"),
        test_write_ts_3: (1000, "00:00:01,000"),
        test_write_ts_4: (1001, "00:00:01,001"),
        test_write_ts_5: (59_999, "00:00:59,999"),
        test_write_ts_6: (60_000, "00:01:00,000"),
        test_write_ts_7: (3_600_000, "01:00:00,000"),
        test_write_ts_8: (7_326_159, "02:02:06,159"),
        test_write_ts_9: (34_380_001, "09:33:00,001"),
        test_write_ts_10: (360_000_001, "100:00:00,001"),
    }
}
