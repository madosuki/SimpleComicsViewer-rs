use gtk4 as gtk;
use gtk::gdk;

use gtk::glib::Propagation;
use gtk::prelude::{
    ActionMapExtManual, ApplicationExt, ApplicationWindowExt, BoxExt, DialogExt, DrawingAreaExt,
    DrawingAreaExtManual, FileChooserExt, FileChooserExtManual, FileExt, GdkCairoContextExt,
    GridExt, GtkApplicationExt, GtkWindowExt, MenuLinkIterExt, MenuModelExt, PixbufLoaderExt,
    PopoverExt, SurfaceExt, WidgetExt,
};
use gtk::{
    cairo, gio, glib, Application, ApplicationWindow, DrawingArea,
    EventControllerKey,
};

use anyhow::Result;

use std::sync::Arc;
use std::sync::Mutex;

use crate::pdf_loader::PdfPixmap;
use crate::{image_container, pdf_loader};
use crate::image_loader;
use crate::utils;
use image_container::{ImageContainer, ImageContainerEx};

#[derive(Default)]
struct PagesInfo {
    current_page_index: Arc<Mutex<usize>>,
    loaded_filename: Arc<Mutex<Option<String>>>,
    loaded_dirname: Arc<Mutex<Option<String>>>,
}

#[derive(Default)]
struct Settings {
    is_dual_mode: Arc<Mutex<bool>>,
}

#[derive(Default)]
struct MarginData {
    left_margin: i32,
    top_margin: i32,
}

#[derive(Default)]
struct MarginDataForDual {
    left_margin: i32,
    top_margin_for_left: i32,
    top_margin_for_right: i32,
}

struct MainWindow {
    window: ApplicationWindow,
    v_box: gtk::Box,
    image_container_list: Arc<Mutex<Vec<ImageContainer>>>,
    pages_info: std::sync::Arc<PagesInfo>,
    settings: std::sync::Arc<Settings>,
    view_window: gtk::ScrolledWindow,
}

fn update_window_title(window: &gtk::ApplicationWindow, title_text: &str) {
    let Some(_) = window.title() else {
        return;
    };

    let new_title = format!("Simple Comics Viewer: {}", title_text);
    window.set_title(Some(&new_title));
}

fn calc_margin_for_single(
    pixbuf_data: &gtk::gdk_pixbuf::Pixbuf,
    target_width: i32,
    target_height: i32,
) -> MarginData {
    let pic_height = pixbuf_data.height();
    let pic_width = pixbuf_data.width();

    let width_diff = target_width - pic_width;
    let left_margin = if width_diff < 0 || width_diff == 0 {
        0
    } else {
        width_diff / 2
    };

    let height_diff = target_height - pic_height;
    let top_margin = if height_diff < 0 || height_diff == 0 {
        0
    } else {
        height_diff / 2
    };

    MarginData {
        left_margin,
        top_margin,
    }
}

fn calc_margin_for_dual(
    left: &gtk::gdk_pixbuf::Pixbuf,
    right: &gtk::gdk_pixbuf::Pixbuf,
    target_width: i32,
    target_height: i32,
) -> MarginDataForDual {
    let left_height = left.height();
    let left_width = left.width();
    let right_height = right.height();
    let right_width = right.width();

    let width_diff = target_width - (left_width + right_width);
    let left_margin = if width_diff <= 0 { 0 } else { width_diff / 2 };

    let left_height_diff = target_height - left_height;
    let top_margin_for_left = if left_height_diff <= 0 {
        0
    } else {
        left_height_diff / 2
    };

    let right_height_diff = target_height - right_height;
    let top_margin_for_right = if right_height_diff <= 0 {
        0
    } else {
        right_height_diff / 2
    };

    MarginDataForDual {
        left_margin,
        top_margin_for_left,
        top_margin_for_right,
    }
}

