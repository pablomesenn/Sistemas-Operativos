#!/bin/bash
# Script de pruebas de carga para el servidor HTTP
# Cumple con los requisitos del PDF de realizar pruebas bajo 3 perfiles de carga

set -e

SERVER_URL="${SERVER_URL:-http://127.0.0.1:8080}"
RESULTS_DIR="./test_results"

mkdir -p "$RESULTS_DIR"

echo "╔════════════════════════════════════════════════════╗"
echo "║     RedUnix HTTP Server - Load Testing Suite      ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""
echo "Server: $SERVER_URL"
echo "Results will be saved to: $RESULTS_DIR"
echo ""

# Función para realizar pruebas
run_test() {
    local test_name=$1
    local endpoint=$2
    local concurrent=$3
    local total=$4
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Test: $test_name"
    echo "Endpoint: $endpoint"
    echo "Concurrent: $concurrent | Total requests: $total"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Usar Apache Bench si está disponible
    if command -v ab &> /dev/null; then
        ab -n "$total" -c "$concurrent" "$SERVER_URL$endpoint" > "$RESULTS_DIR/${test_name}.txt" 2>&1
        
        # Extraer métricas clave
        grep "Time taken for tests:" "$RESULTS_DIR/${test_name}.txt" || true
        grep "Requests per second:" "$RESULTS_DIR/${test_name}.txt" || true
        grep "Time per request:" "$RESULTS_DIR/${test_name}.txt" || true
        grep "Percentage of the requests served within" -A 10 "$RESULTS_DIR/${test_name}.txt" || true
    else
        echo "⚠️  Apache Bench (ab) not found. Using curl instead..."
        
        local start_time=$(date +%s.%N)
        
        for i in $(seq 1 "$total"); do
            curl -s "$SERVER_URL$endpoint" > /dev/null &
            
            # Limitar concurrencia
            if [ $((i % concurrent)) -eq 0 ]; then
                wait
            fi
        done
        
        wait
        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc)
        local rps=$(echo "scale=2; $total / $duration" | bc)
        
        echo "Total time: ${duration}s"
        echo "Requests/sec: $rps"
    fi
    
    echo ""
}

# Perfil 1: Baja carga - Comandos básicos
echo "🔹 PROFILE 1: Low Load - Basic Commands"
run_test "profile1_fibonacci" "/fibonacci?num=20" 5 100
run_test "profile1_reverse" "/reverse?text=hello" 5 100
run_test "profile1_status" "/status" 5 100

echo ""

# Perfil 2: Carga media - Mix de CPU y comandos básicos
echo "🔸 PROFILE 2: Medium Load - Mixed Workload"
run_test "profile2_isprime" "/isprime?n=104729" 10 200
run_test "profile2_fibonacci" "/fibonacci?num=35" 10 200
run_test "profile2_hash" "/hash?text=test123" 10 200

echo ""

# Perfil 3: Alta carga - CPU-intensive
echo "🔺 PROFILE 3: High Load - CPU-Intensive"
run_test "profile3_isprime_large" "/isprime?n=982451653" 20 100
run_test "profile3_factor" "/factor?n=123456789" 20 100
run_test "profile3_pi" "/pi?digits=100" 20 100

echo ""

# Test de Jobs
echo "🔧 TESTING JOB SYSTEM"
echo "Submitting long-running jobs..."

for i in {1..10}; do
    JOB_ID=$(curl -s "$SERVER_URL/jobs/submit?task=isprime&n=999999937&prio=normal" | grep -o '"job_id":"[^"]*' | cut -d'"' -f4)
    echo "  Job $i submitted: $JOB_ID"
    sleep 0.1
done

echo ""
echo "Checking job statuses..."
sleep 2

# Test de métricas finales
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Final Metrics"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s "$SERVER_URL/metrics" | python3 -m json.tool || curl -s "$SERVER_URL/metrics"

echo ""
echo "✅ Load testing complete!"
echo "Results saved in: $RESULTS_DIR"