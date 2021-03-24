use std::io::{Cursor, Write};
use std::time::Instant;

/// Buffer compression output to this chunk size.
const ENCODING_CHUNK_SIZE: usize = 8192;

/// Default LZMA dictionary memory limit in bytes.
const DEFAULT_LZMA_MEMLIMIT: usize = 1_048_576;
/// Default number of LZMA layers to pass to the decompressor.
const DEFAULT_LZMA_LAYERS: u32 = 1;
/// Default max output size for a compression bomb.
const DEFAULT_BOMB_LIMIT: i32 = 1_048_576;
/// Upper limit to max output size for a compression bomb.
const MAX_BOMB_LIMIT: i32 = std::i32::MAX;
/// Default compressed-to-decrompressed ratio that should not be exceeded during decompression.
const DEFAULT_BOMB_RATIO: i64 = 2048;
/// Default time limit for a decompression bomb in microseconds.
const DEFAULT_TIME_LIMIT: u32 = 100_000;
/// Default number of iterations before checking the time limit.
const DEFAULT_TIME_FREQ_TEST: u32 = 256;
/// Default number of layers that will be decompressed
const DEFAULT_LAYER_LIMIT: usize = 2;

#[derive(Copy, Clone)]
/// Decompression options
pub struct Options {
    /// lzma options or None to disable lzma.
    lzma: Option<lzma_rs::decompress::Options>,
    // TODO: implement lzma layers check
    /// number of LZMA layers to pass to the decompressor.
    lzma_layers: u32,
    /// max output size for a compression bomb.
    bomb_limit: i32,
    /// max compressed-to-decrompressed ratio that should not be exceeded during decompression.
    bomb_ratio: i64,
    /// max time for a decompression bomb in microseconds.
    time_limit: u32,
    /// number of iterations to before checking the time_limit.
    time_test_freq: u32,
    /// number of layers of compression we will decompress
    layer_limit: Option<usize>,
}