fn scale_page_for_single(
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    current_page_index: usize,
    target_width: i32,
    target_height: i32,
) {
    if (target_width < 1) || (target_height < 1) {
        return;
    }

    if (*image_container_list.lock().unwrap()).is_empty() {
        return;
    }

    if current_page_index >= (*image_container_list.lock().unwrap()).len() {
        return;
    }

    (*image_container_list.lock().unwrap())[current_page_index].scale(
        target_width,
        target_height,
        false,
    );
}

fn scale_page_for_dual(
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    current_page_index: usize,
    target_width: i32,
    target_height: i32,
) {
    if (target_width < 1) || (target_height < 1) {
        return;
    }

    let image_container_list_ptr = image_container_list.lock().unwrap();
    if image_container_list_ptr.is_empty() {
        return;
    }

    let next_index = current_page_index + 1;
    let _image_container_list_len = image_container_list_ptr.len();

    let final_target_width = target_width / 2;
    image_container_list_ptr[current_page_index].scale(
        final_target_width,
        target_height,
        true,
    );

    if next_index < _image_container_list_len {
        image_container_list_ptr[next_index].scale(
            final_target_width,
            target_height,
            true,
        );
    }
}

fn append_image_container_from_file(
    file: &gio::File,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    page_index: usize,
) {
    let image_container = ImageContainer::default();
    (*image_container_list.lock().unwrap()).push(image_container);

    (*image_container_list.lock().unwrap())[page_index].set_pixbuf_from_file(file);
}

// fn scale_images(
//     image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
//     page_index: usize,
//     width: i32,
//     height: i32,
//     is_dual_mode: bool,
// ) {
//     if is_dual_mode {
//         scale_page_for_dual(image_container_list, page_index, width, height);
//         // let half_width = width / 2;
//         // (*image_container_list.lock().unwrap())[page_index].scale(half_width, height, true);
//     } else {
//         // (*image_container_list.lock().unwrap())[page_index].scale(width, height, false);
//         scale_page_for_single(image_container_list, page_index, width, height);
//     }
// }

fn get_file_path_from_file_desc(file: &gio::File) -> Option<String> {
    let Some(pathbuf) = file.path() else {
        return None;
    };
    let Some(pathname) = pathbuf.as_path().to_str() else {
        return None;
    };
    // let Some(file_name_osstr) = pathbuf.file_name() else {
    //     return None;
    // };
    // let Some(file_name) = file_name_osstr.to_str() else {
    //     return None;
    // };

    // Some(file_name.to_owned())
    Some(pathname.to_owned())
}

fn set_page_from_image_container_list(
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    settings: &Arc<Settings>,
    drawing_area_ref: &DrawingArea,
) {
    let width = drawing_area_ref.allocated_width();
    let height = drawing_area_ref.allocated_height();
    let count = 0;
    let is_dual_model = settings.is_dual_mode.lock().unwrap();
    if *is_dual_model {
        scale_page_for_dual(image_container_list, count, width, height);
    } else {
        scale_page_for_single(image_container_list, count, width, height);
    }
}

fn open_and_set_image_to_image_container_from_zip(
    pathname: &String,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
) -> bool {
    match image_loader::load_from_compressed_file_to_memory(pathname) {
        Ok(extracted) => {
            extracted.into_iter().for_each(|v| {
                let image_container = ImageContainer::default();
                image_container.set_pixbuf_from_bytes(&v.value);
                (*image_container_list.lock().unwrap()).push(image_container);
            });
            true
        },
        Err(e) => {
            eprintln!("{}", e);
            false
        }
    }
}

fn set_image_to_image_container_from_pdf_pixmaps(
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    pdf_pixmaps: &Arc<Mutex<Vec<PdfPixmap>>>
) {
    pdf_pixmaps.lock().unwrap().iter().for_each(|v| {
        let image_container = ImageContainer::default();
        image_container.set_pixbuf_from_pdf_pixmap(&v);
        (*image_container_list.lock().unwrap()).push(image_container);
    });
}


fn open_and_set_image_to_image_container_from_file(
    file: &gio::File,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    page_index: usize,
) -> bool {
    match utils::detect_file_type_from_file(&file) {
        utils::FileType::NONE => false,
        _ => {
            append_image_container_from_file(file, image_container_list, page_index);
            true
        }
    }
}

