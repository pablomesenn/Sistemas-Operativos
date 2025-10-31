#!/bin/bash

# Script para probar todos los endpoints del servidor HTTP
# Autor: Test Suite para RedUnix HTTP Server
# Fecha: 2025-10-30

# NO usar set -e para que continue con todos los tests

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuración
HOST="localhost"
PORT=8080
BASE_URL="http://${HOST}:${PORT}"
LOGFILE="endpoint_tests_$(date +%Y%m%d_%H%M%S).log"

# Contadores
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Funciones auxiliares
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_test() {
    echo -e "${YELLOW}TEST:${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓ PASS:${NC} $1"
    ((PASSED_TESTS++))
}

print_failure() {
    echo -e "${RED}✗ FAIL:${NC} $1"
    ((FAILED_TESTS++))
}

test_endpoint() {
    local name=$1
    local url=$2
    local expected_status=${3:-200}
    local method=${4:-GET}
    local data=${5:-}
    
    ((TOTAL_TESTS++))
    print_test "$name"
    
    # Hacer la petición
    if [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$url" 2>&1 || echo -e "\n000")
    elif [ "$method" = "DELETE" ]; then
        response=$(curl -s -w "\n%{http_code}" -X DELETE \
            "$url" 2>&1 || echo -e "\n000")
    else
        response=$(curl -s -w "\n%{http_code}" "$url" 2>&1 || echo -e "\n000")
    fi
    
    # Extraer código HTTP (última línea)
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    
    # Logging
    echo "=== TEST: $name ===" >> "$LOGFILE"
    echo "URL: $url" >> "$LOGFILE"
    echo "Method: $method" >> "$LOGFILE"
    echo "Expected: $expected_status, Got: $http_code" >> "$LOGFILE"
    echo "Response: $body" >> "$LOGFILE"
    echo "---" >> "$LOGFILE"
    
    # Validar
    if [ "$http_code" = "$expected_status" ]; then
        print_success "$name (Status: $http_code)"
        echo "  Body preview: ${body:0:100}..."
        return 0
    else
        print_failure "$name (Expected: $expected_status, Got: $http_code)"
        if [ "$http_code" = "000" ]; then
            echo "  Error: No se pudo conectar al servidor"
        else
            echo "  Body: ${body:0:200}"
        fi
        return 1
    fi
}

# Verificar que el servidor esté corriendo
echo -e "${BLUE}Verificando conectividad con el servidor...${NC}"
if curl -s -m 2 "$BASE_URL/status" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Servidor accesible en $BASE_URL${NC}\n"
else
    echo -e "${RED}ERROR: El servidor no está corriendo en $BASE_URL${NC}"
    echo ""
    echo "Por favor inicia el servidor primero:"
    echo "  cd /mnt/project"
    echo "  cargo run --release -- --port $PORT"
    echo ""
    echo "O si ya está corriendo en otro puerto, edita este script y cambia la variable PORT"
    exit 1
fi

# Iniciar log
echo "=== ENDPOINT TESTING LOG ===" > "$LOGFILE"
echo "Date: $(date)" >> "$LOGFILE"
echo "Base URL: $BASE_URL" >> "$LOGFILE"
echo "================================" >> "$LOGFILE"

# ==========================================
# 1. ENDPOINT BÁSICO: STATUS
# ==========================================
print_header "1. ENDPOINT BÁSICO: STATUS"
test_endpoint "GET /status" "$BASE_URL/status"

# ==========================================
# 2. COMANDOS BÁSICOS (13 comandos)
# ==========================================
print_header "2. COMANDOS BÁSICOS (13 comandos)"

# 2.1 Fibonacci
test_endpoint "GET /fibonacci?num=10" "$BASE_URL/fibonacci?num=10"
test_endpoint "GET /fibonacci?num=20" "$BASE_URL/fibonacci?num=20"

# 2.2 Reverse
test_endpoint "GET /reverse?text=Hello" "$BASE_URL/reverse?text=Hello"

# 2.3 ToUpper
test_endpoint "GET /toupper?text=hello" "$BASE_URL/toupper?text=hello"

# 2.4 Timestamp
test_endpoint "GET /timestamp" "$BASE_URL/timestamp"

# 2.5 Random
test_endpoint "GET /random?min=1&max=100" "$BASE_URL/random?min=1&max=100"

# 2.6 Hash
test_endpoint "GET /hash?text=password" "$BASE_URL/hash?text=password"

# 2.7 CreateFile
test_endpoint "GET /createfile?name=test1.txt&content=TestContent&repeat=10" \
    "$BASE_URL/createfile?name=test1.txt&content=TestContent&repeat=10"

# 2.8 DeleteFile
test_endpoint "GET /deletefile?name=test1.txt" "$BASE_URL/deletefile?name=test1.txt"

# 2.9 Simulate
test_endpoint "GET /simulate?seconds=5" "$BASE_URL/simulate?seconds=5"

# 2.10 Sleep
test_endpoint "GET /sleep?seconds=1" "$BASE_URL/sleep?seconds=1"

# 2.11 LoadTest
test_endpoint "GET /loadtest?count=5" "$BASE_URL/loadtest?count=5"

# 2.12 Help
test_endpoint "GET /help" "$BASE_URL/help"

# ==========================================
# 3. COMANDOS CPU-BOUND (5 comandos)
# ==========================================
print_header "3. COMANDOS CPU-BOUND (5 comandos)"

# 3.1 IsPrime
test_endpoint "GET /isprime?n=17" "$BASE_URL/isprime?n=17"
test_endpoint "GET /isprime?n=15485863" "$BASE_URL/isprime?n=15485863"

# 3.2 Factor
test_endpoint "GET /factor?n=60" "$BASE_URL/factor?n=60"

# 3.3 PI
test_endpoint "GET /pi?digits=100" "$BASE_URL/pi?digits=100"

# 3.4 Mandelbrot
test_endpoint "GET /mandelbrot?width=400&height=300&max_iter=50" \
    "$BASE_URL/mandelbrot?width=400&height=300&max_iter=50"

# 3.5 MatrixMul
test_endpoint "GET /matrixmul?size=50" "$BASE_URL/matrixmul?size=50"

# ==========================================
# 4. COMANDOS IO-BOUND (5 comandos)
# ==========================================
print_header "4. COMANDOS IO-BOUND (5 comandos)"

# Crear archivo de prueba para IO
echo "Preparando archivo para tests IO..."
curl -s "$BASE_URL/createfile?name=test_io.txt&content=Line%20with%20ERROR%0A&repeat=1000" > /dev/null 2>&1
sleep 1

# 4.1 WordCount
test_endpoint "GET /wordcount?name=test_io.txt" "$BASE_URL/wordcount?name=test_io.txt"

# 4.2 Grep
test_endpoint "GET /grep?name=test_io.txt&pattern=ERROR" \
    "$BASE_URL/grep?name=test_io.txt&pattern=ERROR"

# 4.3 HashFile
test_endpoint "GET /hashfile?name=test_io.txt&algo=sha256" \
    "$BASE_URL/hashfile?name=test_io.txt&algo=sha256"

# 4.4 SortFile (puede tardar un poco)
test_endpoint "GET /sortfile?name=test_io.txt" "$BASE_URL/sortfile?name=test_io.txt"

# 4.5 Compress
test_endpoint "GET /compress?name=test_io.txt&codec=gzip" \
    "$BASE_URL/compress?name=test_io.txt&codec=gzip"

# Limpieza
echo "Limpiando archivos de prueba..."
curl -s "$BASE_URL/deletefile?name=test_io.txt" > /dev/null 2>&1
curl -s "$BASE_URL/deletefile?name=test_io.txt.gz" > /dev/null 2>&1
curl -s "$BASE_URL/deletefile?name=test_io.txt_sorted.txt" > /dev/null 2>&1

# ==========================================
# 5. SISTEMA DE JOBS
# ==========================================
print_header "5. SISTEMA DE JOBS ASÍNCRONO"

# 5.1 Submit Job usando POST - CORREGIDO
echo "Enviando job asíncrono con POST..."
((TOTAL_TESTS++))
JOB_SUBMIT_RESPONSE=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"command":"isprime","params":{"n":"982451653"},"priority":"high"}' \
    "$BASE_URL/jobs/submit")