impl Options {
    /// Get the lzma memlimit.
    ///
    /// A value of 0 indicates that lzma is disabled.
    pub fn get_lzma_memlimit(&self) -> usize {
        if let Some(options) = self.lzma {
            if let Some(memlimit) = options.memlimit {
                memlimit
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Set the lzma memlimit.
    ///
    /// A value of 0 will disable lzma.
    pub fn set_lzma_memlimit(&mut self, memlimit: usize) {
        self.lzma = if memlimit == 0 {
            None
        } else {
            Some(lzma_rs::decompress::Options {
                memlimit: Some(memlimit),
                ..Default::default()
            })
        }
    }

    /// Configures the maximum layers passed to lzma-rs.
    pub fn set_lzma_layers(&mut self, layers: u32) {
        self.lzma_layers = layers;
    }

    /// Get the compression bomb limit.
    pub fn get_bomb_limit(&self) -> i32 {
        self.bomb_limit
    }

    /// Set the compression bomb limit.
    ///
    /// The limit will be set to `MAX_BOMB_LIMIT` if the provided arg exceeds this value.
    pub fn set_bomb_limit(&mut self, bomblimit: usize) {
        if bomblimit > MAX_BOMB_LIMIT as usize {
            self.bomb_limit = MAX_BOMB_LIMIT as i32;
        } else {
            self.bomb_limit = bomblimit as i32
        };
    }

    /// Get the bomb ratio.
    pub fn get_bomb_ratio(&self) -> i64 {
        self.bomb_ratio
    }

    /// Set the bomb ratio.
    pub fn set_bomb_ratio(&mut self, bomb_ratio: i64) {
        self.bomb_ratio = bomb_ratio;
    }

    /// Get the compression time limit in microseconds.
    pub fn get_time_limit(&self) -> u32 {
        self.time_limit
    }

    /// Set the compression time limit in microseconds.
    pub fn set_time_limit(&mut self, time_limit: u32) {
        self.time_limit = time_limit
    }

    /// Get the time test frequency.
    pub fn get_time_test_freq(&self) -> u32 {
        self.time_test_freq
    }

    /// Set the time test frequency.
    pub fn set_time_test_freq(&mut self, time_test_freq: u32) {
        self.time_test_freq = time_test_freq;
    }

    /// Get the decompression layer limit.
    pub fn get_layer_limit(&self) -> Option<usize> {
        self.layer_limit
    }

    /// Set the decompression layer limit.
    pub fn set_layer_limit(&mut self, layer_limit: Option<usize>) {
        self.layer_limit = layer_limit;
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            lzma: Some(lzma_rs::decompress::Options {
                memlimit: Some(DEFAULT_LZMA_MEMLIMIT),
                ..Default::default()
            }),
            lzma_layers: DEFAULT_LZMA_LAYERS,
            bomb_limit: DEFAULT_BOMB_LIMIT,
            bomb_ratio: DEFAULT_BOMB_RATIO,
            time_limit: DEFAULT_TIME_LIMIT,
            time_test_freq: DEFAULT_TIME_FREQ_TEST,
            layer_limit: Some(DEFAULT_LAYER_LIMIT),
        }
    }
}

/// Describes a decompressor that is able to restart and passthrough data.
/// Actual decompression is done using the `Write` trait.
pub trait Decompress: Write {
    /// Restarts the decompressor to try the same one again or a different one.
    fn restart(&mut self) -> std::io::Result<()>;

    /// Tells all decompressors to passthrough their data instead of
    /// decompressing to directly call the callback
    fn set_passthrough(&mut self, passthrough: bool);

    /// Indicates that we have reached the end of data. This would be equivalent
    /// to sending a NULL pointer in C and may be used by the hooks.
    fn finish(&mut self) -> std::io::Result<()>;
}

/// Type alias for callback function.
pub type CallbackFn = Box<dyn FnMut(Option<&[u8]>) -> Result<usize, std::io::Error>>;

/// Simple wrapper around a closure to chain it to the other decompressors
pub struct CallbackWriter(CallbackFn);

impl CallbackWriter {
    /// Create a new CallbackWriter.
    pub fn new(cbk: CallbackFn) -> Self {
        CallbackWriter(cbk)
    }
}

impl Write for CallbackWriter {
    fn write(&mut self, data: &[u8]) -> std::result::Result<usize, std::io::Error> {
        (self.0)(Some(data))
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}

impl Decompress for CallbackWriter {
    fn restart(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn set_passthrough(&mut self, _passthrough: bool) {}

    fn finish(&mut self) -> std::io::Result<()> {
        (self.0)(None)?;
        Ok(())
    }
}

/// Type of compression.
/// cbindgen:rename-all=QualifiedScreamingSnakeCase
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum HtpContentEncoding {
    /// No compression.
    NONE,
    /// Gzip compression.
    GZIP,
    /// Deflate compression (RFC 1951).
    DEFLATE,
    /// Deflate compression with zlib header (RFC 1950)
    ZLIB,
    /// LZMA compression.
    LZMA,
    /// Error retrieving the content encoding.
    ERROR,
}

/// The outer decompressor tracks the number of callbacks and time spent
/// decompressing.
pub struct Decompressor {
    /// First decompressor to call
    inner: Box<dyn Decompress>,
    /// Time we started decompression
    time_before: Option<Instant>,
    /// Time spent decompressing so far in microseconds (usec)
    time_spent: u64,
    /// Number of times the callback was called
    nb_callbacks: u32,
}

impl Decompressor {
    /// Creates a new decompressor from a struct implementing the Decompress trait.
    fn new(inner: Box<dyn Decompress>) -> Self {
        Self {
            inner,
            time_before: None,
            time_spent: 0,
            nb_callbacks: 0,
        }
    }

    /// Creates a new decompressor from a callback to call when decompressed
    /// data is ready.
    fn callback(callback: CallbackFn) -> Self {
        Self::new(Box::new(CallbackWriter::new(callback)))
    }

    /// Prepends a decompressor to this chain by consuming `self.inner`
    /// and creating a new Decompressor.
    ///
    /// Note that decompressors should be added in the same order the data was
    /// compressed, starting with the callback.
    ///
    /// ```
    /// use htp::decompressors::{HtpContentEncoding, Decompressor};
    ///
    /// // Example for "Content-Encoding: gzip, deflate"
    /// let mut decompressor = Decompressor::new_with_callback(HtpContentEncoding::GZIP,
    ///     Box::new(|data: Option<&[u8]>| -> Result<usize, std::io::Error> {
    ///         if let Some(data) = data {
    ///             println!("CALLBACK: {}", data.len());
    ///             Ok(data.len())
    ///         } else {
    ///             println!("CALLBACK: end of data");
    ///             Ok(0)
    ///         }
    ///     }), Default::default()).unwrap();
    ///
    /// decompressor = decompressor.prepend(HtpContentEncoding::DEFLATE, Default::default()).unwrap();
    ///
    /// // Decompressors will be called in this order:
    /// // 1. deflate
    /// // 2. gzip
    /// // 3. callback
    /// decompressor.decompress(&[]).unwrap();
    /// ```
    pub fn prepend(self, encoding: HtpContentEncoding, options: Options) -> std::io::Result<Self> {
        match encoding {
            HtpContentEncoding::NONE => Ok(Decompressor::new(self.inner)),
            HtpContentEncoding::GZIP
            | HtpContentEncoding::DEFLATE
            | HtpContentEncoding::ZLIB
            | HtpContentEncoding::LZMA => Ok(Decompressor::new(Box::new(InnerDecompressor::new(
                encoding, self.inner, options,
            )?))),
            HtpContentEncoding::ERROR => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "expected a valid encoding",
            )),
        }
    }

    /// Creates a new decompressor with `encoding` and adds a callback to be called
    /// when data is ready.
    pub fn new_with_callback(
        encoding: HtpContentEncoding,
        callback: CallbackFn,
        options: Options,
    ) -> std::io::Result<Self> {
        Self::callback(callback).prepend(encoding, options)
    }

    /// Starts the decompression timer.
    fn timer_start(&mut self) {
        self.time_before.replace(Instant::now());
    }

    /// Stops the decompression timer, updates and returns the time spent
    /// decompressing in microseconds (usec).
    pub fn timer_reset(&mut self) -> Option<u64> {
        let now = Instant::now();
        if let Some(time_before) = self.time_before.replace(now) {
            // it is unlikely that more than 2^64 will be spent on a single stream
            self.time_spent += now.duration_since(time_before).as_micros() as u64;
            Some(self.time_spent)
        } else {
            None
        }
    }

    /// Increments the number of times the callback was called.
    pub fn callback_inc(&mut self) -> u32 {
        self.nb_callbacks = self.nb_callbacks.wrapping_add(1);
        self.nb_callbacks
    }

    /// Returns the time spent decompressing in microseconds (usec).
    pub fn time_spent(&self) -> u64 {
        self.time_spent
    }

    /// Decompress the input `data` by calling the chain of decompressors and
    /// the data callback.
    ///
    /// This will reset the number of callbacks called and restart the
    /// decompression timer.
    pub fn decompress(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.nb_callbacks = 0;
        self.timer_start();

        let result = self.inner.write_all(data).and_then(|_| self.inner.flush());

        self.timer_reset();
        result
    }

    /// Notify decompressors that the end of stream as reached. This is equivalent
    /// to sending a NULL data pointer.
    pub fn finish(&mut self) -> std::io::Result<()> {
        self.inner.finish()
    }
}

impl std::fmt::Debug for Decompressor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Decompressor")
            .field("time_spent", &self.time_spent)
            .field("nb_callbacks", &self.nb_callbacks)
            .finish()
    }
}