async fn read_dir_and_set_images(
    file: &gio::File,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
) -> bool {
    let Some(path) =  file.path() else {
        return false;
    };
    let Some(dir_path) = path.parent() else {
        return false;
    };
    let mut count: usize = 0;
    for entry in dir_path.read_dir().expect("Failed call read_dir") {
        if let Ok(entry) = entry {
            match entry.file_type() {
                Ok(v) => {
                    if v.is_file() {
                        let tmp_path = entry.path();
                        let tmp_file = gio::File::for_path(&tmp_path);
                        let r = open_and_set_image_to_image_container_from_file(&tmp_file, &image_container_list, count);
                        if r {
                            count = count + 1;
                        }
                    }
                },
                Err(_) => {
                }
            }
        }
    }
    if count == 0 {
        false
    } else {
        true
    }
}

fn open_file_action(
    window: &gtk::ApplicationWindow,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    drawing_area_ref: &DrawingArea,
    pages_bar: &gtk::ProgressBar,
    settings: &Arc<Settings>,
    pages_info: &Arc<PagesInfo>,
    spinner: &gtk::Spinner,
) {
    let dialog = gtk::FileChooserDialog::new(
        Some("File Select"),
        Some(window),
        gtk::FileChooserAction::Open,
        &[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ],
    );

    let file_filter = gtk::FileFilter::new();
    file_filter.add_pattern("*.zip");
    file_filter.add_pattern("*.jpg");
    file_filter.add_pattern("*.png");
    file_filter.add_pattern("*.pdf");
    dialog.add_filter(&file_filter);

    dialog.connect_response(glib::clone!(#[weak] window, #[strong] image_container_list, #[strong] pages_info, #[weak] drawing_area_ref, #[weak] pages_bar,  #[weak] spinner, #[strong] settings, move |file_dialog, response| {
        if response == gtk::ResponseType::Ok {
            let Some(file) = file_dialog.file() else { return };
            let Some(path) = file.path() else { return };
            if !path.is_file() { return; }

            (*image_container_list.lock().unwrap()).clear();
            *pages_info.current_page_index.lock().unwrap() = 0;
            match utils::detect_file_type_from_file(&file) {
                utils::FileType::ZIP => {
                    let pathname = get_file_path_from_file_desc(&file).unwrap();
                    let pathname_cloned = pathname.clone();
                    let (tx, rx) = std::sync::mpsc::sync_channel::<i32>(1);
                    let image_container_list_arc_cloned = Arc::clone(&image_container_list);

                    spinner.show();
                    spinner.start();

                    let _ = std::thread::spawn(move || {
                        let r = open_and_set_image_to_image_container_from_zip(&pathname_cloned, &image_container_list_arc_cloned);
                        if !r {
                            tx.send(2).unwrap();
                        } else {
                            tx.send(0).unwrap();
                        }
                    });
                    
                    glib::spawn_future_local(glib::clone!(#[weak] window, #[strong] image_container_list, #[strong] settings, #[weak] drawing_area_ref, #[strong] pages_info, async move {
                        update_window_title(&window, "Now Loading...");

                        let _source_id = glib::idle_add_local(glib::clone!(#[strong] image_container_list, #[strong] settings, #[strong] drawing_area_ref, #[strong] pages_bar, move || {
                            match rx.try_recv() {
                                Ok(v) => {
                                    match v {
                                        0 => {
                                            set_page_from_image_container_list(&image_container_list, &settings, &drawing_area_ref);
                                            *pages_info.loaded_filename.lock().unwrap() = Some(pathname.clone());
                                            update_window_title(&window, &pathname);

                                            spinner.stop();
                                            spinner.hide();

                                            pages_bar.set_fraction(0.0);
                                            pages_bar.set_inverted(true);
                                            // pages_bar.show();
                                            drawing_area_ref.queue_draw();

                                            return glib::ControlFlow::Break;
                                        },
                                        2 => {
                                            spinner.stop();
                                            spinner.hide();

                                            update_window_title(&window, "Failed");
                                            return glib::ControlFlow::Break;
                                        }
                                        _ => {
                                            return glib::ControlFlow::Continue;
                                        }
                                    }
                                },
                                Err(_) => {
                                    return glib::ControlFlow::Continue;
                                }
                            }}));
                        // glib::idle_add_local_once(glib::clone!(#[strong] image_container_list, #[strong] settings, #[weak] drawing_area_ref, #[weak] pages_bar, move || {
                        //     *pages_info.loaded_filename.lock().unwrap() = Some(pathname.clone());
                        //     update_window_title(&window, &pathname);

                        //     set_page_from_image_container_list(&image_container_list, &settings, &drawing_area_ref);
                        //     pages_bar.set_fraction(0.0);
                        //     pages_bar.set_inverted(true);
                        //     // pages_bar.show();
                        //     drawing_area_ref.queue_draw();
                        // }));
                    }));
                },
                utils::FileType::PDF => {
                    let pathname = get_file_path_from_file_desc(&file).unwrap();
                    let pathname_cloned = pathname.clone();
                    update_window_title(&window, "Now Loading...");
                    spinner.show();
                    spinner.start();
                    let (tx, rx) = std::sync::mpsc::sync_channel::<i32>(1);
                    let pdf_pixmaps_arc: Arc<Mutex<Vec<PdfPixmap>>> = Arc::new(Mutex::new(vec!()));
                    let pdf_pixmaps_arc_clone = Arc::clone(&pdf_pixmaps_arc);
                    let a = std::thread::spawn(move || {
                        match pdf_loader::load_pdf(&pathname_cloned, &pdf_pixmaps_arc_clone) {
                            Ok(_) => tx.send(0).unwrap(),
                            Err(_) => tx.send(2).unwrap()
                        }
                    });

                    glib::spawn_future_local(glib::clone!(#[weak] window, #[weak] image_container_list, #[strong] settings, #[weak] drawing_area_ref, #[strong] pages_info, async move {

                        let _source_id = glib::idle_add_local(glib::clone!(#[strong] image_container_list, #[strong] settings, #[strong] drawing_area_ref, #[strong] pages_bar, move || {
                            match rx.try_recv() {
                                Ok(v) => {
                                    match v {
                                        0 => {
                                            set_image_to_image_container_from_pdf_pixmaps(&image_container_list, &pdf_pixmaps_arc);
                                            set_page_from_image_container_list(&image_container_list, &settings, &drawing_area_ref);
                                            *pages_info.loaded_filename.lock().unwrap() = Some(pathname.clone());
                                            update_window_title(&window, &pathname);


                                            spinner.stop();
                                            spinner.hide();
                                            
                                            pages_bar.set_fraction(0.0);
                                            pages_bar.set_inverted(true);
                                            // pages_bar.show();
                                            drawing_area_ref.queue_draw();                                            

                                            return glib::ControlFlow::Break;
                                        },
                                        2 => {
                                            spinner.stop();
                                            spinner.hide();

                                            update_window_title(&window, "Failed");
                                            return glib::ControlFlow::Break;
                                        }
                                        _ => {
                                            return glib::ControlFlow::Continue;
                                        }
                                    }
                                },
                                Err(_) => {
                                    return glib::ControlFlow::Continue;
                                }
                            }
                        }));
                    }));                    
                },
                _ => {
                    let Some(dir_path) = path.parent() else {
                        eprintln!("Failed get parent directory from path");
                        return;
                    };

                    let Some(dir_path_str) = dir_path.to_str() else {
                        eprintln!("Failed get dir path string");
                        return;
                    };
                    update_window_title(&window, dir_path_str);
                    *pages_info.loaded_dirname.lock().unwrap() = Some(dir_path_str.to_owned());

                    glib::spawn_future_local(glib::clone!(#[weak] window, #[strong] image_container_list, #[strong] settings, #[weak] drawing_area_ref, #[strong] pages_info, async move {
                        update_window_title(&window, "Now Loading...");

                        let r = read_dir_and_set_images(&file, &image_container_list).await;
                        if !r {
                            return;
                        }
                    
                        glib::idle_add_local_once(glib::clone!(#[strong] image_container_list, #[strong] settings, #[weak] drawing_area_ref, move || {
                            set_page_from_image_container_list(&image_container_list, &settings, &drawing_area_ref);
                            drawing_area_ref.queue_draw();
                        }));
                    }));
                }
            };            
        }
        file_dialog.close();
    }));

    
    dialog.show();
}