echo "Response: $JOB_SUBMIT_RESPONSE"

# Extraer job_id - CORREGIDO para manejar guiones
JOB_ID=""
if command -v jq &> /dev/null; then
    JOB_ID=$(echo "$JOB_SUBMIT_RESPONSE" | jq -r '.job_id' 2>/dev/null)
fi

# Si jq no está disponible, usar grep/sed
if [ -z "$JOB_ID" ] || [ "$JOB_ID" = "null" ]; then
    JOB_ID=$(echo "$JOB_SUBMIT_RESPONSE" | grep -o '"job_id"[[:space:]]*:[[:space:]]*"[^"]*"' | sed 's/.*"\([^"]*\)".*/\1/')
fi

if [ -n "$JOB_ID" ] && [ "$JOB_ID" != "null" ] && [ "$JOB_ID" != "" ]; then
    print_success "POST /jobs/submit (Job ID: ${JOB_ID:0:16}...)"
    
    # Esperar un poco
    sleep 1
    
    # 5.2 Job Status
    test_endpoint "GET /jobs/status?id=$JOB_ID" "$BASE_URL/jobs/status?id=$JOB_ID"
    
    # Esperar para que termine
    sleep 2
    
    # 5.3 Job Result
    test_endpoint "GET /jobs/result?id=$JOB_ID" "$BASE_URL/jobs/result?id=$JOB_ID"
