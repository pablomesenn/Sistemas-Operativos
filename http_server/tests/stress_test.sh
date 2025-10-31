#!/bin/bash
# Script de stress testing para el servidor HTTP
# Implementa 3 perfiles de carga: bajo, medio, alto
# Requerimiento: PDF página 5 - Pruebas de desempeño

set -e

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuración
SERVER_URL="${SERVER_URL:-http://localhost:8080}"
OUTPUT_DIR="./stress_results"
mkdir -p "$OUTPUT_DIR"

# Función para mostrar ayuda
show_help() {
    cat << EOF
Uso: $0 [OPCIONES]

Script de stress testing para el servidor HTTP con 3 perfiles de carga.

OPCIONES:
    --profile PROFILE    Perfil de carga: low, medium, high (default: low)
    --server URL         URL del servidor (default: http://localhost:8080)
    --duration SECONDS   Duración en segundos (default: según perfil)
    --help              Mostrar esta ayuda

PERFILES:
    low     - 10 clientes concurrentes, 100 requests totales
    medium  - 50 clientes concurrentes, 500 requests totales
    high    - 100 clientes concurrentes, 1000 requests totales

EJEMPLOS:
    $0 --profile low
    $0 --profile medium --server http://192.168.1.100:8080
    $0 --profile high --duration 300

REQUISITOS:
    - curl debe estar instalado
    - El servidor debe estar corriendo
    - Suficiente memoria para clientes concurrentes

EOF
}

# Parsear argumentos
PROFILE="low"
DURATION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --server)
            SERVER_URL="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "Opción desconocida: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validar perfil
if [[ ! "$PROFILE" =~ ^(low|medium|high)$ ]]; then
    echo -e "${RED}Error: Perfil inválido '$PROFILE'. Debe ser: low, medium, high${NC}"
    exit 1
fi

# Configurar parámetros según perfil
case $PROFILE in
    low)
        CLIENTS=10
        REQUESTS=100
        [ -z "$DURATION" ] && DURATION=60
        ;;
    medium)
        CLIENTS=50
        REQUESTS=500
        [ -z "$DURATION" ] && DURATION=120
        ;;
    high)
        CLIENTS=100
        REQUESTS=1000
        [ -z "$DURATION" ] && DURATION=180
        ;;
esac

REQUESTS_PER_CLIENT=$((REQUESTS / CLIENTS))

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  Stress Test - Perfil: ${YELLOW}${PROFILE^^}${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo "Configuración:"
echo "  Servidor: $SERVER_URL"
echo "  Clientes concurrentes: $CLIENTS"
echo "  Requests totales: $REQUESTS"
echo "  Requests por cliente: $REQUESTS_PER_CLIENT"
echo "  Duración máxima: ${DURATION}s"
echo "  Output: $OUTPUT_DIR/stress_${PROFILE}_$(date +%Y%m%d_%H%M%S).log"
echo ""

# Verificar que el servidor está corriendo
echo -n "Verificando servidor... "
if ! curl -s -o /dev/null -w "%{http_code}" "$SERVER_URL/status" | grep -q "200"; then
    echo -e "${RED}✗ FALLÓ${NC}"
    echo -e "${RED}El servidor no está respondiendo en $SERVER_URL${NC}"
    exit 1
fi
echo -e "${GREEN}✓ OK${NC}"
echo ""

# Crear archivo de log
LOG_FILE="$OUTPUT_DIR/stress_${PROFILE}_$(date +%Y%m%d_%H%M%S).log"

# =============================================================================
# DEFINIR ENDPOINTS A PROBAR
# =============================================================================

# Mezcla de comandos básicos, CPU-bound e IO-bound (SOLO COMANDOS VÁLIDOS)
ENDPOINTS=(
    "/status"
    "/fibonacci?num=30"
    "/reverse?text=HelloWorld"
    "/toupper?text=lowercase"
    "/timestamp"
    "/random?min=1&max=1000"
    "/hash?text=TestData123&algo=sha256"
    "/help"
    "/fibonacci?num=20"
    "/reverse?text=StressTest"
    "/toupper?text=testing"
    "/random?min=1&max=100"
    "/fibonacci?num=25"
    "/timestamp"
    "/status"
)

NUM_ENDPOINTS=${#ENDPOINTS[@]}

# =============================================================================
# FUNCIÓN PARA EJECUTAR UN CLIENTE
# =============================================================================
run_client() {
    local client_id=$1
    local num_requests=$2
    local output_file="$OUTPUT_DIR/client_${client_id}.tmp"
    
    for i in $(seq 1 $num_requests); do
        # Seleccionar endpoint aleatorio
        local endpoint_idx=$((RANDOM % NUM_ENDPOINTS))
        local endpoint="${ENDPOINTS[$endpoint_idx]}"
        local url="$SERVER_URL$endpoint"
        
        # Realizar request y medir tiempo
        local start_time=$(date +%s%3N)  # Milisegundos
        local http_code=$(curl -s -o /dev/null -w "%{http_code}" \
                              -m 30 \
                              "$url" 2>/dev/null || echo "000")
        local end_time=$(date +%s%3N)
        local latency=$((end_time - start_time))
        
        # Registrar resultado
        echo "$client_id,$i,$endpoint,$http_code,$latency" >> "$output_file"
        
        # Pequeña pausa para no saturar inmediatamente (opcional)
        sleep 0.01
    done
}

# =============================================================================
# EJECUTAR STRESS TEST
# =============================================================================

echo -e "${YELLOW}Iniciando stress test...${NC}"
echo ""

START_TIME=$(date +%s)

# Lanzar clientes en paralelo
for client_id in $(seq 1 $CLIENTS); do
    run_client $client_id $REQUESTS_PER_CLIENT &
    
    # Mostrar progreso cada 10 clientes
    if [ $((client_id % 10)) -eq 0 ]; then
        echo -ne "\rClientes lanzados: $client_id/$CLIENTS"
    fi
done

echo -ne "\rClientes lanzados: $CLIENTS/$CLIENTS"
echo ""
echo "Esperando a que terminen todos los clientes..."

# Esperar a que todos los procesos terminen
wait

END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))