/// Trait that represents the decompression writers (gzip, deflate, etc.) and
/// methods needed to write to a temporary buffer.
pub trait BufWriter: Write {
    /// Get a mutable reference to the buffer.
    fn get_mut(&mut self) -> Option<&mut Cursor<Box<[u8]>>>;
    /// Notify end of data.
    fn finish(self: Box<Self>) -> std::io::Result<Cursor<Box<[u8]>>>;
}

/// A BufWriter that doesn't consume any data.
///
/// This should be used exclusively with passthrough mode.
struct NullBufWriter(Cursor<Box<[u8]>>);

impl Write for NullBufWriter {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl BufWriter for NullBufWriter {
    fn get_mut(&mut self) -> Option<&mut Cursor<Box<[u8]>>> {
        Some(&mut self.0)
    }

    fn finish(self: Box<Self>) -> std::io::Result<Cursor<Box<[u8]>>> {
        Ok(self.0)
    }
}

/// Wrapper around a gzip header parser and a deflate decoder.
/// We parse the header separately because we want to be tolerant of
/// checksum or other gzip errors that do not affect our ability
/// to decompress the data stream but would cause 'correct' gzip decoders
/// to fail. We want to be tolerant of gzip errors because browsers
/// are apparently tolerant of gzip errors
///
/// https://noxxi.de/research/http-evader-explained-5-gzip.html
struct GzipBufWriter {
    buffer: Vec<u8>,
    header: Option<GzHeader>,
    inner: flate2::write::DeflateDecoder<Cursor<Box<[u8]>>>,
}

/// A structure holding a Gzip header
#[derive(PartialEq, Clone, Debug, Default)]
pub struct GzHeader {
    extra: Option<Vec<u8>>,
    filename: Option<Vec<u8>>,
    comment: Option<Vec<u8>>,
    operating_system: u8,
    mtime: i32,
    crc: Option<u16>,
    flags: u8,
    xfl: u8,
}

impl GzHeader {
    const FHCRC: u8 = 1 << 1;
    const FEXTRA: u8 = 1 << 2;
    const FNAME: u8 = 1 << 3;
    const FCOMMENT: u8 = 1 << 4;

