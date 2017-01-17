
pub struct Color {
  r: i8,
  g: i8,
  b: i8,
  a: i8,
}

pub trait Context {
  
}

pub enum InputEvent {}

pub trait Window {
  fn get_size (&self) -> (u32, u32);
  fn paint (&self, ctx: &mut Context);
  fn input (&mut self, ev: InputEvent);
}

#[cfg(windows)]
mod windows {

  use libc::c_void;
  use winapi::windef::HWND;
  use winapi::minwindef::HINSTANCE;

  use graphics::*;

  struct WinContext {
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

  fn print_error (msg: &str) {
    println!("{} Fail: {:?}", msg, ::std::io::Error::last_os_error());
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
        // Aquí se debe recibir un puntero a la ventana, y se debe asociar con
        // SetWindowLongPtr (hwnd, GWLP_USERDATA, (LONG_PTR)self);
        println!("Window Created")
      },
      WM_PAINT => paint_proc(hwnd),
      WM_DESTROY => println!("Window Destroyed"),
      _ => {}
    }
    return ::user32::DefWindowProcW(hwnd, msg, w_param, l_param);
  }

  #[allow(unused_variables)]
  unsafe fn paint_proc (hwnd: ::winapi::windef::HWND) {
    use winapi::winuser::PAINTSTRUCT;
    //use winapi::windef::HBRUSH;

    // puede ser zeroed o uninitialized
    let mut ps: PAINTSTRUCT = ::std::mem::zeroed();
    let hdc = ::user32::BeginPaint(hwnd, &mut ps);

    let brush = ::gdi32::CreateSolidBrush(0xff); //0x00bbggrr
    let rect = ::winapi::windef::RECT{ left:50, top:50, right:100, bottom: 100 };

    ::user32::FillRect(hdc, &rect, brush);

    ::user32::EndPaint(hwnd, &mut ps);
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
        print_error("Window Class Registration");
      }
    }
  }

  fn unregister_clas () {
    let class_users = unsafe {CLASS_USERS -= 1; CLASS_USERS};

    if class_users == 0 {
      println!("Unregistering Class");
      unsafe {
        let result = ::user32::UnregisterClassW(
          W_CLASS_NAME.as_ptr() as ::winapi::winnt::LPCWSTR,
          0 as HINSTANCE
        );

        if result == 0 {
          print_error("Window Class Unregistration");
        }
      }
    }
  }

  #[allow(unused_variables)]
  fn make_window (win: &mut Window, syswnd: HWND) {

    let (width, height) = win.get_size();

    unsafe {
      let hwnd = ::user32::CreateWindowExW(
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
        ::std::ptr::null_mut() // Aquí debe ir un puntero a la ventana
      );

      if hwnd.is_null() {
        print_error("Creating Window");
      }
    }
  }

  pub fn register_window (win: &mut Window, ptr: *mut c_void) -> Result<&mut Context, ()> {
    register_class();
    make_window(win, ptr as HWND);
    Err(())
  }
}

#[cfg(windows)]
pub use self::windows::register_window;