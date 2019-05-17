use std::convert::{TryFrom, TryInto};
use std::io::Cursor;
use std::mem::size_of;

use bytes::Buf;

use super::{File, ParseError};

// magic:u32 + version:u16 + sections:u8
const HEADER_SIZE: usize = size_of::<u32>() + size_of::<u16>() + size_of::<u8>();
const MAGIC: u32 = 0x414d5858;
const SUPPORTED_VERSION: u16 = 768;

impl TryFrom<&[u8]> for File {
    type Error = ParseError;

    fn try_from(source: &[u8]) -> Result<File, Self::Error> {
        if source.len() < HEADER_SIZE {
            return Err(ParseError::HeaderSizeMismatch);
        }

        let mut reader = Cursor::new(source);

        if reader.get_u32_le() != MAGIC {
            return Err(ParseError::MagicMismatch);
        }

        let version = reader.get_u16_le();
        if version != SUPPORTED_VERSION {
            return Err(ParseError::UnsupportedVersion {
                supported: SUPPORTED_VERSION,
                requested: version,
            });
        }

        let sections_count = reader.get_u8();
        if sections_count == 0 {
            return Err(ParseError::NoSections);
        }

        let bin_position: usize = reader
            .position()
            .try_into()
            .expect("HEADER_SIZE is small on any platform to fit");

        // Cut binary to only include raw sections
        let sections_bin = source
            .get(bin_position..)
            .ok_or_else(|| ParseError::NoSections)?
            .to_owned();

        Ok(File {
            sections_count,
            sections_bin,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::fs::File as IoFile;
    use std::io::{self, Read};

    use super::{File as AmxxFile, ParseError};
    use crate::amxx::file::parser::SUPPORTED_VERSION;

    fn _read_file(path: &str) -> io::Result<Vec<u8>> {
        let mut file = IoFile::open(path)?;
        let mut plugin = vec![];
        file.read_to_end(&mut plugin)?;

        Ok(plugin)
    }

    fn read_file(path: &str) -> Vec<u8> {
        _read_file(path).expect(&format!("Could not read {} file", path))
    }

    #[test]
    fn it_parses_correct_file() {
        let plugin_bin = read_file("test/fixtures/amxx/simple.amxx183");
        let _plugin = AmxxFile::try_from(&plugin_bin[..]).expect("Could not parse amxx file");
    }

    #[test]
    fn it_fails_with_header_mismatch() {
        const EMPTY_FILE: &[u8] = b"";

        match AmxxFile::try_from(EMPTY_FILE).err() {
            Some(ParseError::HeaderSizeMismatch) => (),
            _ => panic!("Error should be ParseError::HeaderSizeMismatch"),
        }
    }

    #[test]
    fn it_fails_with_magic_mismatch() {
        const WRONG_MAGIC_HEADER: &[u8] = b"XXXA\0\0\0";

        match AmxxFile::try_from(WRONG_MAGIC_HEADER).err() {
            Some(ParseError::MagicMismatch) => (),
            _ => panic!("Error should be ParseError::MagicMismatch"),
        }
    }

    #[test]
    fn it_fails_with_unsupported_version() {
        const WRONG_MAGIC_HEADER: &[u8] = b"XXMA\0\0\0";

        match AmxxFile::try_from(WRONG_MAGIC_HEADER).err() {
            Some(ParseError::UnsupportedVersion {
                supported: SUPPORTED_VERSION,
                requested: 0,
            }) => (),
            Some(ParseError::UnsupportedVersion {
                supported: s,
                requested: r,
            }) => panic!(
                "UnsupportedVersion error contain wrong supported ({}) or requested ({}) versions",
                s, r
            ),
            _ => panic!("Error should be ParseError::UnsupportedVersion"),
        }
    }

    #[test]
    fn it_fails_when_no_sections() {
        const NO_SECTIONS_HEADER: &[u8] = b"XXMA\0\x03\0";

        match AmxxFile::try_from(NO_SECTIONS_HEADER).err() {
            Some(ParseError::NoSections) => (),
            _ => panic!("Error should be ParseError::NoSections"),
        }
    }
}