    fn parse(data: &[u8]) -> nom::IResult<&[u8], Self> {
        use nom::bytes::streaming::{tag, take, take_until};
        use nom::number::streaming::{le_i32, le_u16, le_u8};
        use nom::sequence::tuple;
        let rest: &[u8] = data;
        let (rest, (_, flags, mtime, xfl, operating_system)) =
            tuple((tag(b"\x1f\x8b\x08"), le_u8, le_i32, le_u8, le_u8))(rest)?;

        let (rest, extra) = match flags & Self::FEXTRA {
            0 => (rest, None),
            _ => {
                let (rest, len) = le_u16(rest)?;
                let (rest, extra) = take(len as usize)(rest)?;
                (rest, Some(extra.into()))
            }
        };

        let (rest, filename) = match flags & Self::FNAME {
            0 => (rest, None),
            _ => {
                let (rest, (filename, _)) = tuple((take_until(b"\0" as &[u8]), tag(b"\0")))(rest)?;
                (rest, Some(filename.into()))
            }
        };

        let (rest, comment) = match flags & Self::FCOMMENT {
            0 => (rest, None),
            _ => {
                let (rest, (comment, _)) = tuple((take_until(b"\0" as &[u8]), tag(b"\0")))(rest)?;
                (rest, Some(comment.into()))
            }
        };

        let (rest, crc) = match flags & Self::FHCRC {
            0 => (rest, None),
            _ => {
                let (rest, crc) = le_u16(rest)?;
                (rest, Some(crc))
            }
        };

        Ok((
            rest,
            GzHeader {
                extra,
                filename,
                comment,
                operating_system,
                mtime,
                crc,
                flags,
                xfl,
            },
        ))
    }
}

impl GzipBufWriter {
    fn new(buf: Cursor<Box<[u8]>>) -> Self {
        GzipBufWriter {
            buffer: Vec::with_capacity(10),
            header: None,
            inner: flate2::write::DeflateDecoder::new(buf),
        }
    }

    fn parse_gz_header(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let parse = if !self.buffer.is_empty() {
            self.buffer.extend_from_slice(data);
            self.buffer.as_ref()
        } else {
            data
        };

        match GzHeader::parse(parse) {
            Ok((rest, header)) => {
                self.header = Some(header);
                if let Some(readlen) = data.len().checked_sub(rest.len()) {
                    Ok(readlen)
                } else {
                    // If we got here, it means we could have parsed
                    // the header out of the stored buffer alone, which
                    // we should have done before we stored it.
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unexpected remaining data",
                    ))
                }
            }
            Err(nom::Err::Incomplete(_)) => {
                // cache for later
                self.buffer.extend_from_slice(data);
                Ok(data.len())
            }
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Could not parse gzip header",
            )),
        }
    }
}

