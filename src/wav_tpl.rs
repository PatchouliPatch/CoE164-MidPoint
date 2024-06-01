use core::fmt;
use std::fs::File;
use std::path::Path;
use std::error;
use std::io::{self, Read, Seek, SeekFrom};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, BigEndian};

/// Represents a PCM WAV file
pub struct PCMWaveInfo {
    pub riff_header: RiffChunk,
    pub fmt_header: PCMWaveFormatChunk,
    pub data_chunks: Vec <PCMWaveDataChunk>,
}

/// Represents a RIFF chnk from a WAV file
/// 
/// The RIFF chunk is the first 12 bytes of a WAV file.
pub struct RiffChunk {
    pub file_size: u32,
    pub is_big_endian: bool,
}

/// Represents a format chunk from a WAV file
/// 
/// A format chunk in a WAV file starts with a magic string
/// `fmt_` where `_` is a space (0x20 in hex) and then followed by
/// 20 bytes of metadata denoting information about the audio file
/// itself such as the sample and bit rates.
#[derive(Clone, Copy)]
pub struct PCMWaveFormatChunk {
    pub num_channels: u16,
    pub samp_rate: u32,
    pub bps: u16,
}

/// Represents a data chunk from a WAV file
/// 
/// A data chunk in a WAV file starts with a magic string `data` and then
/// followed by the number of samples that follow and then finally the
/// audio data samples themselves.
pub struct PCMWaveDataChunk {
    pub size_bytes: u32,
    pub format: PCMWaveFormatChunk,
    pub data_buf: io::BufReader<File>,
}

/// Represents an iterator to a data chunk from a WAV file
/// 
/// This struct is not instantiated by itself and is generated
/// by calling the methods `PCMWaveDataChunk::chunks_byte_rate()`
/// and `PCMWaveDataChunk::chunks()`.
pub struct PCMWaveDataChunkWindow {
    chunk_size: usize,
    data_chunk: PCMWaveDataChunk
}

/// Represents a WAV reader
pub struct WaveReader;

/// Represents an error in the WAV reader
#[derive(Debug)]
pub enum WaveReaderError {
    NotRiffError,
    NotWaveError,
    NotPCMError,
    ChunkTypeError,
    DataAlignmentError,
    ReadError,
}

impl WaveReader {
    /// Open a PCM WAV file
    /// 
    /// The WAV file located at `file_path` will be represented as a `PCMWaveInfo`
    /// struct for further processing.
    /// 
    /// # Errors
    /// Returns a `WaveReaderError` with the appropriate error if something
    /// happens.
    pub fn open_pcm(file_path: &str) -> Result <PCMWaveInfo, WaveReaderError> {
        let mut fh = File::open(file_path)?;
        let riff_header = Self::read_riff_chunk(&mut fh)?;
        let fmt_header = Self::read_fmt_chunk(&mut fh)?;
        let mut data_chunks = Vec::new();
        
        while let Ok(data_chunk) = Self::read_data_chunk(fh.seek(SeekFrom::Current(0))?, &fmt_header, fh.try_clone()?) {
            data_chunks.push(data_chunk);
        }

        Ok(PCMWaveInfo {
            riff_header,
            fmt_header,
            data_chunks,
        })
    }

    /// Read the RIFF header from a PCM WAV file
    /// 
    /// The RIFF header is the first twelve bytes of a PCM WAV
    /// file of the format `<RIFF_magic_str:4B><file_size:4B><RIFF_type_magic_str:4B>`.
    /// Note that the file handle `fh` should point to the very start of the file.
    /// 
    /// # Errors
    /// Returns a `WaveReaderError` with the appropriate error if something
    /// happens. This includes file read errors and format errors.
    fn read_riff_chunk(fh: &mut File) -> Result <RiffChunk, WaveReaderError> {
        let mut riff_id = [0u8; 4];
        fh.read_exact(&mut riff_id)?;
        if &riff_id != b"RIFF" && &riff_id != b"RIFX" {
            return Err(WaveReaderError::NotRiffError);
        }

        let mut buffer = [0u8; 4];
        fh.read_exact(&mut buffer)?;
        let file_size = if &riff_id == b"RIFF" {
            (&buffer[..]).read_u32::<LittleEndian>()?
        } else {
            (&buffer[..]).read_u32::<BigEndian>()?
        };
        
        
        let mut wave_id = [0u8; 4];
        fh.read_exact(&mut wave_id)?;
        if &wave_id != b"WAVE" {
            return Err(WaveReaderError::NotWaveError);
        }

        Ok(RiffChunk {
            file_size,
            is_big_endian: &riff_id == b"RIFX",
        })
    }

