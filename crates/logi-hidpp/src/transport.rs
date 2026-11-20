use crate::error::HidppError;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

pub trait Transport: Send {
    fn write_report(&mut self, report: &[u8]) -> Result<(), HidppError>;
    fn read_report(&mut self, timeout_ms: u32) -> Result<Vec<u8>, HidppError>;
}

/// Real Linux `/dev/hidraw*` transport using direct kernel character device I/O.
pub struct HidrawTransport {
    path: PathBuf,
    file: File,
}

impl HidrawTransport {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, HidppError> {
        let path_buf = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path_buf)?;

        Ok(Self {
            path: path_buf,
            file,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Transport for HidrawTransport {
    fn write_report(&mut self, report: &[u8]) -> Result<(), HidppError> {
        // Try standard write first
        match self.file.write_all(report) {
            Ok(_) => {
                let _ = self.file.flush();
                Ok(())
            }
            Err(e) if e.raw_os_error() == Some(libc::EPIPE) => {
                // If write returns EPIPE (common on Logi Bolt/Unifying USB control endpoint),
                // use HIDIOCSOUTPUT ioctl: _IOC(_IOC_READ | _IOC_WRITE, 'H', 0x0B, len)
                let fd = self.file.as_raw_fd();
                let len = report.len();
                let ioc = ((3u64) << 30) | ((b'H' as u64) << 8) | 0x0B | ((len as u64) << 16);
                let ret = unsafe { libc::ioctl(fd, ioc, report.as_ptr()) };
                if ret < 0 {
                    Err(HidppError::Io(std::io::Error::last_os_error()))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(HidppError::Transport(format!("Failed to write to hidraw: {}", e))),
        }
    }

    fn read_report(&mut self, timeout_ms: u32) -> Result<Vec<u8>, HidppError> {
        let fd = self.file.as_raw_fd();
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };

        let ret = unsafe { libc::poll(&mut pollfd, 1, timeout_ms as libc::c_int) };
        if ret < 0 {
            return Err(HidppError::Io(std::io::Error::last_os_error()));
        } else if ret == 0 {
            return Err(HidppError::Timeout);
        }

        if (pollfd.revents & libc::POLLIN) != 0 {
            let mut buf = vec![0u8; 64];
            match self.file.read(&mut buf) {
                Ok(n) if n > 0 => {
                    buf.truncate(n);
                    Ok(buf)
                }
                Ok(_) => Err(HidppError::Timeout),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(HidppError::Timeout),
                Err(e) => Err(HidppError::Io(e)),
            }
        } else if (pollfd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL)) != 0 {
            Err(HidppError::DeviceOffline)
        } else {
            Err(HidppError::Timeout)
        }
    }
}

/// In-memory mock transport for unit testing and simulation.
pub struct MockTransport {
    responses: HashMap<Vec<u8>, Vec<u8>>,
    history: Vec<Vec<u8>>,
    next_response: Option<Vec<u8>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            history: Vec::new(),
            next_response: None,
        }
    }

    pub fn register_response(&mut self, request_prefix: &[u8], response: &[u8]) {
        self.responses.insert(request_prefix.to_vec(), response.to_vec());
    }

    pub fn set_next_response(&mut self, response: Vec<u8>) {
        self.next_response = Some(response);
    }

    pub fn history(&self) -> &[Vec<u8>] {
        &self.history
    }
}

impl Transport for MockTransport {
    fn write_report(&mut self, report: &[u8]) -> Result<(), HidppError> {
        self.history.push(report.to_vec());

        // Check if there's a registered response for this exact request or prefix
        for (prefix, resp) in &self.responses {
            if report.starts_with(prefix) {
                self.next_response = Some(resp.clone());
                return Ok(());
            }
        }

        Ok(())
    }

    fn read_report(&mut self, _timeout_ms: u32) -> Result<Vec<u8>, HidppError> {
        if let Some(resp) = self.next_response.take() {
            Ok(resp)
        } else {
            Err(HidppError::Timeout)
        }
    }
}
