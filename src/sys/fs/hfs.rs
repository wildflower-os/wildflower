use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use spin::Mutex;

pub struct HiddenFile {
    filename: String,
    data: Vec<u8>,
}

pub struct HiddenFS {
    files: Mutex<Vec<HiddenFile>>,
}

impl HiddenFS {
    pub fn new() -> Self {
        let hfs = HiddenFS {
            files: Mutex::new(Vec::new()),
        };
        hfs.load_from_disk();
        hfs
    }

    pub fn store_file(&self, filename: &str, data: &[u8]) {
        let mut files = self.files.lock();
        if let Some(file) = files.iter_mut().find(|f| f.filename == filename) {
            file.data = data.to_vec();
        } else {
            files.push(HiddenFile {
                filename: filename.to_string(),
                data: data.to_vec(),
            });
        }
        drop(files);
        self.save_to_disk();
    }

    pub fn retrieve_file(&self, filename: &str) -> Option<Vec<u8>> {
        let files = self.files.lock();
        files
            .iter()
            .find(|f| f.filename == filename)
            .map(|f| f.data.clone())
    }

    pub fn delete_file(&self, filename: &str) -> bool {
        let mut files = self.files.lock();
        if let Some(pos) = files.iter().position(|f| f.filename == filename) {
            files.remove(pos);
            drop(files);
            self.save_to_disk();
            true
        } else {
            false
        }
    }

    pub fn list_files(&self) -> Vec<String> {
        let files = self.files.lock();
        files.iter().map(|f| f.filename.clone()).collect()
    }

    pub fn file_exists(&self, filename: &str) -> bool {
        let files = self.files.lock();
        files.iter().any(|f| f.filename == filename)
    }

    fn save_to_disk(&self) {
        let files = self.files.lock();
        let mut data = Vec::new();
        data.extend((files.len() as u32).to_le_bytes());
        for file in files.iter() {
            let name_bytes = file.filename.as_bytes();
            let name_len = name_bytes.len() as u32;
            data.extend(name_len.to_le_bytes());
            data.extend(name_bytes);
            let data_len = file.data.len() as u32;
            data.extend(data_len.to_le_bytes());
            data.extend(&file.data);
        }
        Self::write_storage(&data);
    }

    fn load_from_disk(&self) {
        let data = Self::read_storage();

        let mut cursor = 0;
        if data.len() < 4 {
            return; // Not enough data
        }
        let file_count = u32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        cursor += 4;

        let mut files = self.files.lock();
        for _ in 0..file_count {
            if cursor + 4 > data.len() {
                break; // Not enough data
            }
            let name_len = u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]) as usize;
            cursor += 4;
            if cursor + name_len > data.len() {
                break;
            }
            let filename = match String::from_utf8(data[cursor..cursor + name_len].to_vec()) {
                Ok(name) => name,
                Err(_) => break,
            };
            cursor += name_len;
            if cursor + 4 > data.len() {
                break;
            }
            let data_len = u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]) as usize;
            cursor += 4;
            if cursor + data_len > data.len() {
                break;
            }
            let file_data = data[cursor..cursor + data_len].to_vec();
            cursor += data_len;

            files.push(HiddenFile {
                filename,
                data: file_data,
            });
        }
    }

    fn write_storage(_: &[u8]) {
        // TODO: Implement
    }

    fn read_storage() -> Vec<u8> {
        // TODO: Implement
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test_case]
    fn test_store_retrieve() {
        let fs = HiddenFS::new();
        fs.store_file("test.txt", b"Hello, world!");
        assert_eq!(
            fs.retrieve_file("test.txt"),
            Some(b"Hello, world!".to_vec())
        );
    }

    #[test_case]
    fn test_delete() {
        let fs = HiddenFS::new();
        fs.store_file("test.txt", b"Hello, world!");
        assert_eq!(fs.delete_file("test.txt"), true);
        assert_eq!(fs.delete_file("test.txt"), false);
    }

    #[test_case]
    fn test_list_files() {
        let fs = HiddenFS::new();
        fs.store_file("test.txt", b"Hello, world!");
        fs.store_file("test2.txt", b"Goodbye, world!");
        assert_eq!(
            fs.list_files(),
            vec!["test.txt".to_string(), "test2.txt".to_string()]
        );
    }

    #[test_case]
    fn test_file_exists() {
        let fs = HiddenFS::new();
        fs.store_file("test.txt", b"Hello, world!");
        assert!(fs.file_exists("test.txt"));
        assert!(!fs.file_exists("test2.txt"));
    }
}
