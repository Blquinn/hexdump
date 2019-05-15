use std::io;
use std::io::{Write};
use std::fs;

static HEX_TABLE: &'static [u8] = b"0123456789abcdef";

static SPACE: u8 =   ' ' as u8;
static NEWLINE: u8 = '\n' as u8;
static PIPE: u8 =    '|' as u8;
static PERIOD: u8 =  '.' as u8;

pub enum Writers {
    Stdout(io::Stdout),
    File(fs::File),
    StdoutBuf(io::BufWriter<io::Stdout>),
    FileBuf(io::BufWriter<fs::File>),
}

pub struct HexDumper<'a> {
    n: usize, // number of bytes written total
    buf: [u8; 14],
    right_chars: [u8; 18],
    used: usize, // Number of bytes in the current line
    writer: &'a mut Writers,
    closed: bool,
}

impl<'a> HexDumper<'a> {
    pub fn new(writer: &'a mut Writers) -> HexDumper<'a> {
        HexDumper{
            n: 0,
            buf: [0; 14],
            right_chars: [0; 18],
            used: 0,
            closed: false,
            writer,
        }
    }
}

impl<'a> HexDumper<'a> {
    fn write_buf_slice(&mut self, lower: usize, upper: usize) -> io::Result<usize> {
        match self.writer {
            Writers::File(file) => file.write(&self.buf[lower..upper]),
            Writers::Stdout(stdout) => stdout.write(&self.buf[lower..upper]),
            Writers::FileBuf(file) => file.write(&self.buf[lower..upper]),
            Writers::StdoutBuf(stdout) => stdout.write(&self.buf[lower..upper]),
        }
    }

    fn write_right_chars(&mut self) -> io::Result<usize> {
        match self.writer {
            Writers::File(file) => file.write(&self.right_chars),
            Writers::Stdout(stdout) => stdout.write(&self.right_chars),
            Writers::FileBuf(file) => file.write(&self.right_chars),
            Writers::StdoutBuf(stdout) => stdout.write(&self.right_chars),
        }
    }

    fn write_right_chars_slice(&mut self, lower: usize, upper: usize) -> io::Result<usize> {
        match self.writer {
            Writers::File(file) => file.write(&self.right_chars[lower..upper]),
            Writers::Stdout(stdout) => stdout.write(&self.right_chars[lower..upper]),
            Writers::FileBuf(file) => file.write(&self.buf[lower..upper]),
            Writers::StdoutBuf(stdout) => stdout.write(&self.buf[lower..upper]),
        }
    }
    
    fn do_flush(&mut self) -> io::Result<()> {
        match self.writer {
            Writers::File(file) => file.flush(),
            Writers::Stdout(stdout) => stdout.flush(),
            Writers::FileBuf(file) => file.flush(),
            Writers::StdoutBuf(stdout) => stdout.flush(),
        }
    }

    pub fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        if self.used == 0 {
            return Ok(());
        }

        self.buf[0] = SPACE;
        self.buf[1] = SPACE;
        self.buf[2] = SPACE;
        self.buf[3] = SPACE;
        self.buf[4] = PIPE;
        let n_bytes = self.used;
        while self.used < 16 {
            let mut l = 3;
            if self.used == 7 {
                l = 4;
            } else if self.used == 15 {
                l = 5;
            }

            if let Err(err) = self.write_buf_slice(0, l) {
                return Err(err)
            }
            self.used += 1;
        }
        self.right_chars[n_bytes] = PIPE;
        self.right_chars[n_bytes+1] = NEWLINE;
        if let Err(err) = self.write_right_chars_slice(0, n_bytes+2) {
            return Err(err);
        }

        Ok(())
    }
}

impl<'a> io::Write for HexDumper<'a> {

    // TODO: Use &str safe indexing apis?
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.closed {
            return Err(io::Error::new(io::ErrorKind::Other, "Write on closed writer"));
        }

        for (i, _) in buf.iter().enumerate() {
            if self.used == 0 {
                let mut tmp = [0; 4];
                tmp[0] = (self.n >> 24) as u8;
                tmp[1] = (self.n >> 16) as u8;
                tmp[2] = (self.n >> 8) as u8;
                tmp[3] = self.n as u8;
                encode(&mut self.buf[4..], &mut tmp);

                self.buf[12] = ' ' as u8;
                self.buf[13] = ' ' as u8;

                if let Err(err) = self.write_buf_slice(4, self.buf.len()) {
                    return Err(err);
                }
            }

            encode(&mut self.buf, &buf[i..i+1]);
            self.buf[2] = SPACE;

            let mut l = 3;
            if self.used == 7 {
                self.buf[3] = SPACE;
                l = 4;
            } else if self.used == 15 {
                self.buf[3] = SPACE;
                self.buf[4] = PIPE;
                l = 5;
            }
            if let Err(err) = self.write_buf_slice(0, l) {
                return Err(err);
            }

            self.right_chars[self.used] = to_char(buf[i]);
            self.used += 1;
            self.n += 1;

            if self.used == 16 {
                self.right_chars[16] = PIPE;
                self.right_chars[17] = NEWLINE;
                if let Err(err) = self.write_right_chars() {
                    return Err(err);
                }
                self.used = 0;
            }
        }

        Ok(buf.len())        
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.closed {
            return Err(io::Error::new(io::ErrorKind::Other, "Flush on closed dumper."))
        }

        self.do_flush()
    }
}

fn to_char(c: u8) -> u8 {
    if c < 32 || c > 126 {
        PERIOD
    } else {
        c
    }
}

pub fn encode(dest: &mut [u8], src: &[u8]) -> usize {
    for (i, chr) in src.iter().enumerate() {
        dest[i*2] = HEX_TABLE[(chr>>4) as usize];
        dest[i*2+1] = HEX_TABLE[(chr&0x0f) as usize];
    }

    src.len() * 2
}
