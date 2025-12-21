use super::SecretEntry;
use crate::client::{ClientError, OpBrokerClient};
use base64::Engine;
use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc;
use protocol::OpSecretReference;
use std::cmp;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::runtime::Runtime;
use zeroize::Zeroize;

const ROOT_INO: u64 = 1;
const FIRST_FILE_INO: u64 = 2;
const TTL: Duration = Duration::from_secs(1);

/// allowlist に登録された 1 つのシークレットを表す静的メタデータ。
struct FileEntry {
    reference: OpSecretReference,
    ino: u64,
    name: String,
}

impl FileEntry {
    fn encoded_name(reference: &OpSecretReference) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(reference.as_str())
    }

    fn new(reference: OpSecretReference, ino: u64) -> Self {
        let name = Self::encoded_name(&reference);
        Self {
            reference,
            ino,
            name,
        }
    }
}

/// op-broker のシークレットをファイルに見せかける読み取り専用 FUSE 実装。
pub struct OpSecretsFs {
    runtime: Arc<Runtime>,
    client: Mutex<OpBrokerClient>,
    rpc_timeout: Duration,
    files: Vec<FileEntry>,
    name_to_index: HashMap<String, usize>,
    ino_to_index: HashMap<u64, usize>,
    uid: u32,
    gid: u32,
    start_time: SystemTime,
}

impl OpSecretsFs {
    pub fn new(
        runtime: Arc<Runtime>,
        client: OpBrokerClient,
        entries: Vec<SecretEntry>,
        rpc_timeout: Duration,
    ) -> Self {
        let mut files = Vec::new();
        let mut name_to_index = HashMap::new();
        let mut ino_to_index = HashMap::new();

        for (idx, entry) in entries.into_iter().enumerate() {
            let entry = FileEntry::new(entry.reference, FIRST_FILE_INO + idx as u64);
            name_to_index.insert(entry.name.clone(), idx);
            ino_to_index.insert(entry.ino, idx);
            files.push(entry);
        }

        Self {
            runtime,
            client: Mutex::new(client),
            rpc_timeout,
            files,
            name_to_index,
            ino_to_index,
            uid: unsafe { libc::geteuid() as u32 },
            gid: unsafe { libc::getegid() as u32 },
            start_time: SystemTime::now(),
        }
    }

    pub fn name_for_reference(&self, reference: &OpSecretReference) -> String {
        FileEntry::encoded_name(reference)
    }

    fn lookup_entry_by_name(&self, name: &OsStr) -> Option<&FileEntry> {
        let name = name.to_str()?;
        self.name_to_index
            .get(name)
            .map(|index| &self.files[*index])
    }

    fn entry_by_ino(&self, ino: u64) -> Option<&FileEntry> {
        self.ino_to_index.get(&ino).map(|index| &self.files[*index])
    }

    fn root_attr(&self) -> FileAttr {
        FileAttr {
            ino: ROOT_INO,
            size: 0,
            blocks: 0,
            atime: self.start_time,
            mtime: self.start_time,
            ctime: self.start_time,
            crtime: self.start_time,
            kind: FileType::Directory,
            perm: 0o555,
            nlink: 2,
            uid: self.uid,
            gid: self.gid,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    fn file_attr(&self, entry: &FileEntry) -> FileAttr {
        FileAttr {
            ino: entry.ino,
            size: 0,
            blocks: 0,
            atime: self.start_time,
            mtime: self.start_time,
            ctime: self.start_time,
            crtime: self.start_time,
            kind: FileType::RegularFile,
            perm: 0o400,
            nlink: 1,
            uid: self.uid,
            gid: self.gid,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    fn map_client_error(err: ClientError) -> libc::c_int {
        match err {
            ClientError::PermissionDenied => libc::EACCES,
            ClientError::NotFound => libc::ENOENT,
            ClientError::InvalidRequest => libc::EINVAL,
            ClientError::Timeout => libc::EIO,
            ClientError::Unavailable => libc::EHOSTDOWN,
            ClientError::Internal => libc::EIO,
        }
    }

    /// 指定したシークレット参照で broker に gRPC Read を発行する。
    fn request_secret(&self, reference: String) -> Result<Vec<u8>, ClientError> {
        let mut client = self.client.lock().map_err(|_| ClientError::Internal)?;
        self.runtime
            .block_on(client.read_secret(reference, self.rpc_timeout))
    }
}

impl Filesystem for OpSecretsFs {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent != ROOT_INO {
            reply.error(libc::ENOENT);
            return;
        }

        if let Some(entry) = self.lookup_entry_by_name(name) {
            reply.entry(&TTL, &self.file_attr(entry), 0);
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        if ino == ROOT_INO {
            reply.attr(&TTL, &self.root_attr());
            return;
        }

        if let Some(entry) = self.entry_by_ino(ino) {
            reply.attr(&TTL, &self.file_attr(entry));
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != ROOT_INO {
            reply.error(libc::ENOTDIR);
            return;
        }

        let index = if offset < 0 { 0 } else { offset as usize };
        if index == 0 {
            let full = reply.add(ROOT_INO, 1, FileType::Directory, OsStr::new("."));
            if full {
                return;
            }
        }
        if index == 0 || index == 1 {
            let full = reply.add(ROOT_INO, 2, FileType::Directory, OsStr::new(".."));
            if full {
                return;
            }
        }

        let start = index.saturating_sub(2);
        for (i, entry) in self.files.iter().enumerate().skip(start) {
            let next_offset = (i + 3) as i64;
            if reply.add(
                entry.ino,
                next_offset,
                FileType::RegularFile,
                OsStr::new(entry.name.as_str()),
            ) {
                return;
            }
        }
        reply.ok();
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        if ino == ROOT_INO {
            reply.error(libc::EISDIR);
            return;
        }

        let Some(entry) = self.entry_by_ino(ino) else {
            reply.error(libc::ENOENT);
            return;
        };

        let mut data = match self.request_secret(entry.reference.as_str().to_owned()) {
            Ok(bytes) => bytes,
            Err(err) => {
                reply.error(Self::map_client_error(err));
                return;
            }
        };

        let offset = offset as usize;
        let size = size as usize;
        if offset >= data.len() {
            reply.data(&[]);
            data.zeroize();
            return;
        }

        let end = cmp::min(data.len(), offset.saturating_add(size));
        let slice = &data[offset..end];
        reply.data(slice);
        data.zeroize();
    }

    // fn opendir(&mut self, _req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
    //     if ino != ROOT_INO {
    //         reply.error(libc::ENOTDIR);
    //         return;
    //     }
    //     if flags & libc::O_ACCMODE != libc::O_RDONLY {
    //         reply.error(libc::EACCES);
    //         return;
    //     }
    //     reply.opened(0, 0);
    // }
}
