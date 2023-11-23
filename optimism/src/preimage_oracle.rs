use crate::cannon::{
    HostProgram, Preimage, HINT_CLIENT_READ_FD, HINT_CLIENT_WRITE_FD, PREIMAGE_CLIENT_READ_FD,
    PREIMAGE_CLIENT_WRITE_FD,
};
use command_fds::{CommandFdExt, FdMapping};
use os_pipe::{PipeReader, PipeWriter};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;

use std::process::{Child, Command};
pub struct PreImageOracle {
    pub cmd: Command,
    pub oracle_client: RW,
    pub hint_writer: RW,
}

pub struct ReadWrite<R, W> {
    pub reader: R,
    pub writer: W,
}

pub struct RW(pub ReadWrite<PipeReader, PipeWriter>);

impl RW {
    pub fn create() -> Option<Self> {
        let (reader, writer) = os_pipe::pipe().ok()?;
        Some(RW(ReadWrite { reader, writer }))
    }
}

impl PreImageOracle {
    pub fn create(hp_opt: &Option<HostProgram>) -> PreImageOracle {
        let host_program = hp_opt.as_ref().expect("No host program given");

        let mut cmd = Command::new(&host_program.name);
        cmd.args(&host_program.arguments);

        let p_client = RW::create().expect("");
        let p_oracle = RW::create().expect("");
        let h_client = RW::create().expect("");
        let h_oracle = RW::create().expect("");

        // file descriptors 0, 1, 2 respectively correspond to the inherited stdin,
        // stdout, stderr.
        // We need to map 3, 4, 5, 6 in the child process
        let RW(ReadWrite {
            reader: h_reader,
            writer: h_writer,
        }) = h_oracle;
        let RW(ReadWrite {
            reader: p_reader,
            writer: p_writer,
        }) = p_oracle;

        // Use constant defined
        cmd.fd_mappings(vec![
            FdMapping {
                parent_fd: h_reader.as_raw_fd(),
                child_fd: HINT_CLIENT_READ_FD,
            },
            FdMapping {
                parent_fd: h_writer.as_raw_fd(),
                child_fd: HINT_CLIENT_WRITE_FD,
            },
            FdMapping {
                parent_fd: p_reader.as_raw_fd(),
                child_fd: PREIMAGE_CLIENT_READ_FD,
            },
            FdMapping {
                parent_fd: p_writer.as_raw_fd(),
                child_fd: PREIMAGE_CLIENT_WRITE_FD,
            },
        ])
        .unwrap_or_else(|_| panic!("Could not map file descriptors to server process"));

        PreImageOracle {
            cmd,
            oracle_client: p_client,
            hint_writer: h_client,
        }
    }

    pub fn start(&mut self) -> Child {
        // Spawning inherits the current process's stdin/stdout/stderr descriptors
        self.cmd
            .spawn()
            .expect("Could not spawn pre-image oracle process")
    }

    // The preimage protocol goes as follows
    // 1. Ask for data through a key
    // 2. Get the answers as a
    //   a. a 64-bit integer indicating the length of the actual data
    //   b. the preimage data, with a size of <length> bits
    pub fn get_preimage(&mut self, key: [u8; 32]) -> Preimage {
        let RW(ReadWrite { reader, writer }) = &mut self.oracle_client;

        let mut msg_key = vec![2_u8]; // Assumes Keccak Key
        msg_key.extend_from_slice(&key[1..31]);
        let _ = writer.write(&msg_key);

        let mut buf = [0_u8; 8];
        let _ = reader.read_exact(&mut buf);

        let length = u64::from_be_bytes(buf);
        let mut handle = reader.take(length);
        let mut v = vec![0_u8; length as usize];
        let _ = handle.read(&mut v);

        // We should have read exactly <length> bytes
        assert_eq!(v.len(), length as usize);

        Preimage::create(v)
    }

    pub fn hint(&mut self, _hint: Vec<u8>) {}
}