    /// Read the format chunk from a PCM WAV file
    /// 
    /// The format chunk usually appears immediately after the RIFF header and consists of 24 bytes of metadata.
    /// Note that the file handle `fh` should point to the start of a format chunk.
    /// 
    /// # Errors
    /// Returns a `WaveReaderError` with the appropriate error if something
    /// happens. This includes file read errors and format errors.
    fn read_fmt_chunk(fh: &mut File) -> Result <PCMWaveFormatChunk, WaveReaderError> {
        let mut fmt_id = [0u8; 4];
        fh.read_exact(&mut fmt_id)?;
        if &fmt_id != b"fmt " {
            return Err(WaveReaderError::ChunkTypeError);
        }

        let mut buffer = [0u8; 4];
        fh.read_exact(&mut buffer)?;
        let _fmt_size = (&buffer[..]).read_u32::<LittleEndian>()?;

        let mut buffer = [0u8; 2];
        fh.read_exact(&mut buffer)?;
        let audio_format = (&buffer[..]).read_u16::<LittleEndian>()?;
        if audio_format != 1 {
            return Err(WaveReaderError::NotPCMError);
        }

        let mut buffer = [0u8; 2];
        fh.read_exact(&mut buffer)?;
        let num_channels = (&buffer[..]).read_u16::<LittleEndian>()?;

        let mut buffer = [0u8; 4];
        fh.read_exact(&mut buffer)?;
        let samp_rate = (&buffer[..]).read_u32::<LittleEndian>()?;

        let mut buffer = [0u8; 4];
        fh.read_exact(&mut buffer)?;
        let byte_rate = (&buffer[..]).read_u32::<LittleEndian>()?;

        let mut buffer = [0u8; 2];
        fh.read_exact(&mut buffer)?;
        let block_align = (&buffer[..]).read_u16::<LittleEndian>()?;

        let mut buffer = [0u8; 2];
        fh.read_exact(&mut buffer)?;
        let bps = (&buffer[..]).read_u16::<LittleEndian>()?;

        let fmt_chunk = PCMWaveFormatChunk {num_channels, samp_rate, bps};

        if byte_rate != fmt_chunk.byte_rate() {
            return Err(WaveReaderError::DataAlignmentError);
        }
        if block_align != fmt_chunk.block_align() {
            return Err(WaveReaderError::DataAlignmentError);
        }
        Ok(fmt_chunk)
    }

    /// Read the data chunk from a PCM WAV file
    /// 
    /// The data chunk usually appears immediately after the format
    /// chunk and contains the samples of the audio itself. Note that
    /// a file can contain multiple data chunks, and it is possible that this
    /// method should be called more than once to completely read the file.
    /// Note that the file handle `fh` should point to the start of a data chunk.
    /// 
    /// # Errors
    /// Returns a `WaveReaderError` with the appropriate error if something
    /// happens. This includes file read errors and format errors.
    fn read_data_chunk(start_pos: u64, fmt_info: &PCMWaveFormatChunk, mut fh: File) -> Result <PCMWaveDataChunk, WaveReaderError> {
        fh.seek(SeekFrom::Start(start_pos))?;
        
        let mut data_id = [0u8; 4];
        fh.read_exact(&mut data_id)?;
        if &data_id != b"data" {
            return Err(WaveReaderError::ChunkTypeError);
        }

        let mut buffer = [0u8; 4];
        fh.read_exact(&mut buffer)?;
        let size_bytes = (&buffer[..]).read_u32::<LittleEndian>()?;

        // // Print the remaining contents of the file
        // let mut remaining_contents = Vec::new();
        // fh.read_to_end(&mut remaining_contents)?;
        // println!("Remaining contents: {:?}", remaining_contents);
    
        // Seek back to the start position
        fh.seek(SeekFrom::Start(start_pos + 4))?; // Adjust the seek position based on your file format

        let data_buf = io::BufReader::new(fh);

        Ok(PCMWaveDataChunk {
            size_bytes,
            format: *fmt_info,
            data_buf,
        })
    }
}

