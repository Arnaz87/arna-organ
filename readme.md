# Organ

Un Vst sintetizador de órgano hecho en Rust.
Se debe descargar savihost.exe y mover una copia a la carpeta principal del proyecto

### Nota

El tubo como es soplado, tiene un vientico. Este vientico es una onda seno a una frecuencia de 2.65 respecto a la principal del tubo. Es muy corta y viene con un ruido también.

La frecuencia 2.65 es la armónica 43 de la frecuencia 3 octavas bajo la principal.

# Bugs

Si elimino todos los órganos que están en el proyecto, no puedo abrir ningun otro, esto es por que la clase de windows no se deregistra, y esto a su vez porque el window_handler tiene una referencia circular a sí mismo que hace que Rust nunca lo elimine.


# Próxima versión

- El aspecto visual debe ser más digital, oscuro y plano, el de ahorita es muy analógico y claro.
- Sintetizador substractivo.
  + Waveform (Saw -> Square, morphable, formas del Minimoog)
  + Amp ADSR
  + Filter ADSR
  + Filter Type (Opcional)
  + Unison (Parámetros ¿?)
- Leslie
  + Velocidad Superior
  + Velocidad Inferior
  + Tamaño (Opcional)
  + Distancia
  + Split Freq (Opcional)
- Ambiance
  + Size
  + Diffusion
  + Decay (segundos)
  + Damp
  + Mix
- Distort (Opcional)
  + Color
  + Intensidad
- Tubos
  + Harmonic
  + Color (Cold -> Warm)
  + Breath (Opcional)
  + Attack
  + Decay
- Hammond
  + Forma (Sine -> Saw)
  + Decay
  + Click
- Perc
  + Harmonic
  + Intensity (Volumen y Decay en uno solo)
