use std::os::raw::c_void;
use winapi::windef::HWND;
use winapi::minwindef::HINSTANCE;
use std::io::Write;

use winapi;

//use graphics::*;
use Color;
use Component;

use std::sync::{Arc, Mutex};
use std::cell::{Cell, RefCell};
use std::borrow::BorrowMut;

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
    pos: (i32, i32),
    img: &Image
  ) {
    use gdi32::{CreateCompatibleDC, SelectObject, BitBlt, DeleteDC, GdiAlphaBlend};
    use std::os::raw::{c_int};

    unsafe {
      let hbm = img.winbm.get_hbitmap();
      let mem_hdc = CreateCompatibleDC(self.hdc);
      let old_hbm = SelectObject(mem_hdc, hbm as ::winapi::windef::HGDIOBJ);
      
      let (x, y, w, h) = img.area;

      // Estas constantes no están en winapi.rs

      // wine wingdi.h line 1996
      // #define AC_SRC_OVER  0x00
      // #define AC_SRC_ALPHA 0x01
      let AC_SRC_OVER = 0;
      let AC_SRC_ALPHA = 1;

      let result = GdiAlphaBlend(
        // Dest
        self.hdc,
        pos.0 as c_int,
        pos.1 as c_int,
        w, h,

        // Src
        mem_hdc,
        x, y, w, h,

        ::winapi::wingdi::BLENDFUNCTION{
          BlendOp: AC_SRC_OVER,
          BlendFlags: 0,
          SourceConstantAlpha: 255,
          AlphaFormat: AC_SRC_ALPHA,
        }
      );

      if result==0 {
        print_win_err("AlphaBlend at fill_image");
      }

      SelectObject(mem_hdc, old_hbm);
      DeleteDC(mem_hdc);
    }
  }

  pub fn fill_rect(&mut self,
    pos: (i32, i32),
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
        right: (pos.0 + size.0 as i32) as i32,
        bottom: (pos.1 + size.1 as i32) as i32
      };
      let brush = ::gdi32::CreateSolidBrush(color_i);
      ::user32::FillRect(self.hdc, &rect, brush);
    }
  }
}

// Nota: Esto no es Thread Safe, me fastidió hacerlo así
static mut CLASS_USERS: u32 = 0;
lazy_static!{
  static ref W_CLASS_NAME: Vec<u16> = to_wstring("MyComponentClass");
}

fn to_wstring(str: &str) -> Vec<u16> {
  use std::ffi::OsStr;
  use std::os::windows::ffi::OsStrExt;
  OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect()
}

fn repaint (hwnd: ::winapi::windef::HWND) {
  unsafe {
    ::user32::InvalidateRect(
      hwnd, ::std::ptr::null(), 0 as ::winapi::minwindef::BOOL
    );
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
    WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEMOVE,
  };

  // Trata de obtener la ventana asociada a ese HWND, si no puede imprime el
  // error y continúa normal.
  macro_rules! get_window {
    ($w:ident, $b:block) => {
      // Windows me garantiza que al principio, esto va a ser null
      // y cuando yo lo termine de usar, también lo pondré null
      let long = ::user32::GetWindowLongW(hwnd, ::winapi::winuser::GWLP_USERDATA);

      match (long as *mut Arc<Mutex<Component>>).as_mut() {
        None => printerr!("No graphics::Window asociated with current HWND"),
        Some(winarc) => match winarc.lock() {
          Err(_) => printerr!("Couldn't lock Window Mutex"),
          Ok(mut guard) => {
            let $w: &mut Component = &mut *guard;
            $b
          },
        }
      }
    }
  }

  macro_rules! mouse_ev {
    ($ev:ident, $btn:ident) => {
      get_window!(win, {
        win.event(::Event::$ev(::MouseBtn::$btn));
      });
    };
    ($ev:ident) => {
      let x = winapi::windowsx::GET_X_LPARAM(l_param) as i32;
      let y = winapi::windowsx::GET_Y_LPARAM(l_param) as i32;
      get_window!(win, {
        win.event(::Event::$ev(x, y));
      });
    };
  }

  match msg {

    WM_CREATE => {
      println!("Window Created");
      let lpcs = l_param as ::winapi::winuser::LPCREATESTRUCTW;

      let wbx_ptr = (*lpcs).lpCreateParams as *mut Arc<Mutex<Component>>;

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
      let win_box = Box::from_raw(long as *mut Arc<Mutex<Component>>);

      ::user32::SetWindowLongW(hwnd,
        ::winapi::winuser::GWLP_USERDATA,
        0 as ::winapi::winnt::LONG
      );
    },

    WM_PAINT => {
      get_window!(win, {
        paint_proc(hwnd, win);
      });
    },

    // Mensajes del Mouse

    WM_LBUTTONDOWN => { mouse_ev!(MouseDown, L); },
    WM_LBUTTONUP => { mouse_ev!(MouseUp, L); },

    WM_RBUTTONDOWN => { mouse_ev!(MouseDown, R); },
    WM_RBUTTONUP => { mouse_ev!(MouseUp, R); },

    WM_MBUTTONDOWN => { mouse_ev!(MouseDown, M); },
    WM_MBUTTONUP => { mouse_ev!(MouseUp, M); },

    WM_MOUSEMOVE => { mouse_ev!(MouseMove); },

    //TODO: Mensajes de IO
    _ => {}
  }

  return ::user32::DefWindowProcW(hwnd, msg, w_param, l_param);
}