impl error::Error for WaveReaderError {}

impl fmt::Display for WaveReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaveReaderError::NotRiffError => write!(f, "Not a RIFF file"),
            WaveReaderError::NotWaveError => write!(f, "Not a WAVE file"),
            WaveReaderError::NotPCMError => write!(f, "Not a PCM format"),
            WaveReaderError::ChunkTypeError => write!(f, "Unexpected chunk type"),
            WaveReaderError::DataAlignmentError => write!(f, "Data alignment error"),
            WaveReaderError::ReadError => write!(f, "Error reading file"),
        }
    }
}

impl From <io::Error> for WaveReaderError {
    fn from(_: io::Error) -> Self {
        WaveReaderError::ReadError
    }
}

impl fmt::Display for PCMWaveInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WAVE File {} bytes, {}-bit {} channels, {}Hz, {} data chunks",
               self.riff_header.file_size,
               self.fmt_header.bps,
               self.fmt_header.num_channels,
               self.fmt_header.samp_rate,
               self.data_chunks.len())
    }
}

impl PCMWaveFormatChunk {
    /// Get or calculate the byte rate of this PCM WAV file
    fn byte_rate(&self) -> u32 {
        self.samp_rate as u32 * self.num_channels as u32 * self.bps as u32 / 8
    }

    /// Get or calculate the block alignment of this PCM WAV file
    /// 
    /// The *block alignment* is the size of one *inter-channel* sample
    /// in bytes. An *inter-channel sample* is a sample with all of its
    /// channels collated together.
    fn block_align(&self) -> u16 {
        self.num_channels as u16 * self.bps as u16 / 8
    }
}

impl Iterator for PCMWaveDataChunk {
    type Item = Vec <i64>;

    fn next(&mut self) -> Option <Self::Item> {
        let mut sample = vec![0; self.format.num_channels as usize];
        for i in 0..self.format.num_channels {
            match self.format.bps {
                8 => {
                    sample[i as usize] = self.data_buf.read_u8().ok()? as i64;
                }
                16 => {
                    sample[i as usize] = self.data_buf.read_i16::<LittleEndian>().ok()? as i64;
                }
                24 => {
                    let bytes = [
                        self.data_buf.read_u8().ok()?,
                        self.data_buf.read_u8().ok()?,
                        self.data_buf.read_u8().ok()?,
                    ];
                    sample[i as usize] = LittleEndian::read_i24(&bytes) as i64;
                }
                _ => return None,
            }
            // // Print the value that was just appended
            // println!("Appended value: {}", sample[i as usize]);
        }
            Some(sample)
    }
    
}


impl Iterator for PCMWaveDataChunkWindow {
    type Item = Vec <Vec <i64>>;

    fn next(&mut self) -> Option <Self::Item> {
        let mut samples = Vec::with_capacity(self.chunk_size);
        for _ in 0..self.chunk_size {
            if let Some(sample) = self.data_chunk.next() {
                samples.push(sample);
            } else {
                break;
            }
        }
        if samples.is_empty() {
            None
        } else {
            Some(samples)
        }
    }
}


impl PCMWaveDataChunk {
    /// Consume a data chunk and get an iterator
    /// 
    /// This method is used to get a *single* inter-channel
    /// sample from a data chunk.
    pub fn chunks_byte_rate(self) -> PCMWaveDataChunkWindow {
        PCMWaveDataChunkWindow {
            chunk_size: self.format.byte_rate() as usize,
            data_chunk: self,
        }
    }

    /// Consume a data chunk and get an iterator
    /// 
    /// This method is used to get a `chunk_size` amount of inter-channel
    /// samples. For example, if there are two channels and the chunk size is
    /// 44100 corresponding to a sample rate of 44100 Hz, then the iterator will
    /// return a `Vec` of size *at most* 44100 with each element as another `Vec`
    /// of size 2.
    pub fn chunks(self, chunk_size: usize) -> PCMWaveDataChunkWindow {
        PCMWaveDataChunkWindow {
            chunk_size,
            data_chunk: self,
        }
    }
}