echo -e "${GREEN}✓ Todos los clientes han terminado${NC}"
echo ""

# =============================================================================
# PROCESAR RESULTADOS
# =============================================================================

echo "Procesando resultados..."

# Consolidar todos los resultados
cat $OUTPUT_DIR/client_*.tmp > "$LOG_FILE"
rm -f $OUTPUT_DIR/client_*.tmp

# Calcular estadísticas
TOTAL_REQUESTS=$(wc -l < "$LOG_FILE")
SUCCESS_REQUESTS=$(grep -c ",200," "$LOG_FILE" || echo 0)
ERROR_REQUESTS=$((TOTAL_REQUESTS - SUCCESS_REQUESTS))

# Extraer latencias para análisis
awk -F, '{print $5}' "$LOG_FILE" | sort -n > "$OUTPUT_DIR/latencies.tmp"

# Calcular percentiles
P50=$(awk 'NR==int((0.50*NR)+0.5)' "$OUTPUT_DIR/latencies.tmp")
P95=$(awk 'NR==int((0.95*NR)+0.5)' "$OUTPUT_DIR/latencies.tmp")
P99=$(awk 'NR==int((0.99*NR)+0.5)' "$OUTPUT_DIR/latencies.tmp")
MIN=$(head -1 "$OUTPUT_DIR/latencies.tmp")
MAX=$(tail -1 "$OUTPUT_DIR/latencies.tmp")
MEAN=$(awk '{sum+=$1} END {print int(sum/NR)}' "$OUTPUT_DIR/latencies.tmp")

# Calcular throughput
THROUGHPUT=$(echo "scale=2; $TOTAL_REQUESTS / $TOTAL_TIME" | bc)

# =============================================================================
# MOSTRAR RESULTADOS
# =============================================================================

echo ""
echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  Resultados del Stress Test${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo "Métricas Generales:"
echo "  Tiempo total: ${TOTAL_TIME}s"
echo "  Requests totales: $TOTAL_REQUESTS"
echo "  Requests exitosos: $SUCCESS_REQUESTS ($(( SUCCESS_REQUESTS * 100 / TOTAL_REQUESTS ))%)"
echo "  Requests fallidos: $ERROR_REQUESTS"
echo "  Throughput: ${THROUGHPUT} req/s"
echo ""
echo "Latencias (ms):"
echo "  Mínima: ${MIN}ms"
echo "  Media: ${MEAN}ms"
echo "  p50: ${P50}ms"
echo "  p95: ${P95}ms"
echo "  p99: ${P99}ms"
echo "  Máxima: ${MAX}ms"
echo ""
echo "Códigos HTTP:"
grep -o ",[0-9]\{3\}," "$LOG_FILE" | sort | uniq -c | sort -rn | while read count code; do
    code=$(echo $code | tr -d ',')
    echo "  $code: $count requests"
done
echo ""
echo "Archivo de log: $LOG_FILE"
echo ""

# Limpiar archivos temporales
rm -f "$OUTPUT_DIR/latencies.tmp"

# =============================================================================
# GENERAR REPORTE JSON
# =============================================================================

JSON_FILE="$OUTPUT_DIR/report_${PROFILE}_$(date +%Y%m%d_%H%M%S).json"

cat > "$JSON_FILE" << EOF
{
  "profile": "$PROFILE",
  "timestamp": "$(date -I'seconds')",
  "config": {
    "server": "$SERVER_URL",
    "clients": $CLIENTS,
    "total_requests": $REQUESTS,
    "requests_per_client": $REQUESTS_PER_CLIENT,
    "duration_seconds": $TOTAL_TIME
  },
  "results": {
    "requests": {
      "total": $TOTAL_REQUESTS,
      "successful": $SUCCESS_REQUESTS,
      "failed": $ERROR_REQUESTS,
      "success_rate": $(echo "scale=4; $SUCCESS_REQUESTS * 100 / $TOTAL_REQUESTS" | bc)
    },
    "throughput": {
      "requests_per_second": $THROUGHPUT
    },
    "latency_ms": {
      "min": $MIN,
      "mean": $MEAN,
      "p50": $P50,
      "p95": $P95,
      "p99": $P99,
      "max": $MAX
    }
  },
  "log_file": "$LOG_FILE"
}
EOF

echo "Reporte JSON: $JSON_FILE"
echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}  Stress Test Completado Exitosamente${NC}"
echo -e "${GREEN}================================================${NC}"

# Retornar código de salida basado en tasa de éxito
if [ $SUCCESS_REQUESTS -lt $((TOTAL_REQUESTS * 90 / 100)) ]; then
    echo -e "${RED}⚠ Advertencia: Tasa de éxito < 90%${NC}"
    exit 1
fi

exit 0