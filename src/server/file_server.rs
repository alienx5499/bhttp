use std::fs;
use std::path::{Component, Path, PathBuf};

/// Safe static file server with path traversal defenses and MIME resolution.
#[derive(Debug, Clone)]
pub struct FileServer {
    root: PathBuf,
}

impl FileServer {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            root: root_dir.as_ref().to_path_buf(),
        }
    }

    /// Resolve a sanitized file path safely under the root.
    /// Defends against `..` directory traversal and root escape attacks:
    ///   - Strips query parameters (e.g. /index.html?v=1 -> /index.html)
    ///   - Strips leading forward slashes
    ///   - Lexically parses each Path component using std::path::Component
    ///   - Immediately rejects any ParentDir (..), RootDir (/), or Windows Prefix (C:)
    ///   - If resolving to a directory, looks for an index.html file
    pub fn resolve_path(&self, requested_path: &str) -> Option<PathBuf> {
        let clean = requested_path.split('?').next().unwrap_or(requested_path);
        let trimmed = clean.trim_start_matches('/');

        let mut target = self.root.clone();
        for component in Path::new(trimmed).components() {
            match component {
                // Safe relative file or directory name
                Component::Normal(c) => target.push(c),
                // Current directory (.) is safely ignored
                Component::CurDir => continue,
                // Any parent directory traversal (..) or root override is strictly rejected
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return None;
                }
            }
        }

        // If target resolves to a directory, check for default index.html
        if target.is_dir() {
            let index = target.join("index.html");
            if index.is_file() {
                return Some(index);
            }
        }

        if target.is_file() { Some(target) } else { None }
    }

    /// Read file content and determine appropriate MIME type.
    pub fn read_file(&self, path: &Path) -> Option<(Vec<u8>, &'static str)> {
        let content = fs::read(path).ok()?;
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let mime = match ext.as_str() {
            "html" | "htm" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" => "application/javascript; charset=utf-8",
            "json" => "application/json",
            "txt" => "text/plain; charset=utf-8",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        };

        Some((content, mime))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_path_traversal_blocked() {
        let temp_dir = std::env::temp_dir().join("bhttp_sec_test");
        let _ = fs::create_dir_all(&temp_dir);
        let fs_server = FileServer::new(&temp_dir);

        assert_eq!(fs_server.resolve_path("../../etc/passwd"), None);
        assert_eq!(fs_server.resolve_path("/../root"), None);
    }

    #[test]
    fn test_valid_file_resolved() {
        let temp_dir = std::env::temp_dir().join("bhttp_valid_test");
        let _ = fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("hello.txt");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "hello world").unwrap();

        let fs_server = FileServer::new(&temp_dir);
        assert!(fs_server.resolve_path("hello.txt").is_some());
    }
}
