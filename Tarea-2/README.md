# README — Simulación de Fábrica con Planificadores (FCFS y Round Robin)

## Descripción
Este proyecto simula una línea de producción con **tres estaciones** (Corte → Ensamblaje → Empaque) y compara dos algoritmos de planificación:
- **FCFS** (First-Come, First-Served)
- **Round Robin** con *quantum* configurable (en milisegundos)

Cada estación corre en su **propio hilo**, recibe productos por canales `mpsc` y utiliza un **scheduler** interno para decidir cuánto procesar en cada turno. Al final, se calculan y muestran **estadísticas**: tiempos promedio de espera y *turnaround*, orden de finalización y detalle por producto.

---

## Requisitos
- **Rust** 1.75+ (recomendado) con `cargo`  
  Instala desde: <https://www.rust-lang.org/tools/install>

---

## Ejecución
En la raíz del proyecto:

```bash
# Compilar y ejecutar en modo debug
cargo run

# (Opcional) Modo optimizado
cargo run --release
```

La simulación corre dos veces:
1) **FCFS**
2) **Round Robin** con `quantum_ms: 750`

Se ve en consola las trazas por estación y al final un **resumen de estadísticas**.

---

## Archivos principales
```
src/
  main.rs        # Punto de entrada; genera llegadas y corre ambas simulaciones
  factory.rs     # Orquesta estaciones/hilos, canales y recolección de estadísticas
  scheduler.rs   # Implementa FCFS y Round Robin
  product.rs     # Modelo de producto y utilidades de tiempo
```

---

## Parámetros clave (dónde ajustar)
- **Número de espacios en cola (capacidad):** en `Factory::new(capacity, ...)`
- **Tiempos por estación (ms):** en `StationTimes::default()` de `factory.rs`  
  Por defecto: `cutting=2000`, `assembly=3000`, `packaging=1000`  
  Puedes usar `Factory::new_with_times(capacity, algorithm, StationTimes { ... })`
- **Algoritmo y *quantum*:** en `main.rs`  
  ```rust
  run_simulation(SchedulingAlgorithm::FCFS);
  run_simulation(SchedulingAlgorithm::RoundRobin { quantum_ms: 750 });
  ```
- **Patrón de llegadas:** vector `arrival_intervals` en `main.rs`  
  (ms entre envíos; actualmente 10 llegadas con intervalos escalonados)

---

## ¿Cómo funciona internamente?
- **Pipeline de 3 estaciones** con **canales síncronos** (`mpsc::sync_channel`) para backpressure.
- Cada estación crea su **Scheduler** (según el algoritmo elegido):
  - **FCFS:** procesa el trabajo **completo** y lo pasa a la siguiente estación.
  - **Round Robin:** procesa por **quantum** y, si no termina, **devuelve** el trabajo al final de la cola con el tiempo restante.
- Los productos registran:
  - `arrival_time`, `entry_*`, `exit_*` por estación
  - tiempo acumulado en cada etapa (`accumulated_*_ms`)
- Al final, `StatsCollector` calcula:
  - **Tiempo de espera (waiting):** tiempo en colas (no procesando) antes de cada etapa
  - **Turnaround:** desde llegada hasta salida de Empaque

---

## Salida esperada
Durante la corrida verás líneas como:
```
 Product 3 arrived at 800ms
 Product 3 procesando en Corte (750ms, acumulado: 1250ms)
 Product 3 interrumpido en Ensamblaje (quedan 1500ms)
 Product 7 TERMINADO
```

Y al final:
```
=== RESUMEN DE ESTADÍSTICAS ===
Algoritmo: RoundRobin { quantum_ms: 750 }
Total de productos procesados: 10
 Tiempo promedio de espera: 2.31s
 Tiempo promedio de turnaround: 7.05s
 Orden final de procesamiento:
 1. Product 4
 2. Product 1
 ...
 Detalle por producto:
 Product 1: Espera = 2.15s, Turnaround = 6.90s
 ...
```

> **Nota:** Los valores dependen de los **tiempos de estación**, del **quantum** y del **patrón de llegadas**.