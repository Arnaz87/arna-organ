use std::os::raw::c_void;
use winapi::windef::HWND;
use winapi::minwindef::HINSTANCE;
use std::io::Write;

//use graphics::*;
use Color;
use Window;

use std::sync::{Arc, Mutex, MutexGuard};

pub struct Canvas {
  hdc: ::winapi::windef::HDC
}

macro_rules! printerr(
  ($($arg:tt)*) => { {
    let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
    r.expect("failed printing to stderr");
  } }
);

fn print_win_err (msg: &str) {
  printerr!("[{}] {} Fail: {:?}", file!(), msg, ::std::io::Error::last_os_error());
}

impl Canvas {
  pub fn fill_image (&mut self,
    pos: (u32, u32),
    img: &Image
  ) {

    use gdi32::{CreateCompatibleDC, SelectObject, BitBlt, DeleteDC};
    use std::os::raw::{c_void, c_int};

    unsafe {
      let mem_hdc = CreateCompatibleDC(self.hdc);
      let old_hbm = SelectObject(mem_hdc, img.hbm as ::winapi::windef::HGDIOBJ);

      let result = BitBlt(
        self.hdc,
        pos.0 as c_int,
        pos.1 as c_int,
        img.width as c_int,
        img.height as c_int,
        mem_hdc,
        0, 0, // Source x y
        ::winapi::wingdi::SRCCOPY
      );

      SelectObject(mem_hdc, old_hbm);
      DeleteDC(mem_hdc);
    }
  }

  pub fn fill_rect(&mut self,
    pos: (u32, u32),
    size: (u32, u32),
    color: Color
  ) {
    let color_i = //0x00bbggrr
      ((color.r as u32) << 0) +
      ((color.g as u32) << 8) +
      ((color.b as u32) << 16);

    //println!("Color {:?} {:08x}", color, color_i);

    unsafe {
      let rect = ::winapi::windef::RECT{
        left: pos.0 as i32,
        top: pos.1 as i32,
        right: (pos.0 + size.0) as i32,
        bottom: (pos.1 + size.1) as i32
      };
      let brush = ::gdi32::CreateSolidBrush(color_i);
      ::user32::FillRect(self.hdc, &rect, brush);
    }
  }
}

// Nota: Esto no es Thread Safe, me ladilló hacerlo así
static mut CLASS_USERS: u32 = 0;
lazy_static!{
  static ref W_CLASS_NAME: Vec<u16> = to_wstring("MyWindowClass");
}

fn to_wstring(str : &str) -> Vec<u16> {
  use std::ffi::OsStr;
  use std::os::windows::ffi::OsStrExt;
  OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect()
}

unsafe fn get_window<'a> (hwnd: ::winapi::windef::HWND) -> Option<MutexGuard<'a, Window>> {
  // Windows me garantiza que al principio, esto va a ser null
  // y cuando yo lo termine de usar, también lo pondré null

  let long = ::user32::GetWindowLongW(hwnd, ::winapi::winuser::GWLP_USERDATA);

  match (long as *mut Arc<Mutex<Window>>).as_mut() {
    None => {
      printerr!("No graphics::Window asociated with current HWND");
      None
    },
    Some(winarc) => match winarc.lock() {
      Ok(win) => Some(win),
      Err(_) => None,
    }
  }
}

unsafe extern "system" fn window_proc(
  hwnd:    ::winapi::windef::HWND, 
  msg:     ::winapi::minwindef::UINT,
  w_param: ::winapi::minwindef::WPARAM,
  l_param: ::winapi::minwindef::LPARAM)
  ->       ::winapi::minwindef::LRESULT
{
  use winapi::winuser::{
    WM_CREATE, WM_PAINT, WM_DESTROY,
  };

  match msg {

    WM_CREATE => {
      println!("Window Created");
      let lpcs = l_param as ::winapi::winuser::LPCREATESTRUCTW;

      let wbx_ptr = (*lpcs).lpCreateParams as *mut Arc<Mutex<Window>>;

      // Debería ser SetWindowLongPtrW, pero no existe en el crate
      ::user32::SetWindowLongW(hwnd,
        ::winapi::winuser::GWLP_USERDATA,
        wbx_ptr as ::winapi::winnt::LONG
      );
    },

    WM_DESTROY => {
      println!("Window Destroyed");

      let long = ::user32::GetWindowLongW(hwnd, ::winapi::winuser::GWLP_USERDATA);

      // Recuperar la caja. Cuando salga de contexto, la caja se elimina y
      // elimina la referencia que está usando en el Arc.
      let win_box = Box::from_raw(long as *mut Arc<Mutex<Window>>);

      ::user32::SetWindowLongW(hwnd,
        ::winapi::winuser::GWLP_USERDATA,
        0 as ::winapi::winnt::LONG
      );
    },

    WM_PAINT => {
      match get_window(hwnd) {
        None => println!("No Window to paint"),
        Some(mut win) => {
          paint_proc(hwnd, &mut *win);
        }
      }
    },

    //TODO: Mensajes de IO
    _ => {}
  }

  return ::user32::DefWindowProcW(hwnd, msg, w_param, l_param);
}

#[allow(unused_variables)]
unsafe fn paint_proc (hwnd: ::winapi::windef::HWND, window: &mut Window) {
  use winapi::winuser::PAINTSTRUCT;

  let mut ps: PAINTSTRUCT = ::std::mem::zeroed();
  let hdc = ::user32::BeginPaint(hwnd, &mut ps);

  if hdc.is_null() {
    print_win_err("Begin Paint at paint_proc");
  } else {
    let mut canvas = Canvas {
      hdc: hdc
    };

    window.paint(&mut canvas);

    ::user32::EndPaint(hwnd, &mut ps);
  }

}