fn create_action_entry_for_menu(
    window: &gtk::ApplicationWindow,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    pages_info: &Arc<PagesInfo>,
    drawing_area_ref: &DrawingArea,
    pages_bar: &gtk::ProgressBar,
    settings: &std::sync::Arc<Settings>,
    spinner: &gtk::Spinner,
) -> Vec<gio::ActionEntry<gtk::Application>> {
    let file_action_entry: gio::ActionEntry<gtk::Application> = gio::ActionEntry::builder("file_open")
        .activate(glib::clone!(#[weak] window, #[strong] image_container_list,
            #[strong] pages_info, #[strong] settings, #[strong] drawing_area_ref, #[weak] pages_bar, #[weak] spinner, move |_app: &gtk::Application, _action: &gio::SimpleAction, _user_data: Option<&glib::Variant>| {
                open_file_action(&window, &image_container_list, &drawing_area_ref, &pages_bar, &settings, &pages_info, &spinner);
        }))
        .build();

    let quit_action_entry: gio::ActionEntry<gtk::Application> = gio::ActionEntry::builder("quit")
        .activate(glib::clone!(#[weak] window, move |app: &gtk::Application, action: &gio::SimpleAction, user_data: Option<&glib::Variant>| {
            app.quit();
    })).build();

    let result: Vec<gio::ActionEntry<gtk::Application>> =
        vec![file_action_entry, quit_action_entry];
    result
}