// TODO: Add more tests here!
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod read_riff {
        use super::*;
        use std::io::Write;

        fn create_temp_file(file_name: &str, content: &[u8]) -> Result <(), io::Error> {
            let mut file = File::create(file_name)?;
            file.write_all(content)?;

            Ok(())
        }
        
        macro_rules! internal_tests {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() -> Result <(), WaveReaderError> {
                    let (input, (will_panic, expected)) = $value;

                    let file_name = format!("midp_{}.wav.part", stringify!($name));
                    let result;
                    {
                        create_temp_file(&file_name, input)?;
                        let mut input_fh = File::open(&file_name)?;
                        result = WaveReader::read_riff_chunk(&mut input_fh);
                    }
                    std::fs::remove_file(&file_name)?;

                    if will_panic {
                        assert!(result.is_err());
                    }
                    else if let Ok(safe_result) = result {
                        assert_eq!(expected.file_size, safe_result.file_size);
                        assert_eq!(expected.is_big_endian, safe_result.is_big_endian);
                    }
                    else {
                        result?;
                    }

                    Ok(())
                }
            )*
            }
        }
        
        internal_tests! {
            it_valid_le_00: (
                &[0x52, 0x49, 0x46, 0x46, 0x0, 0x0, 0x0, 0x0, 0x57, 0x41, 0x56, 0x45],
                (
                    false,
                    RiffChunk {
                        file_size: 0,
                        is_big_endian: false,
                    },
                )),
            it_valid_le_01: (
                &[0x52, 0x49, 0x46, 0x46, 0x80, 0x0, 0x0, 0x0, 0x57, 0x41, 0x56, 0x45],
                (
                    false,
                    RiffChunk {
                        file_size: 128,
                        is_big_endian: false,
                    },
                )),
            it_valid_le_02: (
                &[0x52, 0x49, 0x46, 0x46, 0x1C, 0x40, 0x36, 0x0, 0x57, 0x41, 0x56, 0x45],
                (
                    false,
                    RiffChunk {
                        file_size: 3_555_356,
                        is_big_endian: false,
                    },
                )),
            it_valid_be_00: (
                &[0x52, 0x49, 0x46, 0x58, 0x0, 0x0, 0x0, 0x0, 0x57, 0x41, 0x56, 0x45],
                (
                    false,
                    RiffChunk {
                        file_size: 0,
                        is_big_endian: true,
                    },
                )),
            it_valid_be_01: (
                &[0x52, 0x49, 0x46, 0x58, 0x00, 0x0, 0x0, 0x80, 0x57, 0x41, 0x56, 0x45],
                (
                    false,
                    RiffChunk {
                        file_size: 128,
                        is_big_endian: true,
                    },
                )),
            it_valid_be_02: (
                &[0x52, 0x49, 0x46, 0x58, 0x00, 0x36, 0x40, 0x1C, 0x57, 0x41, 0x56, 0x45],
                (
                    false,
                    RiffChunk {
                        file_size: 3_555_356,
                        is_big_endian: true,
                    },
                )),
            it_bad_riff: (
                &[0x00, 0x49, 0x46, 0x46, 0x00, 0x36, 0x40, 0x1C, 0x57, 0x41, 0x56, 0x45],
                (
                    true,
                    RiffChunk {
                        file_size: 0,
                        is_big_endian: false,
                    },
                )),
            it_bad_wave: (
                &[0x52, 0x49, 0x46, 0x46, 0x00, 0x36, 0x40, 0x1C, 0x57, 0x41, 0x56, 0x00],
                (
                    true,
                    RiffChunk {
                        file_size: 0,
                        is_big_endian: false,
                    },
                )),
        }
    }

    #[cfg(test)]
    mod read_wav_fmt {
        use super::*;
        use std::io::Write;

        fn create_temp_file(file_name: &str, content: &[u8]) -> Result <(), io::Error> {
            let mut file = File::create(file_name)?;
            file.write_all(content)?;

            Ok(())
        }
        
        macro_rules! internal_tests {
            ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() -> Result <(), WaveReaderError> {
                    let (input, (will_panic, expected)) = $value;

                    let file_name = format!("midp_{}.wav.part", stringify!($name));
                    let result;
                    {
                        create_temp_file(&file_name, input)?;
                        let mut input_fh = File::open(&file_name)?;
                        result = WaveReader::read_fmt_chunk(&mut input_fh);
                    }
                    std::fs::remove_file(&file_name)?;

                    if will_panic {
                        assert!(result.is_err());
                    }
                    else if let Ok(safe_result) = result {
                        assert_eq!(expected.num_channels, safe_result.num_channels);
                        assert_eq!(expected.samp_rate, safe_result.samp_rate);
                        assert_eq!(expected.bps, safe_result.bps);
                    }
                    else {
                        result?;
                    }

                    Ok(())
                }
            )*
            }
        }
        
        internal_tests! {
            it_valid_00: (
                &[
                    0x66, 0x6d, 0x74, 0x20,
                    0x10, 0x0, 0x0, 0x0,
                    0x01, 0x0,
                    0x01, 0x0,
                    0x44, 0xac, 0x0, 0x0,
                    0x44, 0xac, 0x0, 0x0,
                    0x01, 0x00, 0x08, 0x0,
                ],
                (
                    false,
                    PCMWaveFormatChunk {
                        num_channels: 1,
                        samp_rate: 44100,
                        bps: 8,
                    },
                )),
            it_valid_01: (
                &[
                    0x66, 0x6d, 0x74, 0x20,
                    0x10, 0x0, 0x0, 0x0,
                    0x01, 0x0,
                    0x02, 0x0,
                    0x44, 0xac, 0x0, 0x0,
                    0x88, 0x58, 0x01, 0x0,
                    0x02, 0x00, 0x08, 0x0,
                ],
                (
                    false,
                    PCMWaveFormatChunk {
                        num_channels: 2,
                        samp_rate: 44100,
                        bps: 8,
                    },
                )),
            it_valid_02: (
                &[
                    0x66, 0x6d, 0x74, 0x20,
                    0x10, 0x0, 0x0, 0x0,
                    0x01, 0x0,
                    0x02, 0x0,
                    0x44, 0xac, 0x0, 0x0,
                    0x10, 0xb1, 0x02, 0x0,
                    0x04, 0x00, 0x10, 0x0,
                ],
                (
                    false,
                    PCMWaveFormatChunk {
                        num_channels: 2,
                        samp_rate: 44100,
                        bps: 16,
                    },
                )),
            it_invalid_badfmt: (
                &[
                    0x00, 0x6d, 0x74, 0x20,
                    0x10, 0x0, 0x0, 0x0,
                    0x01, 0x0,
                    0x02, 0x0,
                    0x44, 0xac, 0x0, 0x0,
                    0x10, 0xb1, 0x02, 0x0,
                    0x04, 0x00, 0x10, 0x0,
                ],
                (
                    true,
                    PCMWaveFormatChunk {
                        num_channels: 2,
                        samp_rate: 44100,
                        bps: 16,
                    },
                )),    
        }
    }
    #[cfg(test)]
    mod byte_rate_comp{
        use super::*;
        #[test]
        fn it_works() {
            let samp_1 = PCMWaveFormatChunk{
                num_channels: 1,
                samp_rate: 44100,
                bps: 16,
            };
            let samp_2 = PCMWaveFormatChunk {
                num_channels: 2,
                samp_rate: 32000,
                bps: 8,
            };
            let samp_3 = PCMWaveFormatChunk {
                num_channels: 1,
                samp_rate: 12000,
                bps: 4,
            };
            let res_1 = samp_1.byte_rate();
            let res_2 = samp_2.byte_rate();
            let res_3 = samp_3.byte_rate();

            assert_eq!(res_1, 88200 as u32);
            assert_eq!(res_2, 64000 as u32);
            assert_eq!(res_3, 6000 as u32);
        }
    }
    #[cfg(test)] 
    mod block_align_comp{
        use super::*;
        #[test]
        fn it_works() {
            let samp_1 = PCMWaveFormatChunk{
                num_channels: 1,
                samp_rate: 44100,
                bps: 16,
            };
            let samp_2 = PCMWaveFormatChunk {
                num_channels: 2,
                samp_rate: 32000,
                bps: 8,
            };
            let samp_3 = PCMWaveFormatChunk {
                num_channels: 2,
                samp_rate: 12000,
                bps: 4,
            };
            let res_1 = samp_1.block_align();
            let res_2 = samp_2.block_align();
            let res_3 = samp_3.block_align();

            assert_eq!(res_1, 2);
            assert_eq!(res_2, 2);
            assert_eq!(res_3, 1);
        }
    }

    mod read_data_fmt {
        // TODO
    }
}