else
    print_failure "POST /jobs/submit - No se pudo extraer job_id"
    echo "  Response: $JOB_SUBMIT_RESPONSE"
fi

# ==========================================
# 6. MÉTRICAS
# ==========================================
print_header "6. MÉTRICAS Y OBSERVABILIDAD"

test_endpoint "GET /metrics" "$BASE_URL/metrics"

# ==========================================
# 7. PRUEBAS DE ERROR HANDLING
# ==========================================
print_header "7. ERROR HANDLING (debe fallar con 400/404)"

# Parámetros inválidos (esperamos 400)
test_endpoint "GET /fibonacci sin parámetros (esperado 400)" "$BASE_URL/fibonacci" 400
test_endpoint "GET /isprime con texto (esperado 400)" "$BASE_URL/isprime?n=abc" 400

# Archivo inexistente (esperamos 404)
test_endpoint "GET /deletefile archivo inexistente (esperado 404)" \
    "$BASE_URL/deletefile?name=nonexistent_file_12345.txt" 404

# Endpoint inexistente (esperamos 404)
test_endpoint "GET /nonexistent (esperado 404)" "$BASE_URL/nonexistent" 404

# ==========================================
# 8. RESUMEN
# ==========================================
print_header "RESUMEN DE PRUEBAS"

echo -e "Total de pruebas: ${BLUE}$TOTAL_TESTS${NC}"
echo -e "Pruebas exitosas: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Pruebas fallidas: ${RED}$FAILED_TESTS${NC}"

if [ $TOTAL_TESTS -gt 0 ]; then
    PASS_RATE=$(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")
    echo -e "Tasa de éxito: ${GREEN}${PASS_RATE}%${NC}"
else
    echo -e "Tasa de éxito: ${RED}0.00%${NC}"
fi

echo -e "\nLog detallado guardado en: ${YELLOW}$LOGFILE${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "\n${GREEN}🎉 TODAS LAS PRUEBAS PASARON 🎉${NC}\n"
    exit 0
else
    echo -e "\n${RED}⚠️  $FAILED_TESTS PRUEBAS FALLARON ⚠️${NC}"
    echo -e "Revisa el log para más detalles: ${YELLOW}$LOGFILE${NC}\n"
    exit 1
fi