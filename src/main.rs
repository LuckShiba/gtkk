use std::rc::Rc;

use futures::AsyncReadExt;
use gtk4::{
    gdk_pixbuf::Pixbuf,
    gio::{MemoryInputStream, NONE_CANCELLABLE},
    glib::{clone, Bytes, MainContext},
    prelude::*,
    Application, ApplicationWindow, Button, HeaderBar, Image, Orientation, Window,
};
use isahc::AsyncReadResponseExt;

fn main() {
    let application = Application::builder()
        .application_id("dev.luckshiba.shiba.images")
        .build();
    application.connect_activate(|application| {
        futures::executor::block_on(build_ui(application));
    });
    application.run();
}

async fn build_ui(application: &Application) {
    let container = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let image = Image::from_pixbuf(create_pixbuf().await.as_ref());
    image.set_pixel_size(400);
    container.append(&image);
    let reload_button = Button::builder().icon_name("view-refresh").build();
    let bar = HeaderBar::new();
    bar.pack_start(&reload_button);

    let window = Rc::new(
        ApplicationWindow::builder()
            .title("shiba")
            .default_width(1000)
            .default_height(600)
            .child(&container)
            .visible(true)
            .resizable(false)
            .build(),
    );

    window.set_titlebar(Some(&bar));
    window.set_application(Some(application));

    reload_button.connect_clicked(clone!(@strong window =>
        move |_| {
            MainContext::default().spawn_local(dialog(Rc::clone(&window)));
        }
    ));
}

async fn dialog<W: IsA<Window>>(window: Rc<W>) {
    let image: Image = window
        .child()
        .unwrap()
        .last_child()
        .unwrap()
        .downcast()
        .unwrap();
    image.set_from_pixbuf(create_pixbuf().await.as_ref())
}

async fn create_pixbuf() -> Option<Pixbuf> {
    let body = isahc::get_async("https://shibe.online/api/shibes")
        .await
        .ok()?
        .json::<Vec<String>>()
        .await
        .ok()?;
    let image = body.first().unwrap();
    let mut res = isahc::get_async(image).await.ok()?;
    let body = res.body_mut();
    let mut bytes_vec = Vec::new();
    body.read_to_end(&mut bytes_vec).await.ok();
    let bytes = Bytes::from(&bytes_vec);
    let stream = MemoryInputStream::from_bytes(&bytes);
    Pixbuf::from_stream(&stream, NONE_CANCELLABLE).ok()
}
