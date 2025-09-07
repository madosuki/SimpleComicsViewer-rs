SimpleComicsViewer-rs is rewrite https://github.com/madosuki/SimpleComicsViewer using Rust and gtk4.  
Currenlty status: WIP. But can read jpg and png and zip(jpg, png).

### Depends List
- **[GTK4](https://crates.io/crates/gtk4)**
- **[libarchive](https://www.libarchive.org/)**
- **[libarchive_extractor-rs](https://github.com/madosuki/libarchive_extractor-rs)**
- **[mupdf-rs](https://github.com/ArtifexSoftware/mupdf)**
  
### Supported file format
- **Single File**
    - JPEG
    - PNG
- **Compressed File**
    - PNG or JPEG inside of non encrypted zip
- **PDF**

 ### Manual
- **Shortcut Key**
    - **Move to Right**  
        l or right arrow or Ctrl+f  
    - **Move to Left**  
        h or left arrow or Ctrl+b  
    - **Open File**  
        Ctrl+o  
    - **Quit**  
        Ctrl+q or Alt+F4  
