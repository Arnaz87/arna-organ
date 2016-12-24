
## FL Studio Reverb

El órgano debe tener un efecto donde simulo un cuarto. Vi que el plugin Reverb2 de Fl Studio hace un muy genial trabajo, así que estoy estudiando cómo se comporta para tratar de recrearlo aquí.

Por lo que he visto se divide en varias fases:

La primera fase es convertir la señal en Mono. También aparentemente una muestra influye en las siguientes ~30 muestras siguientes, es decir, si le doy un pulso de [10, 0, 0, 0, 0], sale un pulso de [10, 6, 4, 2, 1].

No estoy seguro de si esta Influencia se aplica después de la primera fase o después de la segunda, porque con el tamaño pequeño parece que los pulsos de la segunda fase se afectan entre ellos (Los primeros afectan a los siguientes). Aunque probablemente eso simplemente sea porque el un impulso empieza cuando otro no ha terminado y sus volúmenes simplemente se suman, así que creo que esto sí pasa en la primera fase.

(NOTA: Esta "influencia" es un filtro de paso bajo ...)

La segunda fase es un delay controlado por el tamaño, son unos pulsos simples. El formato es: Lado (L R), Momento (Milésimas de segundo, ej 0.001s = 1ms) y Volumen (En porcentaje con respecto al sonido original).

Los pulsos con el tamaño máximo:

- L 96ms 100%
- R 104ms 100%
- R 163ms 60%
- L 181ms -60%
- R 239ms -30%
- L 260ms 30%.

Los pulsos con el tamaño mínimo:

- L 13.5ms 100%
- R 15.5ms 100%
- R 22.7ms 60%
- L 25.1ms -60%
- R 33.3ms -45%
- L 36.2ms 45%
- R 43.2ms 15%
- L 45.2ms 15%
- L 54.0ms -12%
- R 55.25ms -12%
- L 64.6ms 10%
- R 65.4ms 10%
- L 76.5ms -5%
- R 77.75ms -5%
- R 88.5ms 2%
- L 89.5ms 2%
- R 98.8ms 1%
- L 99.25ms -1%
- R 107.3ms -.8%
- L 112.3ms .8%
- L 118.3ms .4%
- R 119.3ms .4%

La tercera fase parece un montón de pulsos aleatorios. Parece como una forma de onda definida (como un golpe metálico) y cada muestra de esa forma de onda indica el volumen del pulso en esa posición multiplicado por la difusión, pero cuando la difusión esta en 100 (e incluso desde que está por los 50) reemplaza a los pulsos que describí arriba. El tamaño modifica la duración y el tono de la forma de onda. Como dije ya, la difusión multiplica el volumen de cada pulso, pero parece que no es lineal sino sigmoide.

## Análisis de los pulsos en difusión 0

Son pares de pulsos, un pulso siempre va a la izquierda y otro a la derecha, alternándose cuál va primero, algunos a veces tienen la fase invertida también, y los pares van bajando de volumen. Voy a analizar los del tamaño mínimo, porque son más.

Aquí hay una lista de los pares, indicando cuánto bajaron de volumen, cuánto tiempo hay de diferencia con el anterior, y cuánto tiempo hay de diferencia entre los pulsos del par:

- 13.5  2     1
- 9.2   2.4   0.6
- 10.6  2.9   0.75
- 9.9   2.0   0.'3
- 10.8  1.25  0.8
- 10.6  0.8   0.8'3
- 11.9  1.25  0.5
- 12    1     0.4
- 10.3  0.45  0.5
- 8.5   5     0.8
- 11    1     0.5

Grafiqué estos datos, y no hay ningún patrón visible, aunque de lo que he leído de internet, es común que los pulsos sean múltiplos e números primos

Con cuál lado empieza cada par: LRRRLLLRRRL, aparentemente empieza conL y luego repite el mismo 3 veces.
Cuáles pulsos son positivos y negativos: +++--+++--++--+++--+++, son series de tres positivos y dos negativos (excepto en la tercera serie que son solo dos positivos).

## Click de AZR3

El click de azr3 simplemente el un bandpass de una señal constante (no de una onda, una constante), y la frecuencia de ese bandpass es determinada por la nota, pero no es necesariamente la frecuencia de la nota (de hecho, específicamente es (f+70)*16). Según mi análisis, un paso banda a una constante es básicamente una onda senoidal, así que simplemente voy a hacer eso. Pero el click de AZR3, por lo que se ve en la onda, tiene una pequeña disturbación aleatoria, que no sé cómo la consigue, no sé cómo replicarla.

## Último análisis

No entendía como funcionan los reverberadores, pero ahora lo sé e hice otro análisis a FL Studio.

Antes que todo se convierte en Mono, y esa señal es la de los ecos
Después son los pulsos, en eso estuve bien, los pulsos son los mismos.

La tercera fase son los ecos con feedback (los filtros comb con feedback) mas dos ecos antes de la señal original. Las configuraciones son: Para los comb, 235, 313, 610, 835 muestras de retardo (en tamaño máximo), y para los pre-ecos 313 y 835 muestras de adelanto. Obviamente no puedo hacer un eco sin haber visto la señal original, así que la señal principal se retrasa 835 muestras, el primer pre-eco con cero retardo (835 adelantado a la señal principal), y el segundo con 835-313=522 retardo (313 de adelanto), a la principal es a la que se le aplican los Comb.

Hay una cuarta fase de ecos para hacer el resultado aún más ancho, pero no logro entender cómo está hecha, porque ya con la anterior hay muchísimos ecos y es difícil distinguir cual micro eco es de cuál señal.

Lo otro es que la tercera fase retarda todos los pulsos, y yo medí la posición final de todos, después de haber aplicado todas las fases (estoy seguro de que la cuarta fase retrasa aún más), así que la posición original seguro es mucho antes (al menos 835 muestras antes), y tengo que acomodar eso.

235:  5.3288 ms
313:  7.0975 ms
610: 13.8322 ms
835: 18.9342 ms