fn draw_single_page(
    image_container_list: &Vec<ImageContainer>,
    pages_info: &PagesInfo,
    area: &DrawingArea,
    ctx: &cairo::Context,
) {
    // let _index = _pages_info.current_page_index.as_ref().borrow().clone();
    let index = pages_info.current_page_index.lock().unwrap().clone();

    let Some(modified) = image_container_list[index].get_modified_pixbuf_data() else {
        return;
    };
    let format = if modified.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };
    let pix_w = modified.width();
    let pix_h = modified.height();
    let Ok(surface) = cairo::ImageSurface::create(format, pix_w, pix_h) else {
        return;
    };

    let margin = calc_margin_for_single(&modified, area.allocated_width(), area.allocated_height());
    let left_margin = f64::from(margin.left_margin);
    let top_margin = f64::from(margin.top_margin);

    let _ = ctx.set_source_surface(&surface, left_margin, top_margin);
    let _ = ctx.set_source_pixbuf(&modified, left_margin, top_margin);
    let _ = ctx.paint();
}

fn draw_dual_page(
    image_container_list: &Vec<ImageContainer>,
    pages_info: &PagesInfo,
    settings: &Settings,
    area: &DrawingArea,
    ctx: &cairo::Context,
) {
    // let _index = _pages_info.current_page_index.as_ref().borrow().clone();
    let index = pages_info.current_page_index.lock().unwrap().clone();
    let right_index = index;
    let left_index = index + 1;
    let half_area_width = area.allocated_width() / 2;

    let Some(right) = image_container_list[right_index].get_modified_pixbuf_data() else {
        return;
    };
    let right_format = if right.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };
    let pix_w = right.width();
    let pix_h = right.height();
    let Ok(surface_for_right) = cairo::ImageSurface::create(right_format, pix_w, pix_h) else {
        return;
    };

    let right_pos = if pix_w <= half_area_width {
        f64::from((half_area_width - pix_w) + pix_w)
    } else {
        0.0
    };

    if left_index >= image_container_list.len() {
        // FIXME: refelect page dirction. current is only support right to left.
        let margin = calc_margin_for_single(&right, area.allocated_width(), area.allocated_height());
        let top_margin = f64::from(margin.top_margin);

        let _ = ctx.set_source_surface(&surface_for_right, right_pos, top_margin);
        let _ = ctx.set_source_pixbuf(&right, right_pos, top_margin);
        let _ = ctx.paint();
        return;
    }

    let Some(left) = image_container_list[left_index].get_modified_pixbuf_data() else {
        let _ = ctx.set_source_surface(&surface_for_right, right_pos, 0.0);
        let _ = ctx.set_source_pixbuf(&right, right_pos, 0.0);
        let _ = ctx.paint();
        return;
    };
    let left_format = if left.has_alpha() {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };

    let margin = calc_margin_for_dual(
        &left,
        &right,
        area.allocated_width(),
        area.allocated_height(),
    );
    let left_margin = f64::from(margin.left_margin);
    let top_margin_for_left = f64::from(margin.top_margin_for_left);
    let top_margin_for_right = f64::from(margin.top_margin_for_right);

    let left_pic_width = left.width();
    let final_left_margin = if left_pic_width > half_area_width || left_pic_width == half_area_width
    {
        0.0
    } else {
        left_margin
    };

    let right_margin = if left_pic_width > half_area_width {
        final_left_margin + f64::from(left_pic_width)
    } else {
        left_margin + f64::from(left_pic_width)
    };

    let _ = ctx.set_source_surface(&surface_for_right, right_margin, top_margin_for_right);
    let _ = ctx.set_source_pixbuf(&right, right_margin, top_margin_for_right);
    let _ = ctx.paint();

    let Ok(surface_for_left) =
        cairo::ImageSurface::create(left_format, left.width(), left.height())
    else {
        return;
    };
    let _ = ctx.set_source_surface(&surface_for_left, final_left_margin, top_margin_for_left);
    let _ = ctx.set_source_pixbuf(&left, final_left_margin, top_margin_for_left);
    let _ = ctx.paint();
}