#[allow(unused_variables)]
unsafe fn paint_proc (hwnd: ::winapi::windef::HWND, window: &mut Component) {
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

struct WinBitmap <'a> {
  hbitmap: *const c_void,
  phantom: ::std::marker::PhantomData<&'a c_void>
}
impl<'a> WinBitmap<'a> {
  fn new (hbm: ::winapi::windef::HBITMAP) -> WinBitmap<'a> {
    WinBitmap {
      hbitmap: hbm as *const c_void,
      phantom: ::std::marker::PhantomData
    }
  }
  fn get_hbitmap (&self) -> ::winapi::windef::HBITMAP {
    self.hbitmap as ::winapi::windef::HBITMAP
  }
}
impl<'a> Drop for WinBitmap<'a> {
  fn drop (&mut self) {
    unsafe { ::gdi32::DeleteObject(self.hbitmap as *mut c_void) };
  }
}

static mut EMPTY_ADDRESS: u16 = 0;

pub struct Image {
  pub width: u32,
  pub height: u32,
  winbm: Arc<WinBitmap<'static>>,
  area: (i32, i32, i32, i32),
  //img: ::image::DynamicImage
}

unsafe impl Send for Image { }

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

          // Aquí tengo que invertir 'y'
          let (r,g,b,a) = img.get_pixel(x,height-(y+1)).channels4();

          macro_rules! premul {
            ($a:expr, $b:expr) => ( (($a as u16*$b as u16) / 255) as u8 )
          }

          // Windows usa alfa premultiplicado
          pixels[i+0] = premul!(b, a);
          pixels[i+1] = premul!(g, a);
          pixels[i+2] = premul!(r, a);
          pixels[i+3] = a;
        }
      }
    }

    hbitmap
  }

  /// path_str debe ser relativo al dll
  pub fn load (path_str: &str) -> Option<Image> {
    use image::GenericImage;
    use winapi::windef::POINT;

    use std::path::PathBuf;

    //kernel32::GetModuleFileNameW

    let mut path_buf = {
      use std::ffi::OsString;
      use std::os::windows::ffi::OsStringExt;

      let mut hmodule = 0 as ::winapi::minwindef::HMODULE;

      let mut wstr = [0u16; 256];

      unsafe {
        // https://msdn.microsoft.com/en-us/library/windows/desktop/ms683200(v=vs.85).aspx
        let GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT = 0x02;
        let GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS = 0x04;

        ::kernel32::GetModuleHandleExW(
          GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
          GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
          &EMPTY_ADDRESS, &mut hmodule
        );
      };

      let size = unsafe{ ::kernel32::GetModuleFileNameW(
        hmodule,
        wstr.as_mut_ptr(),
        256
      ) };

      let os_str = OsString::from_wide(&wstr[0..(size as usize+1)]);
      PathBuf::from(os_str)
    }; // dll file path

    path_buf.pop(); // folder containing the dll
    path_buf.push(path_str);

    // Final resource path
    let path = path_buf.as_path();

    match ::image::open(path) {
      Err(_) => {
        printerr!("File {:?} not found", path);
        None
      },
      Ok(img) => {
        let hbitmap = Image::make_hbitmap(&img);
        if hbitmap.is_null() { None }
        else {
          let (width, height) = img.dimensions();
          Some( Image {
            width: width,
            height: height,
            winbm: Arc::new(WinBitmap::new(hbitmap)),
            area: (0, 0, width as i32, height as i32)
            //img: img,
          } )
        }
      }
    }
  }

  /// Rota la imagen con centro en el centro medio de la imagen (1 vuelta = 360°)
  pub fn rotate (mut self, angle: f32) -> Self {
    printerr!("Warning: Image Rotation not yet implemented.");
    self
  }

  pub fn crop (mut self, x: i32, y: i32, w: i32, h: i32) -> Self {
    let (_x, _y, _w, _h) = self.area;
    self.area = ( _x+x, _y+y, w, h, );
    self
  }
}

