use mupdf::{Document, TextPageOptions, Pixmap};

use crate::utils;

pub struct PdfPixmap {
    pub pixmap: Pixmap
}

pub fn load_pdf(file_path: &str) -> Result<Vec<PdfPixmap>, Box<dyn std::error::Error>> {

    let document = Document::open(file_path)?;

    let mut result: Vec<PdfPixmap> = vec!();

    let ppi = utils::get_dpi();

    for page_result in document.pages()? {
        let page = page_result?;
        let boundbox = page.bounds()?;
        let display_list = page.to_display_list(false)?;
        let device = mupdf::Device::from_display_list(&display_list)?;
        let zoom = (ppi / 96.0) as f32;
        let mut ctm = mupdf::Matrix::new_scale(zoom, zoom);
        ctm.rotate(0.0);

        page.run(&device, &ctm)?;

        let colorspace = mupdf::Colorspace::device_rgb();
        let pixmap = page.to_pixmap(&ctm, &colorspace, true, true)?;
        let pixmap_h = pixmap.height();
        result.push(PdfPixmap { pixmap: pixmap });
    }
    
    Ok(result)
}