fn fullscreen(
    window: &gtk::ApplicationWindow,
    pages_bar: &gtk::ProgressBar,
) {
    if window.is_fullscreen() {
        window.unfullscreen();
        window.set_show_menubar(true);
        pages_bar.show();
    } else {
        window.fullscreen();
        window.set_show_menubar(false);
        pages_bar.hide();
    }
}

fn move_page(
    n: i32,
    settings: &Settings,
    drawing_area: &DrawingArea,
    pages_bar: &gtk::ProgressBar,
    image_container_list: &Arc<Mutex<Vec<ImageContainer>>>,
    pages_info: &Arc<PagesInfo>,
) {
    if n == 0 || (*image_container_list.lock().unwrap()).is_empty() {
        return;
    }

    let is_dual = *settings.is_dual_mode.lock().unwrap();
    let size = (*image_container_list.lock().unwrap()).len();
    let i = pages_info.current_page_index.lock().unwrap().clone();
    if i == 0 && n < 0 {
        return;
    }

    let mut result = if n > -1 {
        i + (n as usize)
    } else {
        i - (n.abs() as usize)
    };
    if size <= result {
        return;
    }

    if result >= size {
        result = size - 1;
    }

    let step_of_result = if is_dual { 2 } else { 1 };
    let tmp_step_left = if is_dual { (result + step_of_result) as f64 } else { (result + 1) as f64 };
    let tmp_step_right = size as f64;
    let step = if (result + step_of_result) >= size {
        1.0
    } else if result == 0 {
        0.0
    } else {
        tmp_step_left / tmp_step_right
    };
    // println!("result: {}", result);
    // println!("step: ${}", step);
    pages_bar.set_fraction(step);
    pages_bar.show();

    glib::spawn_future_local(glib::clone!(#[weak] pages_bar, async move {
        glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
            pages_bar.hide();
            glib::ControlFlow::Break
        });
    }));

    // _pages_info.current_page_index.replace(_result);
    *pages_info.current_page_index.lock().unwrap() = result;
    let height = drawing_area.allocated_height();
    let width = drawing_area.allocated_width();
    if *settings.is_dual_mode.lock().unwrap() {
        scale_page_for_dual(&image_container_list, result, width, height);
    } else {
        scale_page_for_single(&image_container_list, result, width, height);
    }

    drawing_area.queue_draw();
}

