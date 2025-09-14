use mupdf::{Document, TextPageOptions, Pixmap};

use crate::utils;

pub struct PdfPixmap {
    pub pixmap: Pixmap
}

unsafe impl Send for PdfPixmap {}
unsafe impl Sync for PdfPixmap {}

pub fn load_pdf(file_path: &str, pdf_pixmaps: &std::sync::Arc<std::sync::Mutex<Vec<PdfPixmap>>>) -> Result<(), Box<dyn std::error::Error>> {

    let document = Document::open(file_path)?;

    let ppi = utils::get_dpi();

    for page_result in document.pages()? {
        let page = page_result?;
        let boundbox = page.bounds()?;
        // let display_list = page.to_display_list(false)?;
        // let device = mupdf::Device::from_display_list(&display_list)?;
        let zoom = (ppi / 96.0) as f32;
        let mut ctm = mupdf::Matrix::new_scale(zoom, zoom);
        ctm.rotate(0.0);
        let cs = mupdf::Colorspace::device_rgb();

        // page.run(&device, &ctm)?;

        let pixmap = page.to_pixmap(&ctm, &cs, true, true)?;
        pdf_pixmaps.lock().unwrap().push(PdfPixmap { pixmap: pixmap });
    }
    
    Ok(())
}
