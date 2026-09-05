use base64::{engine::general_purpose::STANDARD, Engine};
use gtk::prelude::*;
use spotlight_core::model::{Entry, Kind};

/// Resolve the OS icon only for visible search results. GTK decodes it to a
/// bounded PNG, so the webview never receives arbitrary filesystem access.
pub fn resolve(entry: &Entry) -> Option<String> {
    let icon = match entry.kind {
        Kind::App => gio::DesktopAppInfo::new(&entry.path)?.icon()?,
        Kind::Folder => gio::content_type_get_icon("inode/directory"),
        Kind::File => {
            let (content_type, _) = gio::content_type_guess(Some(&entry.path), &[]);
            gio::content_type_get_icon(&content_type)
        }
    };
    let theme = gtk::IconTheme::default()?;
    let info = theme.lookup_by_gicon(&icon, 64, gtk::IconLookupFlags::FORCE_SIZE)?;
    let pixbuf = info.load_icon().ok()?;
    let scaled = pixbuf.scale_simple(64, 64, gtk::gdk_pixbuf::InterpType::Bilinear)?;
    let png = scaled.save_to_bufferv("png", &[]).ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}