impl MainWindow {
    fn new() -> Self {
        let window_ui_src = include_str!("window.ui");

        let builder = gtk::Builder::new();
        let _ = builder.add_from_string(window_ui_src);

        let win = builder.object("window").unwrap();

        let result = MainWindow {
            window: win,
            v_box: gtk::Box::new(gtk::Orientation::Vertical, 1),
            image_container_list: Arc::new(Mutex::new(vec![])),
            pages_info: std::sync::Arc::new(PagesInfo::default()),
            settings: std::sync::Arc::new(Settings::default()),
            view_window: gtk::ScrolledWindow::new(),
        };

        result
    }

    fn init(&self, app: &Application, width: i32, height: i32) -> Result<()> {
        // let _header_bar = gtk::HeaderBar::builder().build();
        // self.window.set_titlebar(Some(&_header_bar));
        self.window.set_title(Some("Simple Comics Viewer"));
        self.window.set_default_size(width, height);
        self.window.set_show_menubar(true);

        let window = &self.window;
        let image_container_list = &self.image_container_list;
        let pages_info = &self.pages_info;
        let settings = &self.settings;
        *settings.is_dual_mode.lock().unwrap() = true;

        let menu_ui_src = include_str!("menu.ui");
        let builder = gtk::Builder::new();
        builder.add_from_string(menu_ui_src)?;
        let menu_model: gio::MenuModel = builder.object("menu").unwrap();
        // let _menu_button = gtk::MenuButton::builder()
        //     .menu_model(&_menu_model)
        //     .focus_on_click(true)
        //     .build();
        // _header_bar.pack_end(&_menu_button);

        let popover_menu = gtk::PopoverMenu::from_model(Some(&menu_model));
        app.set_menubar(Some(&popover_menu.menu_model().unwrap()));
        // let _history = _menubar.;
        // println!("{:?}", _history);
        // let _tmp_section = _history.unwrap().n_children();
        // println!("{}", _tmp_section);

        // let _popover_menu_bar = gtk::PopoverMenuBar::from_model(Some(&_menu_model));
        // let _menu_button = gtk::MenuButton::builder().label("M").build();

        let pages_bar = gtk::ProgressBar::builder()
            .build();
        pages_bar.hide();
        pages_bar.set_valign(gtk::Align::End);

        let spinner = gtk::Spinner::builder().build();
        spinner.hide();
        spinner.set_valign(gtk::Align::Center);

        let drawing_area = gtk::DrawingArea::builder()
            .hexpand_set(true)
            .vexpand_set(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .build();
        drawing_area.set_draw_func(glib::clone!(#[strong] image_container_list, #[strong] pages_info, #[strong] settings, move |area: &DrawingArea, ctx: &cairo::Context, width: i32, height: i32| {
            if (*image_container_list.lock().unwrap()).is_empty() {
                return;
            }

            if *settings.is_dual_mode.lock().unwrap() {
                draw_dual_page(&*image_container_list.lock().unwrap(), &pages_info, &settings, area, ctx);
            } else {
                draw_single_page(&*image_container_list.lock().unwrap(), &pages_info, area, ctx);
            }
        }));

        let _ = drawing_area.connect_resize(glib::clone!(#[strong] image_container_list, #[strong] pages_info, #[strong] settings, move|_drawing_area: &DrawingArea, width: i32, height: i32| {
            if (*image_container_list.lock().unwrap()).is_empty() { return; }
            
            let index = pages_info.current_page_index.lock().unwrap().clone();
            if *settings.is_dual_mode.lock().unwrap() {
                scale_page_for_dual(&image_container_list, index, width, height);
            } else {
                scale_page_for_single(&image_container_list, index, width, height);                
            }
        }));

