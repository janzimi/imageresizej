fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon.ico");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        let icon_path = "../../assets/icon.ico";
        if std::path::Path::new(icon_path).exists() {
            res.set_icon(icon_path);
        }
        res.set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
</assembly>
"#,
        );
        res.compile()
            .expect("Не удалось вшить ресурсы Windows в imageresizej.exe");
    }
}