fn register_class () {
  use winapi::winuser::WNDCLASSW;
  use winapi::winnt::LPCWSTR;

  // Esto no es Thread Safe
  let class_users = unsafe {CLASS_USERS += 1; CLASS_USERS};

  if class_users == 1 {
    println!("Registering Class");

    let wnd = WNDCLASSW {
      style: 0,
      lpfnWndProc: Some(window_proc),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: 0 as HINSTANCE,
      hIcon: 0 as ::winapi::windef::HICON,
      hCursor: 0 as ::winapi::windef::HCURSOR,
      hbrBackground: 16 as ::winapi::windef::HBRUSH,
      lpszMenuName: 0 as LPCWSTR,
      lpszClassName: W_CLASS_NAME.as_ptr() as LPCWSTR,
    };

    let result = unsafe { ::user32::RegisterClassW(&wnd) };

    if result == 0 {
      print_win_err("Window Class Registration");
    }
  }
}

fn unregister_class () {
  let class_users = unsafe {CLASS_USERS -= 1; CLASS_USERS};

  if class_users == 0 {
    println!("Unregistering Class");

    let result = unsafe { ::user32::UnregisterClassW(
      W_CLASS_NAME.as_ptr() as ::winapi::winnt::LPCWSTR,
      0 as HINSTANCE
    ) };

    if result == 0 {
      print_win_err("Window Class Unregistration");
    }
  }
}

fn make_window<'a> (win_arc: Arc<Mutex<Window>>, syswnd: HWND) {

  let (width, height) = {
    let win = win_arc.lock().unwrap();
    (*win).get_size()
  };

  // Esto crea un puntero independiente en la memoria
  let winbox_ptr: *mut Arc<Mutex<Window>> = Box::into_raw(Box::new(win_arc));

  let hwnd = unsafe { ::user32::CreateWindowExW(
    0,
    W_CLASS_NAME.as_ptr(),
    to_wstring("Window").as_ptr(),
    {
      use winapi::winuser::*;
      WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS
    },
    0, 0, width as i32, height as i32,
    syswnd,
    0 as ::winapi::windef::HMENU,
    0 as HINSTANCE,
    winbox_ptr as *mut c_void
  ) };

  if hwnd.is_null() {
    print_win_err("Create Window at make_window");
  }
}

pub struct Image {
  width: u32,
  height: u32,
  hbm: ::winapi::windef::HBITMAP,
  img: ::image::DynamicImage
}

impl Image {

  fn make_hbitmap (img: &::image::DynamicImage) -> ::winapi::windef::HBITMAP {
    use std::mem::size_of;

    use image::{GenericImage, Pixel};

    use winapi::wingdi::{BITMAPINFO, BITMAPINFOHEADER};
    use winapi::winnt::{LONG};
    use winapi::minwindef::DWORD;

    let (width, height) = img.dimensions();

    let header = BITMAPINFOHEADER {
      biSize: size_of::<BITMAPINFOHEADER>() as DWORD,
      biWidth: width as LONG,
      biHeight: height as LONG, // Positivo: bottom-up, Negativo: top-down
      biPlanes: 1,
      biBitCount: 32,
      biCompression: ::winapi::wingdi::BI_RGB,

      // El resto de los campos no son importantes
      biSizeImage: 0, // This may be set to zero for BI_RGB bitmaps.
      biXPelsPerMeter: 0,
      biYPelsPerMeter: 0,
      biClrUsed: 0, // If zero, uses maximum number of colors.
      biClrImportant: 0, // If zero, all colors are required.
    };

    let bminfo = BITMAPINFO {
      bmiHeader: header,
      bmiColors: []
    };

    let mut bits: *mut c_void = ::std::ptr::null_mut();

    let hbitmap = unsafe {
      ::gdi32::CreateDIBSection(
        0 as ::winapi::windef::HDC,
        &bminfo,
        ::winapi::wingdi::DIB_RGB_COLORS,
        &mut bits,
        0 as ::winapi::winnt::HANDLE,
        0
      )
    };

    if !bits.is_null() {

      let pixels: &mut [u8] = unsafe {
        let ptr = bits as *mut u8;
        let len = (width*height*4) as usize;
        ::std::slice::from_raw_parts_mut(ptr, len)
      };

      // Windows va de abajo hacia arriba
      for y in 0..height {
        for x in 0..width {
          let i = ((y*width + x)*4) as usize;

          // Aquí tengo que invertir y
          let (r,g,b,a) = img.get_pixel(x,height-(y+1)).channels4();

          pixels[i+0] = r;
          pixels[i+1] = g;
          pixels[i+2] = b;
          pixels[i+3] = a;
        }
      }
    }

    hbitmap
  }

  pub fn load (path_str: &str) -> Option<Image> {
    use image::GenericImage;

    let path = ::std::path::Path::new(path_str);
    match ::image::open(path) {
      Err(_) => None,
      Ok(img) => {
        let hbitmap = Image::make_hbitmap(&img);
        if hbitmap.is_null() { None }
        else {
          let (width, height) = img.dimensions();
          Some( Image {
            width: width,
            height: height,
            hbm: hbitmap,
            img: img,
          } )
        }
      }
    }
  }
}

impl Drop for Image {
  fn drop (&mut self) {
    unsafe { ::gdi32::DeleteObject(self.hbm as *mut c_void) };
  }
}

pub fn register_window<W: Window + 'static> (win: W, ptr: *mut c_void) -> Arc<Mutex<Window>> {
  register_class();
  let data: Arc<Mutex<Window>> = Arc::new(Mutex::new(win));
  make_window(data.clone(), ptr as HWND);
  data.clone()
}