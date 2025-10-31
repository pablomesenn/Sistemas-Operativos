# Manual de Usuario - Servidor HTTP RedUnix

**Versión:** 0.1.0  
**Fecha:** Octubre 2025  
**Autor:** RedUnix S.A.

---

## Tabla de Contenidos

1. [Introducción](#introducción)
2. [Instalación y Configuración](#instalación-y-configuración)
3. [Iniciar el Servidor](#iniciar-el-servidor)
4. [Endpoints Disponibles](#endpoints-disponibles)
   - [Comandos Básicos](#comandos-básicos)
   - [Comandos CPU-Bound](#comandos-cpu-bound)
   - [Comandos IO-Bound](#comandos-io-bound)
   - [Sistema de Jobs](#sistema-de-jobs)
   - [Métricas y Observabilidad](#métricas-y-observabilidad)
5. [Códigos de Respuesta HTTP](#códigos-de-respuesta-http)
6. [Ejemplos de Uso Completos](#ejemplos-de-uso-completos)
7. [Troubleshooting](#troubleshooting)
8. [Preguntas Frecuentes](#preguntas-frecuentes)

---

## Introducción

El Servidor HTTP RedUnix es un servidor HTTP/1.0 diseñado para procesar múltiples tipos de tareas de forma concurrente. Soporta operaciones básicas, procesamiento intensivo de CPU, operaciones de E/S, y un sistema de jobs para tareas de larga duración.

### Características principales:

- ✅ Múltiples clientes concurrentes
- ✅ Workers especializados por tipo de comando
- ✅ Sistema de jobs asíncrono para tareas largas
- ✅ Procesamiento CPU-bound (cálculos matemáticos)
- ✅ Procesamiento IO-bound (operaciones con archivos)
- ✅ Métricas de desempeño en tiempo real
- ✅ Manejo robusto de errores

### Casos de uso:

- Procesamiento de cálculos matemáticos intensivos
- Manipulación de archivos grandes
- Ejecución de tareas en segundo plano
- Monitoreo de desempeño del servidor
- Testing de carga y concurrencia

---

## Instalación y Configuración

### Requisitos previos:

- **Rust 1.70 o superior** (si compilas desde fuente)
- **Sistema operativo:** Linux, macOS o Windows con WSL
- **Herramientas:** `curl` o Postman para hacer peticiones
- **Memoria RAM:** Mínimo 512MB disponible
- **Espacio en disco:** 100MB para el binario y datos

### Compilación:

```bash
# Clonar el repositorio
git clone <url-del-repositorio>
cd http_server

# Compilar en modo release
cargo build --release

# El ejecutable estará en:
# target/release/http_server
```

### Configuración:

El servidor acepta los siguientes parámetros de línea de comandos:

| Parámetro | Descripción | Valor por defecto |
|-----------|-------------|-------------------|
| `--port <N>` | Puerto de escucha | 8080 |
| `--workers <N>` | Workers por comando | 4 |
| `--queue-depth <N>` | Tamaño máximo de cola | 100 |
| `--timeout-cpu <MS>` | Timeout tareas CPU (ms) | 60000 |
| `--timeout-io <MS>` | Timeout tareas IO (ms) | 120000 |

**Ejemplo de configuración avanzada:**
```bash
export HTTP_SERVER_PORT=3000
export HTTP_SERVER_WORKERS=8
export HTTP_SERVER_QUEUE_DEPTH=200
```

---

## Iniciar el Servidor

### Inicio básico:

```bash
# Desde el directorio del proyecto
./target/release/http_server --port 8080
```

**Salida esperada:**
```
[INFO] Starting RedUnix HTTP Server...
[INFO] Version: 0.1.0
[INFO] Listening on 0.0.0.0:8080
[INFO] Workers per command: 4
[INFO] Queue depth: 100
[INFO] Ready to accept connections
```

### Con configuración personalizada:

```bash
./target/release/http_server \
  --port 8080 \
  --workers 8 \
  --queue-depth 200 \
  --timeout-cpu 90000 \
  --timeout-io 180000
```

### Verificar que el servidor está corriendo:

```bash
curl http://localhost:8080/status
```

**Respuesta esperada:**
```json
{
  "status": "running",
  "version": "0.1.0",
  "server": "RedUnix HTTP/1.0",
  "uptime_seconds": 42
}
```

### Detener el servidor:

```bash
# Ctrl+C en la terminal donde corre el servidor
# O enviar señal SIGTERM:
kill <PID>
```

---

## Endpoints Disponibles

### Comandos Básicos

#### 1. `/status` - Estado del servidor

Obtiene información sobre el estado actual del servidor.

**Método:** `GET`  
**Parámetros:** Ninguno

**Ejemplo:**
```bash
curl http://localhost:8080/status
```

**Respuesta:**
```json
{
  "status": "running",
  "version": "0.1.0",
  "server": "RedUnix HTTP/1.0",
  "uptime_seconds": 3600,
  "connections_handled": 1523
}
```

---

#### 2. `/fibonacci?num=N` - Cálculo de Fibonacci

Calcula el N-ésimo número de Fibonacci.

**Método:** `GET`  
**Parámetros:**
- `num` (requerido): Número entero positivo (0-90)

**Ejemplo:**
```bash
curl "http://localhost:8080/fibonacci?num=10"
```

**Respuesta:**
```json
{
  "num": 10,
  "result": 55
}
```

**Errores comunes:**
- `num` fuera de rango → 400 Bad Request
- `num` no numérico → 400 Bad Request

---

#### 3. `/reverse?text=STRING` - Invertir texto

Invierte una cadena de texto.

**Método:** `GET`  
**Parámetros:**
- `text` (requerido): Texto a invertir

**Ejemplo:**
```bash
curl "http://localhost:8080/reverse?text=Hello"
```

**Respuesta:**
```json
{
  "original": "Hello",
  "reversed": "olleH"
}
```

**Nota:** Los espacios deben codificarse como `%20` o usar comillas.

---

#### 4. `/toupper?text=STRING` - Convertir a mayúsculas

Convierte texto a mayúsculas.

**Método:** `GET`  
**Parámetros:**
- `text` (requerido): Texto a convertir

**Ejemplo:**
```bash
curl "http://localhost:8080/toupper?text=hello"
```

**Respuesta:**
```json
{
  "original": "hello",
  "upper": "HELLO"
}
```

---

#### 5. `/timestamp` - Timestamp actual

Obtiene el timestamp Unix actual en segundos.

**Método:** `GET`  
**Parámetros:** Ninguno

**Ejemplo:**
```bash
curl http://localhost:8080/timestamp
```

**Respuesta:**
```json
{
  "timestamp": 1699123456
}
```

---

#### 6. `/random?min=A&max=B&count=N` - Números aleatorios

Genera números aleatorios en un rango especificado.

**Método:** `GET`  
**Parámetros:**
- `min` (opcional): Valor mínimo (default: 0)
- `max` (opcional): Valor máximo (default: 100)
- `count` (opcional): Cantidad de números (default: 1)

**Ejemplo:**
```bash
curl "http://localhost:8080/random?min=1&max=100&count=5"
```

**Respuesta:**
```json
{
  "count": 5,
  "min": 1,
  "max": 100,
  "values": [42, 17, 93, 8, 65]
}
```

---

#### 7. `/hash?text=STRING` - Hash simple

Calcula un hash simple del texto proporcionado.

**Método:** `GET`  
**Parámetros:**
- `text` (requerido): Texto a hashear

**Ejemplo:**
```bash
curl "http://localhost:8080/hash?text=password"
```

**Respuesta:**
```json
{
  "text": "password",
  "hash": "aa24901d4ff1f696",
  "algorithm": "simple-hash"
}
```

---

#### 8. `/createfile?name=FILE&content=TEXT&repeat=N` - Crear archivo

Crea un archivo con contenido repetido N veces.

**Método:** `GET`  
**Parámetros:**
- `name` (requerido): Nombre del archivo
- `content` (requerido): Contenido a escribir
- `repeat` (requerido): Número de repeticiones (1-10000)

**Ejemplo:**
```bash
curl "http://localhost:8080/createfile?name=test.txt&content=Hello&repeat=10"
```

**Respuesta:**
```json
{
  "filename": "test.txt",
  "size": 50,
  "repeat": 10
}
```

**Nota:** El archivo se guarda en el directorio de trabajo del servidor.

---

#### 9. `/deletefile?name=FILE` - Eliminar archivo

Elimina un archivo del sistema.

**Método:** `GET`  
**Parámetros:**
- `name` (requerido): Nombre del archivo

**Ejemplo:**
```bash
curl "http://localhost:8080/deletefile?name=test.txt"
```

**Respuesta exitosa:**
```json
{
  "filename": "test.txt",
  "deleted": true
}
```

**Respuesta error (404):**
```json
{
  "error": "File not found: test.txt"
}
```

---

#### 10. `/simulate?seconds=S` - Simular trabajo

Simula trabajo computacional por S segundos.

**Método:** `GET`  
**Parámetros:**
- `seconds` (requerido): Segundos de simulación (1-30)

**Ejemplo:**
```bash
curl "http://localhost:8080/simulate?seconds=5"
```

**Respuesta:**
```json
{
  "task": "simulation",
  "seconds": 5,
  "elapsed": 5.003,
  "iterations": 2445215000
}
```

**Nota:** Útil para testing de concurrencia.

---

#### 11. `/sleep?seconds=S` - Dormir

Hace que el servidor duerma por S segundos.

**Método:** `GET`  
**Parámetros:**
- `seconds` (requerido): Segundos de espera (1-10)

**Ejemplo:**
```bash
curl "http://localhost:8080/sleep?seconds=2"
```

**Respuesta:**
```json
{
  "slept": 2
}
```

---

#### 12. `/loadtest?count=N` - Test de carga

Ejecuta múltiples tareas concurrentes para pruebas de carga.

**Método:** `GET`  
**Parámetros:**
- `count` (opcional): Número de tareas (default: 10)

**Ejemplo:**
```bash
curl "http://localhost:8080/loadtest?count=5"
```

**Respuesta:**
```json
{
  "tasks": 10,
  "sleep_ms": 10,
  "total_time_ms": 101
}
```

---

#### 13. `/help` - Ayuda

Lista todos los comandos disponibles con sus descripciones.

**Método:** `GET`  
**Parámetros:** Ninguno

**Ejemplo:**
```bash
curl http://localhost:8080/help
```

**Respuesta:**
```json
{
  "commands": [
    {
      "path": "/status",
      "description": "Server status and metrics",
      "parameters": []
    },
    {
      "path": "/fibonacci",
      "description": "Calculate Fibonacci number",
      "parameters": ["num"]
    }
  ]
}
```

---

### Comandos CPU-Bound

Estos comandos realizan procesamiento intensivo de CPU.

#### 1. `/isprime?n=NUM` - Test de primalidad

Verifica si un número es primo usando el algoritmo Miller-Rabin.

**Método:** `GET`  
**Parámetros:**
- `n` (requerido): Número entero positivo a verificar

**Ejemplo:**
```bash
curl "http://localhost:8080/isprime?n=17"
```

**Respuesta:**
```json
{
  "n": 17,
  "is_prime": true,
  "method": "miller-rabin",
  "elapsed_ms": 0
}
```

**Casos especiales:**
- n = 1 → `is_prime: false`
- n = 2 → `is_prime: true` (único primo par)
- Números grandes pueden tardar varios segundos

---

#### 2. `/factor?n=NUM` - Factorización

Factoriza un número en sus factores primos.

**Método:** `GET`  
**Parámetros:**
- `n` (requerido): Número entero positivo a factorizar

**Ejemplo:**
```bash
curl "http://localhost:8080/factor?n=60"
```

**Respuesta:**
```json
{
  "n": 60,
  "factors": [
    [2, 2],
    [3, 1],
    [5, 1]
  ],
  "elapsed_ms": 0
}
```

**Formato:** `[[primo1, exponente1], [primo2, exponente2], ...]`  
**Interpretación:** 60 = 2² × 3¹ × 5¹

---

#### 3. `/pi?digits=D` - Cálculo de Pi

Calcula π con la precisión especificada.

**Método:** `GET`  
**Parámetros:**
- `digits` (requerido): Número de dígitos de precisión

**Ejemplo:**
```bash
curl "http://localhost:8080/pi?digits=100"
```

**Respuesta:**
```json
{
  "digits": 100,
  "value": "3.14159265358979323846264338327950288419716939937510...",
  "elapsed_ms": 15
}
```

**Advertencia:** Valores altos (>1000) pueden consumir mucha CPU.

---

#### 4. `/mandelbrot?width=W&height=H&max_iter=I` - Conjunto de Mandelbrot

Genera datos del conjunto de Mandelbrot.

**Método:** `GET`  
**Parámetros:**
- `width` (requerido): Ancho de la imagen en píxeles
- `height` (requerido): Alto de la imagen en píxeles
- `max_iter` (requerido): Iteraciones máximas por punto

**Ejemplo:**
```bash
curl "http://localhost:8080/mandelbrot?width=400&height=300&max_iter=50"
```

**Respuesta:**
```json
{
  "width": 400,
  "height": 300,
  "max_iter": 50,
  "sample_data": [
    [1, 1, 1, 2, 3, 5, 8, 13, ...],
    [1, 1, 2, 3, 5, 8, 13, 21, ...],
    ...
  ],
  "elapsed_ms": 234
}
```

**Nota:** `sample_data` muestra las primeras 10 filas.

---

#### 5. `/matrixmul?size=N&seed=S` - Multiplicación de matrices

Multiplica dos matrices N×N aleatorias.

**Método:** `GET`  
**Parámetros:**
- `size` (requerido): Tamaño de la matriz (N×N)
- `seed` (opcional): Semilla para generación aleatoria (default: 42)

**Ejemplo:**
```bash
curl "http://localhost:8080/matrixmul?size=50&seed=123"
```

**Respuesta:**
```json
{
  "size": 50,
  "seed": 123,
  "result_hash": "74958089ab9f3ed3",
  "elapsed_ms": 45
}
```

**Nota:** El hash permite verificar la correctitud del resultado.

---

### Comandos IO-Bound

Estos comandos realizan operaciones intensivas de entrada/salida.

**⚠️ Prerequisito:** Los archivos deben existir. Créalos primero con `/createfile`.

#### 1. `/wordcount?name=FILE` - Contar palabras

Cuenta líneas, palabras y bytes de un archivo (equivalente a `wc`).

**Método:** `GET`  
**Parámetros:**
- `name` (requerido): Nombre del archivo

**Ejemplo:**
```bash
# Primero crear el archivo
curl "http://localhost:8080/createfile?name=test.txt&content=Hello%20World&repeat=100"

# Luego contar
curl "http://localhost:8080/wordcount?name=test.txt"
```

**Respuesta:**
```json
{
  "file": "test.txt",
  "lines": 100,
  "words": 500,
  "bytes": 2500,
  "elapsed_ms": 3
}
```

---

#### 2. `/grep?name=FILE&pattern=PATTERN` - Buscar patrón

Busca un patrón en un archivo y retorna coincidencias.

**Método:** `GET`  
**Parámetros:**
- `name` (requerido): Nombre del archivo
- `pattern` (requerido): Patrón de búsqueda (regex simple)

**Ejemplo:**
```bash
curl "http://localhost:8080/grep?name=test.txt&pattern=ERROR"
```

**Respuesta:**
```json
{
  "file": "test.txt",
  "pattern": "ERROR",
  "matches": 5,
  "sample_lines": [
    "Line 10: ERROR occurred",
    "Line 25: ERROR in processing",
    "..."
  ],
  "elapsed_ms": 8
}
```

**Nota:** Retorna máximo 10 líneas de muestra.

---

#### 3. `/hashfile?name=FILE&algo=ALGO` - Hash de archivo

Calcula el hash criptográfico de un archivo.

**Método:** `GET`  
**Parámetros:**
- `name` (requerido): Nombre del archivo
- `algo` (requerido): Algoritmo (sha256)

**Ejemplo:**
```bash
curl "http://localhost:8080/hashfile?name=test.txt&algo=sha256"
```

**Respuesta:**
```json
{
  "file": "test.txt",
  "algo": "sha256",
  "hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "elapsed_ms": 12
}
```

---

#### 4. `/sortfile?name=FILE&algo=ALGO` - Ordenar archivo

Ordena las líneas de un archivo (debe contener números enteros).

**Método:** `GET`  
**Parámetros:**
- `name` (requerido): Nombre del archivo
- `algo` (opcional): Algoritmo (merge, quick) (default: merge)

**Ejemplo:**
```bash
# Crear archivo con números
curl "http://localhost:8080/createfile?name=data.txt&content=5%0A3%0A8%0A1%0A&repeat=100"

# Ordenar
curl "http://localhost:8080/sortfile?name=data.txt&algo=merge"
```

**Respuesta:**
```json
{
  "file": "data.txt",
  "algo": "merge",
  "sorted_file": "data.txt.sorted",
  "lines": 400,
  "elapsed_ms": 234
}
```

**Nota:** El archivo ordenado se guarda con extensión `.sorted`.

---

#### 5. `/compress?name=FILE&codec=CODEC` - Comprimir archivo

Comprime un archivo usando el códec especificado.

**Método:** `GET`  
**Parámetros:**
- `name` (requerido): Nombre del archivo
- `codec` (requerido): Códec de compresión (gzip, xz)

**Ejemplo:**
```bash
# ⚠️ Importante: Usar comillas simples para evitar problemas con &
curl 'http://localhost:8080/compress?name=test.txt&codec=gzip'
```

**Respuesta:**
```json
{
  "file": "test.txt",
  "codec": "gzip",
  "output": "test.txt.gz",
  "original_size": 2500,
  "compressed_size": 856,
  "ratio": 0.342,
  "elapsed_ms": 45
}
```

---

### Sistema de Jobs

Para tareas que pueden tardar mucho tiempo, usa el sistema de jobs asíncrono.

#### Flujo de trabajo:

1. **Submit** → Encola el trabajo
2. **Status** → Consulta el progreso
3. **Result** → Obtiene el resultado final
4. **Cancel** → Cancela si es necesario

---

#### 1. `/jobs/submit` - Enviar job

Encola un trabajo para ejecución asíncrona.

**Método:** `POST` o `GET`

**Opción 1: POST con JSON (recomendado)**

```bash
curl -X POST http://localhost:8080/jobs/submit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "isprime",
    "params": {"n": "982451653"},
    "priority": "high"
  }'
```

**Opción 2: GET con query parameters**

```bash
curl "http://localhost:8080/jobs/submit?task=isprime&n=982451653&prio=high"
```

**Respuesta:**
```json
{
  "job_id": "job-86f5fb7728eb8bb8",
  "status": "queued"
}
```

**Prioridades disponibles:**
- `low` - Baja prioridad
- `normal` - Prioridad normal (default)
- `high` - Alta prioridad (se procesa primero)

---

#### 2. `/jobs/status?id=JOBID` - Estado del job

Consulta el estado actual de un job.

**Método:** `GET`  
**Parámetros:**
- `id` (requerido): ID del job

**Ejemplo:**
```bash
curl "http://localhost:8080/jobs/status?id=job-86f5fb7728eb8bb8"
```

**Respuesta (en progreso):**
```json
{
  "status": "running",
  "progress": 45,
  "eta_ms": 5000
}
```

**Estados posibles:**
- `queued` - En cola esperando
- `running` - Ejecutándose actualmente
- `done` - Completado exitosamente
- `error` - Error durante ejecución
- `timeout` - Excedió el timeout
- `canceled` - Cancelado por el usuario

---

#### 3. `/jobs/result?id=JOBID` - Resultado del job

Obtiene el resultado de un job completado.

**Método:** `GET`  
**Parámetros:**
- `id` (requerido): ID del job

**Ejemplo:**
```bash
curl "http://localhost:8080/jobs/result?id=job-86f5fb7728eb8bb8"
```

**Respuesta (exitosa):**
```json
{
  "n": 982451653,
  "is_prime": true,
  "method": "miller-rabin",
  "elapsed_ms": 4523
}
```

**Respuesta (error):**
```json
{
  "error": "Timeout exceeded after 60000ms"
}
```

**Nota:** Solo disponible cuando `status = done`.

---

#### 4. `/jobs/cancel?id=JOBID` - Cancelar job

Intenta cancelar un job en ejecución.

**Método:** `GET`  
**Parámetros:**
- `id` (requerido): ID del job

**Ejemplo:**
```bash
curl "http://localhost:8080/jobs/cancel?id=job-86f5fb7728eb8bb8"
```

**Respuesta (exitosa):**
```json
{
  "status": "canceled"
}
```

**Respuesta (no cancelable):**
```json
{
  "error": "Job already finished and cannot be canceled"
}
```

**Nota:** Jobs en estado `done` o `error` no pueden cancelarse.

---

### Métricas y Observabilidad

#### `/metrics` - Métricas del servidor

Obtiene estadísticas de desempeño del servidor.

**Método:** `GET`  
**Parámetros:** Ninguno

**Ejemplo:**
```bash
curl http://localhost:8080/metrics
```

**Respuesta:**
```json
{
  "server": {
    "uptime_seconds": 3600,
    "connections_handled": 15234
  },
  "queues": {
    "isprime": 3,
    "factor": 0,
    "matrixmul": 5
  },
  "workers": {
    "isprime": {
      "total": 4,
      "busy": 2,
      "idle": 2
    }
  },
  "latency_ms": {
    "isprime": {
      "p50": 123,
      "p95": 456,
      "p99": 789,
      "avg": 234,
      "stddev": 89
    }
  }
}
```

**Métricas disponibles:**
- **Colas:** Tamaño actual por tipo de comando
- **Workers:** Total, ocupados, ociosos
- **Latencias:** p50, p95, p99, promedio, desviación estándar

---

## Códigos de Respuesta HTTP

El servidor utiliza los siguientes códigos de respuesta:

| Código | Significado | Cuándo ocurre |
|--------|-------------|---------------|
| 200 | OK | Operación exitosa |
| 400 | Bad Request | Parámetros inválidos o faltantes |
| 404 | Not Found | Ruta o archivo no encontrado |
| 409 | Conflict | Job no disponible aún |
| 429 | Too Many Requests | Rate limiting activado |
| 500 | Internal Server Error | Error interno del servidor |
| 503 | Service Unavailable | Cola llena, reintentar más tarde |

**Formato de error:**
```json
{
  "error": "Descripción del error"
}
```

**Códigos 503 incluyen header:**
```
Retry-After: 5
```

---

## Ejemplos de Uso Completos

### Ejemplo 1: Verificar si un número es primo

```bash
# Número pequeño
curl "http://localhost:8080/isprime?n=17"
# Respuesta inmediata: {"n": 17, "is_prime": true, ...}

# Número grande (usar jobs)
curl -X POST http://localhost:8080/jobs/submit \
  -H "Content-Type: application/json" \
  -d '{"command":"isprime","params":{"n":"982451653"},"priority":"high"}'
  
# Respuesta: {"job_id": "job-abc123", "status": "queued"}

# Consultar estado
curl "http://localhost:8080/jobs/status?id=job-abc123"

# Obtener resultado cuando done
curl "http://localhost:8080/jobs/result?id=job-abc123"
```

### Ejemplo 2: Procesar un archivo grande

```bash
# 1. Crear archivo de 50MB (aproximadamente)
curl "http://localhost:8080/createfile?name=bigfile.txt&content=TestData%0A&repeat=5000000"

# 2. Contar palabras
curl "http://localhost:8080/wordcount?name=bigfile.txt"

# 3. Comprimir
curl 'http://localhost:8080/compress?name=bigfile.txt&codec=gzip'

# 4. Calcular hash del comprimido
curl "http://localhost:8080/hashfile?name=bigfile.txt.gz&algo=sha256"

# 5. Limpiar
curl "http://localhost:8080/deletefile?name=bigfile.txt"
curl "http://localhost:8080/deletefile?name=bigfile.txt.gz"
```

### Ejemplo 3: Test de carga con jobs

```bash
# Script para enviar múltiples jobs
for i in {1..10}; do
  curl -X POST http://localhost:8080/jobs/submit \
    -H "Content-Type: application/json" \
    -d "{\"command\":\"pi\",\"params\":{\"digits\":\"$((i * 100))\"},\"priority\":\"normal\"}" &
done

# Esperar a que terminen
wait

# Ver métricas
curl http://localhost:8080/metrics
```

---

## Troubleshooting

### Problema: "Connection refused"

**Causa:** El servidor no está corriendo.

**Solución:**
```bash
# Verificar si el proceso está activo
ps aux | grep http_server

# Iniciar el servidor
./target/release/http_server --port 8080
```

---

### Problema: "File not found"

**Causa:** El archivo no existe en el directorio de trabajo.

**Solución:**
```bash
# Crear el archivo primero
curl "http://localhost:8080/createfile?name=test.txt&content=Hello&repeat=10"

# Luego usarlo
curl "http://localhost:8080/wordcount?name=test.txt"
```

---

### Problema: "Missing required parameter"

**Causa:** Falta un parámetro obligatorio.

**Solución:**
```bash
# MAL
curl "http://localhost:8080/fibonacci"

# BIEN
curl "http://localhost:8080/fibonacci?num=10"
```

---

### Problema: Job se queda en "queued"

**Causa:** Todos los workers están ocupados.

**Solución:**
```bash
# Ver estado de workers
curl http://localhost:8080/metrics

# Esperar o aumentar número de workers al reiniciar
./target/release/http_server --workers 8
```

---

### Problema: "Parameter must be between X and Y"

**Causa:** Valor fuera del rango permitido.

**Solución:**
```bash
# MAL
curl "http://localhost:8080/simulate?seconds=100"  # Máximo es 30

# BIEN
curl "http://localhost:8080/simulate?seconds=5"
```

---

### Problema: El símbolo & causa problemas en bash

**Causa:** Bash interpreta `&` como "ejecutar en background".

**Solución:**
```bash
# MAL
curl http://localhost:8080/compress?name=test.txt&codec=gzip

# BIEN (comillas simples)
curl 'http://localhost:8080/compress?name=test.txt&codec=gzip'

# BIEN (escape)
curl "http://localhost:8080/compress?name=test.txt\&codec=gzip"
```

---

## Preguntas Frecuentes

### ¿Cuál es la diferencia entre ejecución directa y jobs?

**Ejecución directa:** Para tareas rápidas (<5 segundos). Respuesta inmediata.

**Jobs:** Para tareas largas que pueden tardar minutos. Permite consultar progreso.

---

### ¿Cuántos workers debo configurar?

**Recomendación:**
- CPU-bound: Número de núcleos (ej: 4-8)
- IO-bound: 2-3× número de núcleos (ej: 16-24)

```bash
./target/release/http_server --workers 8
```

---

### ¿Cómo manejo archivos grandes (>100MB)?

Usa el sistema de jobs para evitar timeouts:

```bash
# Enviar como job
curl -X POST http://localhost:8080/jobs/submit \
  -H "Content-Type: application/json" \
  -d '{"command":"sortfile","params":{"name":"huge.txt","algo":"merge"}}'
```

---

### ¿Puedo usar HTTPS?

No, este servidor implementa HTTP/1.0 sin cifrado. Para producción, usa un reverse proxy como nginx con SSL/TLS.

---

### ¿Cómo detengo el servidor gracefully?

```bash
# Enviar SIGTERM (Ctrl+C)
# El servidor terminará las tareas actuales antes de cerrar

# O con kill
kill -TERM <PID>
```

---

### ¿Los archivos persisten entre reinicios?

Sí, los archivos se guardan en disco. Los metadatos de jobs son efímeros.

---

### ¿Puedo usar POST para todos los endpoints?

No, solo `/jobs/submit` acepta POST. Los demás usan GET.

---

### ¿Hay límite de tamaño para archivos?

El límite depende del disco disponible. Probado con archivos de hasta 1GB.

---

### ¿Cómo monitoreo el desempeño?

Usa `/metrics` regularmente y analiza las latencias p95/p99:

```bash
# Cada 10 segundos
watch -n 10 'curl -s http://localhost:8080/metrics | jq'
```

---

## Contacto y Soporte

**Desarrollado por:** RedUnix S.A.  
**Versión:** 0.1.0  
**Licencia:** MIT  

**Repositorio:** <url-del-repositorio>  
**Issues:** <url-del-repositorio>/issues  
**Documentación técnica:** Ver README.md  

---

**Última actualización:** Octubre 2025