impl Write for GzipBufWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        if self.header.is_none() {
            self.parse_gz_header(data)
        } else {
            self.inner.write(data)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl BufWriter for GzipBufWriter {
    fn get_mut(&mut self) -> Option<&mut Cursor<Box<[u8]>>> {
        Some(self.inner.get_mut())
    }

    fn finish(self: Box<Self>) -> std::io::Result<Cursor<Box<[u8]>>> {
        self.inner.finish()
    }
}

/// Simple wrapper around a deflate implementation
struct DeflateBufWriter(flate2::write::DeflateDecoder<Cursor<Box<[u8]>>>);

impl Write for DeflateBufWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.write(data)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl BufWriter for DeflateBufWriter {
    fn get_mut(&mut self) -> Option<&mut Cursor<Box<[u8]>>> {
        Some(self.0.get_mut())
    }

    fn finish(self: Box<Self>) -> std::io::Result<Cursor<Box<[u8]>>> {
        self.0.finish()
    }
}

/// Simple wrapper around a zlib implementation
struct ZlibBufWriter(flate2::write::ZlibDecoder<Cursor<Box<[u8]>>>);

impl Write for ZlibBufWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.write(data)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl BufWriter for ZlibBufWriter {
    fn get_mut(&mut self) -> Option<&mut Cursor<Box<[u8]>>> {
        Some(self.0.get_mut())
    }

    fn finish(self: Box<Self>) -> std::io::Result<Cursor<Box<[u8]>>> {
        self.0.finish()
    }
}

/// Simple wrapper around an lzma implementation
struct LzmaBufWriter(lzma_rs::decompress::Stream<Cursor<Box<[u8]>>>);

impl Write for LzmaBufWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.write(data)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl BufWriter for LzmaBufWriter {
    fn get_mut(&mut self) -> Option<&mut Cursor<Box<[u8]>>> {
        self.0.get_output_mut()
    }

    fn finish(self: Box<Self>) -> std::io::Result<Cursor<Box<[u8]>>> {
        self.0.finish().map_err(|e| match e {
            lzma_rs::error::Error::IOError(e) => e,
            lzma_rs::error::Error::HeaderTooShort(e) => {
                std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))
            }
            lzma_rs::error::Error::LZMAError(e) | lzma_rs::error::Error::XZError(e) => {
                std::io::Error::new(std::io::ErrorKind::Other, e)
            }
        })
    }
}

/// Structure that represents each decompressor in the chain.
struct InnerDecompressor {
    /// Decoder implementation that will write to a temporary buffer.
    writer: Option<Box<dyn BufWriter>>,
    /// Next decompressor to call.
    inner: Option<Box<dyn Decompress>>,
    /// Encoding type of the decompressor.
    encoding: HtpContentEncoding,
    /// Next encoding to try when we fail to decompress
    next_encoding: HtpContentEncoding,
    /// Indicates whether to pass through the data without calling the writer.
    passthrough: bool,
    /// Tracks the number of restarts
    restarts: u8,
    /// Options for decompression
    options: Options,
}

impl InnerDecompressor {
    /// Returns a new writer according to the content encoding type and whether to passthrough.
    fn writer(
        encoding: HtpContentEncoding,
        options: &Options,
    ) -> std::io::Result<(Box<dyn BufWriter>, bool)> {
        let buf = Cursor::new(Box::new([0u8; ENCODING_CHUNK_SIZE]) as Box<[u8]>);

        match encoding {
            HtpContentEncoding::GZIP => Ok((Box::new(GzipBufWriter::new(buf)), false)),
            HtpContentEncoding::DEFLATE => Ok((
                Box::new(DeflateBufWriter(flate2::write::DeflateDecoder::new(buf))),
                false,
            )),
            HtpContentEncoding::ZLIB => Ok((
                Box::new(ZlibBufWriter(flate2::write::ZlibDecoder::new(buf))),
                false,
            )),
            HtpContentEncoding::LZMA => {
                if let Some(options) = options.lzma {
                    Ok((
                        Box::new(LzmaBufWriter(
                            lzma_rs::decompress::Stream::new_with_options(&options, buf),
                        )),
                        false,
                    ))
                } else {
                    Ok((Box::new(NullBufWriter(buf)), true))
                }
            }
            HtpContentEncoding::NONE | HtpContentEncoding::ERROR => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "expected a valid encoding",
            )),
        }
    }

    /// Create a new `InnerDecompressor` given a content encoding type and the
    /// next (`inner`) decompressor to call.
    fn new(
        encoding: HtpContentEncoding,
        inner: Box<dyn Decompress>,
        options: Options,
    ) -> std::io::Result<Self> {
        let (writer, passthrough) = Self::writer(encoding, &options)?;
        Ok(Self {
            inner: Some(inner),
            encoding,
            next_encoding: encoding,
            writer: Some(writer),
            passthrough,
            restarts: 0,
            options,
        })
    }

    /// Tries to pass data to the callback instead of calling the writers.
    ///
    /// This will set passthrough mode on success or revert on error.
    fn try_passthrough(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.set_passthrough(true);
        if let Some(inner) = &mut self.inner {
            let result = inner.write(data);
            if result.is_err() {
                self.set_passthrough(false);
            }
            result
        } else {
            Ok(data.len())
        }
    }

    /// Flushes the writer and the temporary buffer it writes to.
    ///
    /// The writer should be taken out of its slot and passed directly instead of
    /// `self.writer` to avoid holding multiple mutable references.
    fn flush_writer(&mut self, writer: &mut Box<dyn BufWriter>) -> std::io::Result<()> {
        if let Some(mut inner) = self.inner.take() {
            while {
                let result = writer.flush();

                // Flush all of the bytes the writer has written to our temporary
                // buffer of fixed size.
                if let Some(cursor) = writer.get_mut() {
                    inner.write_all(&cursor.get_ref()[0..cursor.position() as usize])?;
                    cursor.set_position(0);
                }

                // Continue flushing if the flush resulted in a `WriteZero`. This
                // error indicates that the writer was unable to write all bytes
                // to our temporary buffer, likely because it was full.
                if let Err(e) = result {
                    match e.kind() {
                        std::io::ErrorKind::WriteZero => true,
                        _ => {
                            self.restart()?;
                            false
                        }
                    }
                } else {
                    false
                }
            } {}
            self.inner.replace(inner);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "nothing to flush to",
            ))
        }
    }
}

