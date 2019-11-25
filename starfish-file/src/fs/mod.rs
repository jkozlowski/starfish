mod file;
mod filesystem;

pub use file::File;
pub use filesystem::FileSystem;

//#[cfg(test)]
//mod test {
//
//    use std::fs::File;
//    use std::io::Write;
//    use tempfile::tempfile;
//
//    #[cfg(uring)]
//    #[test]
//    fn test_read_exact() -> Result<(), io_uring::Error> {
//        let rcookie: u64 = 4321;
//        let mut queue = UringQueue::new(128)?;
//        let test_str = "write test";
//        let mut buf: Vec<u8> = vec![];
//
//        let file = prepare_test_file(test_str)?;
//
//        buf.resize_with(test_str.as_bytes().len(), Default::default);
//        queue.prepare_read(file.as_raw_fd(), &mut buf[..], 0, rcookie)?;
//        queue.submit_requests()?;
//        let completion_cookie_opt = queue.get_completion(true)?;
//        assert!(completion_cookie_opt.is_some());
//        let completion_cookie = completion_cookie_opt.unwrap();
//        assert!(
//            completion_cookie == rcookie,
//            "completion_cookie={} cookie={}",
//            completion_cookie,
//            rcookie
//        );
//
//        let result_str = std::str::from_utf8(&buf).unwrap();
//        assert!(
//            result_str == test_str,
//            "result_str={}, test_str={}",
//            result_str,
//            test_str
//        );
//        println!("Output: \'{}\'", result_str);
//        Ok(())
//    }
//
//    fn prepare_test_file(s: &str) -> Result<File, io_uring::Error> {
//        let mut file = tempfile().unwrap();
//        writeln!(file, "{}", s).unwrap();
//        Ok(file)
//    }
//}
