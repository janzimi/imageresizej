fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon.ico");

    #[cfg(windows)]
    {
        let icon_path = "../../assets/icon.ico";
        if std::path::Path::new(icon_path).exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon(icon_path);
            res.compile().expect("Не удалось вшить icon.ico в imageresizej.exe");
        }
    }
}