impl Write for InnerDecompressor {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        // Passthrough mode
        if self.passthrough {
            if let Some(inner) = &mut self.inner {
                inner.write(data)
            } else {
                Ok(data.len())
            }

        // Take the writer out of its slot to avoid holding multiple mutable
        // references. Any calls using `self.writer` should be avoided while the
        // writer is in this state.
        } else if let Some(mut writer) = self.writer.take() {
            match writer.write(data) {
                Ok(consumed) => {
                    let result = if consumed == 0 {
                        // This could indicate that we have reached the end
                        // of the stream. Any data after the first end of
                        // stream (such as in multipart gzip) is ignored and
                        // we pretend to have consumed this data.
                        Ok(data.len())
                    } else {
                        Ok(consumed)
                    };
                    self.writer.replace(writer);
                    result
                }
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::WriteZero => {
                            self.flush_writer(&mut writer)?;
                            // Recursion: the buffer was flushed until `WriteZero`
                            // stopped occuring.
                            self.writer.replace(writer);
                            self.write(data)
                        }
                        _ => {
                            // try to restart, any data in the temp buffer will be
                            // discarded
                            if self.restart().is_err() {
                                self.try_passthrough(data)
                            } else {
                                // Recursion: restart will fail after a small
                                // number of attempts
                                self.write(data)
                            }
                        }
                    }
                }
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "writer was not initialized",
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(mut writer) = self.writer.take() {
            self.flush_writer(&mut writer)?;
            self.writer.replace(writer);
        }
        if let Some(inner) = &mut self.inner {
            inner.flush()
        } else {
            Ok(())
        }
    }
}