        let event_controller_key = EventControllerKey::builder().build();
        let _ = event_controller_key.connect_key_pressed(glib::clone!(#[strong] app, #[strong] window, #[strong] image_container_list, #[strong] pages_info, #[strong] settings, #[strong] drawing_area, #[strong] pages_bar, #[strong] pages_info, #[strong] spinner, move |_event_controller_key: &EventControllerKey, keyval: gdk::Key, _keycode: u32, state: gdk::ModifierType| {
            
            if state == gdk::ModifierType::ALT_MASK && keyval == gdk::Key::Return {
                fullscreen(&window, &pages_bar);
                return Propagation::Stop;
            }

            if state == gdk::ModifierType::CONTROL_MASK && keyval == gdk::Key::o {
                open_file_action(&window, &image_container_list, &drawing_area, &pages_bar, &settings, &pages_info, &spinner);
                return Propagation::Stop;
            }

            let is_pressed_ctrl = state == gdk::ModifierType::CONTROL_MASK;
            let mut is_move = false;
            let mut additional_val = 0;
            match keyval {
                gdk::Key::q => {
                    if is_pressed_ctrl {
                        app.quit();
                    }
                },
                gdk::Key::Left => {
                    if *settings.is_dual_mode.lock().unwrap() {
                        // when right to left mode
                        additional_val = 2;
                    } else {
                        additional_val = 1;
                    }
                    is_move = true;
                },
                gdk::Key::h => {
                    if *settings.is_dual_mode.lock().unwrap() {
                        additional_val = 2;
                    } else {
                        additional_val = 1;
                    }
                    is_move = true;
                },
                gdk::Key::b => {
                    if is_pressed_ctrl {
                        if *settings.is_dual_mode.lock().unwrap() {
                            additional_val = 2;
                        } else {
                            additional_val = 1;
                        }
                        is_move = true;
                    }
                },
                gdk::Key::Right => {
                    if *settings.is_dual_mode.lock().unwrap() {
                        // when right to left mode
                        additional_val = -2;
                    } else {
                        additional_val = -1;
                    }
                    is_move = true;
                },
                gdk::Key::l => {
                    if *settings.is_dual_mode.lock().unwrap() {
                        additional_val = -2;
                    } else {
                        additional_val = -1;
                    }
                    is_move = true;
                },
                gdk::Key::f => {
                    if is_pressed_ctrl {
                        if *settings.is_dual_mode.lock().unwrap() {
                            additional_val = -2;
                        } else {
                            additional_val = -1;
                        }
                        is_move = true;
                    }
                },
                _ => is_move = false
            }
            if is_move {
                move_page(additional_val, &settings, &drawing_area, &pages_bar, &image_container_list, &pages_info);
            }
            Propagation::Stop
        }));
        self.window.add_controller(event_controller_key);

        self.view_window.set_hexpand(true);
        self.view_window.set_vexpand(true);
        self.view_window.set_halign(gtk::Align::Fill);
        self.view_window.set_valign(gtk::Align::Fill);

        let drawing_area_ref = &drawing_area;
        let pages_bar_ref = &pages_bar;
        let spinner_ref = &spinner;
        let action_entry = create_action_entry_for_menu(
            window,
            image_container_list,
            pages_info,
            drawing_area_ref,
            pages_bar_ref,
            settings,
            spinner_ref
        );
        app.add_action_entries(action_entry);
        self.view_window.set_child(Some(drawing_area_ref));

        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&self.view_window));
        overlay.add_overlay(spinner_ref);
        overlay.add_overlay(pages_bar_ref);


        self.v_box.set_halign(gtk::Align::Fill);
        self.v_box.set_valign(gtk::Align::Fill);
        self.v_box.set_hexpand(true);
        self.v_box.set_vexpand(true);
        self.v_box.append(&overlay);

        self.window.set_application(Some(app));
        // self.window.set_child(Some(&self.view_window));
        // self.window.set_child(Some(&self.v_box));
        self.window.set_child(Some(&self.v_box));

        Ok(())
    }

    fn run(&self) {
        self.window.show();
    }
}

pub fn activate(app: &Application) {
    let main = MainWindow::new();
    match main.init(app, 1024, 768) {
        Ok(_) => {
            main.run();
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
