use vst2::editor::{Editor as VstEditor};

//use libc::c_void;

pub struct Editor {
  isopen: bool,
  class_name: Vec<u16>,
  width: i32,
  height: i32,
}

fn to_wstring(str : &str) -> Vec<u16> {
  use std::ffi::OsStr;
  use std::os::windows::ffi::OsStrExt;
  OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect()
}

impl Editor {
  pub fn new () -> Editor {
    Editor {
      isopen: false,
      class_name: to_wstring("SynthEditor"),
      width: 300,
      height: 100,
    }
  }

  #[allow(unused_variables)]
  unsafe fn paint_proc (hwnd: ::winapi::windef::HWND) {
    use winapi::winuser::PAINTSTRUCT;

    // puede ser zeroed o uninitialized, pero zeroed me gusta más
    let mut ps: PAINTSTRUCT = ::std::mem::zeroed();

    let hdc = ::user32::BeginPaint(hwnd, &mut ps);

    ::user32::EndPaint(hwnd, &mut ps);
  }

  unsafe extern "system" fn window_proc(
    hwnd:    ::winapi::windef::HWND, 
    msg:     ::winapi::minwindef::UINT,
    w_param: ::winapi::minwindef::WPARAM,
    l_param: ::winapi::minwindef::LPARAM )
    ->       ::winapi::minwindef::LRESULT
  {
    use winapi::winuser::{
      WM_CREATE, WM_PAINT, WM_DESTROY,
    };
    match msg {
      WM_CREATE =>  println!("Window Created"),
      WM_PAINT => Self::paint_proc(hwnd),
      WM_DESTROY => println!("Window Destroyed"),
      _ => {}
    }
    return ::user32::DefWindowProcW(hwnd, msg, w_param, l_param);
  }

  fn register_class (&self) {
    use winapi::minwindef::HINSTANCE;
    use winapi::winuser::WNDCLASSW;
    use winapi::winnt::LPCWSTR;

    unsafe {
      let wnd = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(Self::window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: 0 as HINSTANCE,
        hIcon: 0 as ::winapi::windef::HICON,
        hCursor: 0 as ::winapi::windef::HCURSOR,
        hbrBackground: 16 as ::winapi::windef::HBRUSH,
        lpszMenuName: 0 as LPCWSTR,
        lpszClassName: self.class_name.as_ptr() as LPCWSTR,
      };

      let result = ::user32::RegisterClassW(&wnd);

      if result == 0 {
        println!("WindowsClass Registration Failed: {:?}", ::std::io::Error::last_os_error());
      }
    }
  }

  fn load_resources (&mut self) {

  }

  #[allow(unused_variables)]
  fn open_windows (&mut self, syswnd_p: *mut ::libc::c_void) {
    use winapi::minwindef::HINSTANCE;
    use winapi::windef::HWND;

    unsafe {
      let syswnd = syswnd_p as HWND;

      let hwnd = ::user32::CreateWindowExW(
        0,
        self.class_name.as_ptr(),
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
      );

    }
  }
}

impl VstEditor for Editor {
  fn size (&self) -> (i32, i32) { (self.width, self.height) }
  fn position (&self) -> (i32, i32) { (0, 0) }
  fn is_open (&mut self) -> bool { self.isopen }

  fn open (&mut self, window: *mut ::libc::c_void) {
    self.register_class();
    self.load_resources();
    self.open_windows(window);
    self.isopen = true;
  }
}