impl Decompress for InnerDecompressor {
    fn restart(&mut self) -> std::io::Result<()> {
        if self.restarts < 3 {
            // first retry the same encoding type
            self.next_encoding = if self.restarts == 0 {
                self.encoding
            } else {
                // if that still fails, try the other method we support
                match self.next_encoding {
                    HtpContentEncoding::GZIP => HtpContentEncoding::DEFLATE,
                    HtpContentEncoding::DEFLATE => HtpContentEncoding::ZLIB,
                    HtpContentEncoding::ZLIB => HtpContentEncoding::GZIP,
                    HtpContentEncoding::LZMA => HtpContentEncoding::DEFLATE,
                    HtpContentEncoding::NONE | HtpContentEncoding::ERROR => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "expected a valid encoding",
                        ))
                    }
                }
            };
            let (writer, passthrough) = Self::writer(self.next_encoding, &self.options)?;
            self.writer = Some(writer);
            if passthrough {
                self.passthrough = passthrough;
            }
            self.restarts += 1;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "too many restart attempts",
            ))
        }
    }

    // Tell all the decompressors to pass through the data instead of calling
    // the writer.
    fn set_passthrough(&mut self, passthrough: bool) {
        self.passthrough = passthrough;
        if let Some(inner) = &mut self.inner {
            inner.set_passthrough(passthrough);
        }
    }

    // Tell all decompressors that there is no more data to receive.
    fn finish(&mut self) -> std::io::Result<()> {
        let output = if let Some(mut writer) = self.writer.take() {
            self.flush_writer(&mut writer)?;
            Some(writer.finish()?)
        } else {
            None
        };

        if let Some(mut inner) = self.inner.take() {
            if let Some(output) = output {
                inner.write_all(&output.get_ref()[..output.position() as usize])?;
            }
            inner.finish()
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_gz_header() {
    // No flags or other bits
    let input = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x00";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: None,
                filename: None,
                comment: None,
                operating_system: 0,
                mtime: 0,
                crc: None,
                flags: 0,
                xfl: 0,
            }
        ))
    );

    // Just CRC
    let input = b"\x1f\x8b\x08\x02\x00\x00\x00\x00\x00\x00\x11\x22";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: None,
                filename: None,
                comment: None,
                operating_system: 0,
                mtime: 0,
                crc: Some(0x2211),
                flags: 0b0000_0010,
                xfl: 0,
            }
        ))
    );

    // Just extra
    let input = b"\x1f\x8b\x08\x04\x00\x00\x00\x00\x00\x00\x04\x00abcd";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: Some(b"abcd".to_vec()),
                filename: None,
                comment: None,
                operating_system: 0,
                mtime: 0,
                crc: None,
                flags: 0b0000_0100,
                xfl: 0,
            }
        ))
    );

    // Just filename
    let input = b"\x1f\x8b\x08\x08\x00\x00\x00\x00\x00\x00variable\x00";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: None,
                filename: Some(b"variable".to_vec()),
                comment: None,
                operating_system: 0,
                mtime: 0,
                crc: None,
                flags: 0b0000_1000,
                xfl: 0,
            }
        ))
    );

    // Just comment
    let input = b"\x1f\x8b\x08\x10\x00\x00\x00\x00\x00\x00also variable\x00";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: None,
                filename: None,
                comment: Some(b"also variable".to_vec()),
                operating_system: 0,
                mtime: 0,
                crc: None,
                flags: 0b0001_0000,
                xfl: 0,
            }
        ))
    );

    // Extra and Filename
    let input = b"\x1f\x8b\x08\x0c\x00\x00\x00\x00\x00\x00\x05\x00extrafilename\x00";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: Some(b"extra".to_vec()),
                filename: Some(b"filename".to_vec()),
                comment: None,
                operating_system: 0,
                mtime: 0,
                crc: None,
                flags: 0b0000_1100,
                xfl: 0,
            }
        ))
    );

    // Extra and Comment and CRC
    let input = b"\x1f\x8b\x08\x16\x00\x00\x00\x00\x00\x00\x05\x00extracomment\x00\x34\x12";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: Some(b"extra".to_vec()),
                filename: None,
                comment: Some(b"comment".to_vec()),
                operating_system: 0,
                mtime: 0,
                crc: Some(0x1234),
                flags: 0b0001_0110,
                xfl: 0,
            }
        ))
    );

    // Filename and Comment
    let input = b"\x1f\x8b\x08\x18\x00\x00\x00\x00\x00\x00filename\x00comment\x00";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: None,
                filename: Some(b"filename".to_vec()),
                comment: Some(b"comment".to_vec()),
                operating_system: 0,
                mtime: 0,
                crc: None,
                flags: 0b0001_1000,
                xfl: 0,
            }
        ))
    );

    // Extra Filename and Comment and CRC
    let input =
        b"\x1f\x8b\x08\x1e\x00\x00\x00\x00\x00\x00\x05\x00extrafilename\x00comment\x00\x34\x12";
    assert_eq!(
        GzHeader::parse(input),
        Ok((
            b"" as &[u8],
            GzHeader {
                extra: Some(b"extra".to_vec()),
                filename: Some(b"filename".to_vec()),
                comment: Some(b"comment".to_vec()),
                operating_system: 0,
                mtime: 0,
                crc: Some(0x1234),
                flags: 0b0001_1110,
                xfl: 0,
            }
        ))
    );

    // Too short
    let input = b"\x1f\x8b\x08\x1e\x00\x00\x00\x00\x00\x00\x05\x00extrafilename\x00comment\x00\x34";
    assert!(GzHeader::parse(input).is_err());
    let input = b"\x1f\x8b\x08\x01\x00\x00\x00\x00\x00";
    assert!(GzHeader::parse(input).is_err());
}
