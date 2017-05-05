
/* Esto es extremadamente importante
 * Mingw de linux por defecto viene con mecanismos y librerías diferentes a las
 * que rust espera e instala por defecto en windows, por eso hay que descargar,
 * con rustup, el toolchain de windows con todas las librerías necesarias.
 * Luego hay que indicarle al linker que use el path de esas librerías, este
 * archivo es la manera de hacerlo.
 * También es muy importante que se agreguen estas líneas a .cargo/config:

  linker = "i686-w64-mingw32-gcc"
  ar = "i686-w64-mingw32-ar"
  rustflags = ["-C", "panic=abort"]

 * Obviamente todo esto depende de cómo esté configurado el sistema en cuestión
 * y qué nombres y directorios usen todos los programas involucrados, que son
 * mingw, cargo y rustup, y en menor medida gcc y rustc (estos dos aún no
 * me han dado problemas)
 */

fn main () {
  println!("cargo:rustc-link-search=/home/arnaud/.rustup/toolchains/stable-i686-pc-windows-gnu/lib/rustlib/i686-pc-windows-gnu/lib");
}