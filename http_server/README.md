# HTTP Server - RedUnix

Servidor HTTP/1.0 concurrente implementado en Rust para el curso de Principios de Sistemas Operativos.

## Tabla de Contenidos

- [Características](#características)
- [Requisitos](#requisitos)
- [Instalación](#instalación)
- [Compilación](#compilación)
- [Uso](#uso)
- [Arquitectura](#arquitectura)
- [API Reference](#api-reference)
- [Testing](#testing)
- [Configuración](#configuración)
- [Troubleshooting](#troubleshooting)

## Características

- ✅ **Servidor HTTP/1.0** completo desde cero (sin frameworks)
- ✅ **Concurrencia** con pools de workers por categoría (básico, CPU-bound, IO-bound)
- ✅ **22 comandos** implementados:
  - 12 comandos básicos (fibonacci, reverse, createfile, etc.)
  - 5 comandos CPU-intensive (isprime, factor, pi, mandelbrot, matrixmul)
  - 5 comandos IO-intensive (sortfile, wordcount, grep, compress, hashfile)
- ✅ **Sistema de Jobs asíncrono** con prioridades y timeouts
- ✅ **Métricas avanzadas** (latencias p50/p95/p99, throughput)
- ✅ **Observabilidad** con headers X-Request-Id, X-Worker-Pid, X-Worker-Thread
- ✅ **Backpressure** con respuestas 503 y Retry-After
- ✅ **Configuración flexible** via CLI y variables de entorno
- ✅ **Coverage ~90%** con 146+ tests unitarios

## Requisitos

### Software Necesario

- **Rust** 1.70+ (2021 edition)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **curl** o **Postman** para testing
- **bc** para cálculos en scripts (opcional)

### Dependencias de Rust

Las dependencias se instalan automáticamente con `cargo build`:
- `clap` - Parsing de argumentos CLI
- `flate2` - Compresión gzip
- `regex` - Expresiones regulares
- `sha2` - Hashing SHA256
- `serde` + `serde_json` - Serialización JSON

##  Instalación

```bash
# Clonar el repositorio
git clone https://github.com/tu-usuario/http_server.git
cd http_server

# Verificar instalación de Rust
rustc --version
cargo --version

# Instalar dependencias y compilar (modo debug)
cargo build

# Compilar en modo release (optimizado)
cargo build --release
```

##  Compilación

### Modo Desarrollo (debug)
```bash
cargo build
# Ejecutable en: ./target/debug/http_server
```

### Modo Producción (release)
```bash
cargo build --release
# Ejecutable en: ./target/release/http_server
```

### Ejecutar directamente sin compilar
```bash
cargo run -- --port 8080
```

## Uso

### Inicio Básico

```bash
# Con configuración por defecto (puerto 8080, 4 workers)
./target/release/http_server

# Especificar puerto
./target/release/http_server --port 3000

# Configuración personalizada
./target/release/http_server \
  --port 8080 \
  --workers-cpu 8 \
  --workers-io 6 \
  --workers-basic 4 \
  --queue-cpu 128 \
  --data-dir ./my_data
```

### Configuración con Variables de Entorno

```bash
export HTTP_PORT=9000
export WORKERS_CPU=8
export WORKERS_IO=6
./target/release/http_server
```

### Verificar que el Servidor Está Corriendo

```bash
curl http://localhost:8080/status
```

Deberías ver un JSON con información del servidor:
```json
{
  "status": "running",
  "uptime": 120,
  "pid": 12345,
  "connections_served": 42,
  "workers": { ... },
  "queue_sizes": { ... }
}
```

##  Arquitectura

### Estructura del Proyecto

```
http_server/
├── Cargo.toml              # Dependencias y configuración
├── README.md               # Este archivo
├── src/
│   ├── main.rs            # Punto de entrada
│   ├── lib.rs             # Módulo raíz
│   ├── config.rs          # Configuración y CLI parser
│   ├── http/              # Protocolo HTTP/1.0
│   │   ├── mod.rs
│   │   ├── request.rs     # Parser de requests
│   │   ├── response.rs    # Constructor de responses
│   │   └── status.rs      # Códigos de estado HTTP
│   ├── server/            # Servidor TCP
│   │   ├── mod.rs
│   │   └── tcp.rs         # Listener y manejo de conexiones
│   ├── router/            # Sistema de routing
│   │   └── mod.rs         # Mapeo path → handler
│   ├── commands/          # Implementación de comandos
│   │   ├── mod.rs
│   │   ├── basic.rs       # 12 comandos básicos
│   │   ├── cpu_bound.rs   # 5 comandos CPU-intensive
│   │   └── io_bound.rs    # 5 comandos IO-intensive
│   ├── jobs/              # Sistema asíncrono de jobs
│   │   ├── mod.rs
│   │   ├── types.rs       # JobStatus, JobPriority, JobType
│   │   ├── manager.rs     # JobManager (coordina workers)
│   │   ├── queue.rs       # Cola de prioridad thread-safe
│   │   ├── storage.rs     # Persistencia en JSON
│   │   └── handlers.rs    # Endpoints HTTP de jobs
│   └── metrics/           # Métricas de observabilidad
│       ├── mod.rs
│       └── collector.rs   # Latencias, throughput, etc.
├── data/                  # Directorio de datos (creado en runtime)
│   ├── jobs.json         # Persistencia de jobs
│   └── *.txt, *.gz       # Archivos de usuario
└── target/               # Artefactos de compilación
    ├── debug/
    └── release/
        └── http_server   # Ejecutable final
```

### Flujo de una Request

```
1. Cliente → TCP Socket (TcpListener en tcp.rs)
2. Thread dedicado lee la request
3. Parser HTTP/1.0 (request.rs) → Request struct
4. Router (router.rs) → Determina handler según path
5. Handler ejecuta comando → Response struct
6. Serialización HTTP/1.0 (response.rs)
7. Response → Cliente
8. MetricsCollector registra latencia y throughput
```

### Concurrencia y Workers

El servidor utiliza **3 pools de workers** independientes:

1. **Basic Workers** (4 por defecto)
   - Comandos rápidos y ligeros
   - fibonacci, reverse, toupper, etc.

2. **CPU-bound Workers** (4 por defecto)
   - Tareas computacionalmente intensivas
   - isprime, factor, pi, mandelbrot, matrixmul

3. **IO-bound Workers** (4 por defecto)
   - Operaciones de entrada/salida
   - sortfile, wordcount, grep, compress, hashfile

Cada pool tiene:
- ✅ Cola de prioridad thread-safe (`Arc<Mutex<VecDeque<Job>>>`)
- ✅ Workers que procesan jobs de su cola
- ✅ Backpressure: devuelve 503 si la cola está llena

##  API Reference

### Comandos Básicos

#### GET /status
Devuelve el estado del servidor.

**Response:**
```json
{
  "status": "running",
  "uptime": 3600,
  "pid": 12345,
  "connections_served": 1000,
  "workers": {
    "basic": [...],
    "cpu_bound": [...],
    "io_bound": [...]
  },
  "queue_sizes": {
    "basic": 2,
    "cpu_bound": 5,
    "io_bound": 1
  },
  "metrics": {
    "total_requests": 1000,
    "avg_latency_ms": 45,
    "p50_latency_ms": 30,
    "p95_latency_ms": 120,
    "p99_latency_ms": 250
  }
}
```

#### GET /fibonacci?num=N
Calcula el N-ésimo número de Fibonacci.

**Parameters:**
- `num` (required): Número entero >= 0

**Example:**
```bash
curl "http://localhost:8080/fibonacci?num=20"
```

**Response:**
```json
{
  "num": 20,
  "result": 6765,
  "elapsed_ms": 0
}
```

#### GET /reverse?text=STRING
Invierte una cadena de texto.

**Parameters:**
- `text` (required): Texto a invertir

**Example:**
```bash
curl "http://localhost:8080/reverse?text=Hello%20World"
```

**Response:**
```json
{
  "original": "Hello World",
  "reversed": "dlroW olleH",
  "length": 11
}
```

#### GET /toupper?text=STRING
Convierte texto a mayúsculas.

**Parameters:**
- `text` (required): Texto a convertir

**Example:**
```bash
curl "http://localhost:8080/toupper?text=hello"
```

**Response:**
```json
{
  "original": "hello",
  "uppercase": "HELLO"
}
```

#### GET /createfile?name=NAME&content=CONTENT&repeat=N
Crea un archivo con contenido repetido.

**Parameters:**
- `name` (required): Nombre del archivo
- `content` (required): Contenido a escribir
- `repeat` (optional, default=1): Veces a repetir

**Example:**
```bash
curl "http://localhost:8080/createfile?name=test.txt&content=Hello&repeat=100"
```

**Response:**
```json
{
  "file": "test.txt",
  "size": 500,
  "elapsed_ms": 5
}
```

#### GET /deletefile?name=NAME
Elimina un archivo.

**Parameters:**
- `name` (required): Nombre del archivo a eliminar

**Example:**
```bash
curl "http://localhost:8080/deletefile?name=test.txt"
```

**Response:**
```json
{
  "file": "test.txt",
  "deleted": true
}
```

### Comandos CPU-Bound

#### GET /isprime?num=N
Verifica si un número es primo.

**Parameters:**
- `num` (required): Número a verificar

**Example:**
```bash
curl "http://localhost:8080/isprime?num=15485863"
```

**Response:**
```json
{
  "num": 15485863,
  "is_prime": true,
  "elapsed_ms": 42
}
```

#### GET /factor?num=N
Factoriza un número en primos.

**Parameters:**
- `num` (required): Número a factorizar

**Example:**
```bash
curl "http://localhost:8080/factor?num=123456"
```

**Response:**
```json
{
  "num": 123456,
  "factors": [2, 2, 2, 2, 2, 2, 3, 643],
  "elapsed_ms": 15
}
```

#### GET /pi?iterations=N
Calcula Pi usando Monte Carlo.

**Parameters:**
- `iterations` (optional, default=1000000): Número de iteraciones

**Example:**
```bash
curl "http://localhost:8080/pi?iterations=10000000"
```

**Response:**
```json
{
  "iterations": 10000000,
  "pi_estimate": 3.14159265,
  "elapsed_ms": 1250
}
```

#### GET /mandelbrot?width=W&height=H&max_iter=I
Genera el conjunto de Mandelbrot.

**Parameters:**
- `width` (optional, default=800): Ancho
- `height` (optional, default=600): Alto
- `max_iter` (optional, default=100): Iteraciones máximas

**Example:**
```bash
curl "http://localhost:8080/mandelbrot?width=1920&height=1080&max_iter=1000"
```

#### GET /matrixmul?size=N
Multiplica dos matrices N×N.

**Parameters:**
- `size` (optional, default=100): Tamaño de las matrices

**Example:**
```bash
curl "http://localhost:8080/matrixmul?size=500"
```

### Comandos IO-Bound

#### GET /sortfile?name=FILE&algo=ALGO
Ordena números en un archivo.

**Parameters:**
- `name` (required): Nombre del archivo en `data/`
- `algo` (optional, default=merge): Algoritmo (`merge` o `quick`)

**Example:**
```bash
curl "http://localhost:8080/sortfile?name=large_numbers.txt&algo=merge"
```

**Response:**
```json
{
  "file": "large_numbers.txt",
  "algo": "merge",
  "sorted_file": "large_numbers.txt.sorted",
  "lines": 1000000,
  "elapsed_ms": 2500
}
```

#### GET /wordcount?name=FILE
Cuenta líneas, palabras y bytes.

**Parameters:**
- `name` (required): Nombre del archivo

**Example:**
```bash
curl "http://localhost:8080/wordcount?name=large_text.txt"
```

**Response:**
```json
{
  "file": "large_text.txt",
  "lines": 1000000,
  "words": 5000000,
  "bytes": 52428800,
  "elapsed_ms": 850
}
```

#### GET /grep?name=FILE&pattern=REGEX
Busca líneas que coincidan con una regex.

**Parameters:**
- `name` (required): Nombre del archivo
- `pattern` (required): Expresión regular

**Example:**
```bash
curl "http://localhost:8080/grep?name=large_text.txt&pattern=ERROR"
```

**Response:**
```json
{
  "file": "large_text.txt",
  "pattern": "ERROR",
  "matches": 200000,
  "sample_lines": ["[ERROR] Line 1...", "..."],
  "elapsed_ms": 650
}
```

#### GET /compress?name=FILE&codec=gzip
Comprime un archivo.

**Parameters:**
- `name` (required): Nombre del archivo
- `codec` (optional, default=gzip): Codec (`gzip`)

**Example:**
```bash
curl "http://localhost:8080/compress?name=large_text.txt&codec=gzip"
```

**Response:**
```json
{
  "file": "large_text.txt",
  "codec": "gzip",
  "output": "large_text.txt.gz",
  "original_size": 52428800,
  "compressed_size": 5242880,
  "ratio": 0.10,
  "elapsed_ms": 3200
}
```

#### GET /hashfile?name=FILE&algo=sha256
Calcula hash SHA256 de un archivo.

**Parameters:**
- `name` (required): Nombre del archivo
- `algo` (optional, default=sha256): Algoritmo (`sha256`)

**Example:**
```bash
curl "http://localhost:8080/hashfile?name=large_hash.txt&algo=sha256"
```

**Response:**
```json
{
  "file": "large_hash.txt",
  "algo": "sha256",
  "hash": "a1b2c3d4...",
  "bytes": 52428800,
  "elapsed_ms": 1100
}
```

### Sistema de Jobs

#### POST /jobs/submit
Envía un job para ejecución asíncrona.

**Body (JSON):**
```json
{
  "command": "isprime",
  "params": {"num": "982451653"},
  "priority": "HIGH"
}
```

**Response:**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "QUEUED"
}
```

#### GET /jobs/status?id=JOB_ID
Consulta el estado de un job.

**Example:**
```bash
curl "http://localhost:8080/jobs/status?id=550e8400-e29b-41d4-a716-446655440000"
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "COMPLETED",
  "progress": 100
}
```

#### GET /jobs/result?id=JOB_ID
Obtiene el resultado de un job completado.

#### DELETE /jobs/cancel?id=JOB_ID
Cancela un job en ejecución.

### Métricas

#### GET /metrics
Devuelve métricas detalladas del servidor.

**Response:**
```json
{
  "requests": {
    "total": 10000,
    "success": 9800,
    "errors": 200
  },
  "latency_ms": {
    "mean": 45,
    "stddev": 12,
    "p50": 30,
    "p95": 120,
    "p99": 250,
    "min": 5,
    "max": 1200
  },
  "throughput": {
    "requests_per_second": 150
  },
  "job_queues": {
    "basic": 2,
    "cpu_bound": 5,
    "io_bound": 1
  }
}
```

## Testing

### Ejecutar Todos los Tests

```bash
cargo test
```

### Ejecutar Tests de un Módulo Específico

```bash
cargo test commands::basic
cargo test commands::cpu_bound
cargo test commands::io_bound
cargo test jobs
```

### Test con Output Detallado

```bash
cargo test -- --nocapture
```

### Coverage con Tarpaulin

```bash
# Instalar tarpaulin
cargo install cargo-tarpaulin

# Ejecutar coverage
cargo tarpaulin --out Html --output-dir ./coverage

# Ver reporte
open coverage/index.html
```

### Tests de Integración

```bash
# Iniciar el servidor en una terminal
./target/release/http_server --port 8080

# En otra terminal, ejecutar tests manuales
curl http://localhost:8080/status
curl http://localhost:8080/fibonacci?num=30
curl http://localhost:8080/isprime?num=15485863
```

### Stress Testing

```bash
# Generar datasets grandes primero
./generate_large_datasets.sh

# Ejecutar stress test con perfil bajo
./stress_test.sh --profile low

# Perfil medio
./stress_test.sh --profile medium

# Perfil alto
./stress_test.sh --profile high
```

Los resultados se guardan en `stress_results/` con:
- Logs detallados (CSV)
- Reportes JSON con métricas
- Latencias p50/p95/p99
- Throughput

## Configuración

### Opciones CLI

```
USAGE:
    http_server [OPTIONS]

OPTIONS:
    -p, --port <PORT>                  Puerto del servidor [default: 8080]
        --host <HOST>                  Host/IP [default: 127.0.0.1]
        --data-dir <DIR>               Directorio de datos [default: ./data]
        --workers-cpu <N>              Workers CPU-bound [default: 4]
        --workers-io <N>               Workers IO-bound [default: 4]
        --workers-basic <N>            Workers básicos [default: 2]
        --queue-cpu <N>                Tamaño cola CPU [default: 64]
        --queue-io <N>                 Tamaño cola IO [default: 64]
        --queue-basic <N>              Tamaño cola básica [default: 32]
        --timeout-cpu <MS>             Timeout CPU (ms) [default: 60000]
        --timeout-io <MS>              Timeout IO (ms) [default: 60000]
        --timeout-basic <MS>           Timeout básico (ms) [default: 30000]
    -h, --help                         Muestra ayuda
    -V, --version                      Muestra versión
```

### Variables de Entorno

Todas las opciones CLI tienen una variable de entorno equivalente:

- `HTTP_PORT` → --port
- `HTTP_HOST` → --host
- `DATA_DIR` → --data-dir
- `WORKERS_CPU` → --workers-cpu
- `WORKERS_IO` → --workers-io
- `WORKERS_BASIC` → --workers-basic
- `QUEUE_CPU` → --queue-cpu
- `QUEUE_IO` → --queue-io
- `QUEUE_BASIC` → --queue-basic
- `TIMEOUT_CPU` → --timeout-cpu
- `TIMEOUT_IO` → --timeout-io
- `TIMEOUT_BASIC` → --timeout-basic

**Ejemplo:**
```bash
export HTTP_PORT=9000
export WORKERS_CPU=16
export DATA_DIR=/tmp/http_data
./target/release/http_server
```

## Troubleshooting

### El servidor no inicia

**Síntoma:** Error "Address already in use"

**Solución:** El puerto está ocupado. Cambia el puerto o libera el puerto actual:
```bash
# Ver qué proceso usa el puerto 8080
lsof -i :8080
# o
netstat -tuln | grep 8080

# Matar el proceso
kill -9 <PID>

# O usar otro puerto
./target/release/http_server --port 9000
```

### Errores de compilación

**Síntoma:** `error: could not compile ...`

**Solución:**
```bash
# Limpiar y recompilar
cargo clean
cargo build --release

# Actualizar Rust
rustup update stable
```

### Tests fallan

**Síntoma:** Tests de IO fallan

**Solución:**
```bash
# Asegurar que existe el directorio data/
mkdir -p ./data

# Ejecutar solo el test problemático
cargo test test_sortfile_handler_success -- --nocapture
```

### Requests muy lentas

**Síntoma:** Latencias altas (p95 > 500ms)

**Posibles causas:**
1. Pocos workers → Incrementar `--workers-cpu` y `--workers-io`
2. CPU saturado → Reducir carga concurrente
3. Disco lento → Usar SSD o RAM disk para `data/`

**Solución:**
```bash
# Incrementar workers
./target/release/http_server --workers-cpu 16 --workers-io 12

# Monitorear recursos
top
htop
```

### Cola llena (503 Service Unavailable)

**Síntoma:** Respuesta 503 con header `Retry-After`

**Solución:** El sistema está saturado. Opciones:
1. Incrementar tamaño de colas: `--queue-cpu 256 --queue-io 256`
2. Incrementar workers: `--workers-cpu 16`
3. Reducir carga de clientes


##  Autores

- Luis Gerardo Urbina Salazar - Pablo Mauricio Mesen Alvarado
- Profesor: Kenneth Obando Rodríguez
- Instituto Tecnológico de Costa Rica


---

**Documentación completa del proyecto disponible en el informe científico.**