impl Clone for Image {
  fn clone (&self) -> Image {
    Image {
      width: self.width,
      height: self.height,
      winbm: self.winbm.clone(),
      area: self.area.clone(),
    }
  }
}

pub struct HandlerImpl {
  // Debe ser immutable y Sync

  // Option<Arc<Mutex<T>>> Simula un puntero a un T mutable concurrente que
  // puede ser null. Pero necesito poder cambiar el puntero de Window y
  // Handler debe ser immutable, así que lo encierraría en un Cell, pero al
  // mismo tiempo debe ser Sync para poder compartirse en threads, así que
  // en cambio lo encierro en un Mutex, que funciona como un RefCell.

  // Creo que aquí hay un leak, porque Window tiene un Handler.
  win: Option<Arc<Mutex<Component>>>,
  hwnd: HWND,
  width: i32,
  height: i32,
}

impl HandlerImpl {
  pub fn new () -> Self {
    HandlerImpl {
      win: None,
      hwnd: 0 as HWND,
      width: 0,
      height: 0,
    }
  }

  pub fn open (&mut self, ptr: *mut c_void) {
    // Already open
    if !self.hwnd.is_null() { return; }

    register_class();

    let syswnd = ptr as HWND;

    let hwnd = unsafe { ::user32::CreateWindowExW(
      0,
      W_CLASS_NAME.as_ptr(),
      to_wstring("Window").as_ptr(),
      {
        use winapi::winuser::*;
        WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS
      },
      0, 0, self.width, self.height,
      syswnd,
      0 as ::winapi::windef::HMENU,
      0 as HINSTANCE,
      ::std::ptr::null_mut()
      //winbox_ptr as *mut c_void
    ) };

    if hwnd.is_null() {
      print_win_err("Create Window at Handler.open");
      return;
    }

    self.hwnd = hwnd;
    self.attach_impl();
  }

  pub fn close (&mut self) {
    let result = unsafe { ::user32::DestroyWindow(self.hwnd) };
    if result == 0 {
      print_win_err("Destroy Window at Handler.close");
    } else {
      self.hwnd = 0 as HWND;
    }
  }

  pub fn repaint (&self) {
    unsafe {
      ::user32::InvalidateRect(
        self.hwnd, ::std::ptr::null(), 0 as ::winapi::minwindef::BOOL
      );
    }
  }

  pub fn capture (&self) {
    unsafe {
      ::user32::SetCapture(self.hwnd);
    }
  }

  pub fn release (&self) {
    unsafe {
      ::user32::ReleaseCapture();
    }
  }

  pub fn set_size (&mut self, w: usize, h: usize) {
    self.width = w as i32;
    self.height = h as i32;
  }

  fn attach_impl (&mut self) {
    match self.win {
      Some(ref win_arc) => {
        // Aquí, este win_ptr debe ser Send porque lo voy a enviar al
        // thread del GUI de Windows.
        let win_ptr = Box::into_raw(Box::new(win_arc.clone()));
        unsafe {
          // Debería ser SetWindowLongPtrW, pero rust winapi no lo tiene
          ::user32::SetWindowLongW(
            self.hwnd,
            ::winapi::winuser::GWLP_USERDATA,
            win_ptr as ::winapi::winnt::LONG
          );
        }
      },
      None => {}
    }
  }

  pub fn attach <T: Component + 'static> (&mut self, winx: Arc<Mutex<T>>) {
    // El uso de Send está en attach_impl
    //let winx = Arc::new(Mutex::new(win));
    self.win = Some(winx);
    self.attach_impl();
  }

  /*pub fn component (&self) -> Arc<Mutex<T>> {
    match &self.win {
      &Some(ref arc) => arc.clone(),
      &None => panic!("No associated component")
    }
  }*/
}