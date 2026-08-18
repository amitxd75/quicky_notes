fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        let _ = res.compile();
    }
}
