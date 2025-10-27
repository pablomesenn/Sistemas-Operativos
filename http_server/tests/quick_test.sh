#!/bin/bash
# Script de verificación rápida v2

SERVER_URL="http://127.0.0.1:8080"

echo "🧪 RedUnix HTTP Server - Quick Verification v2"
echo "==============================================="
echo ""

# Función para probar un endpoint
test_endpoint() {
    local name=$1
    local endpoint=$2
    local expected=$3
    
    echo -n "Testing $name... "
    response=$(curl -s "$SERVER_URL$endpoint")
    
    if echo "$response" | grep -q "$expected"; then
        echo "✅ PASS"
        return 0
    else
        echo "❌ FAIL"
        echo "  Expected to find: $expected"
        echo "  Got: $response"
        return 1
    fi
}

# Verificar que el servidor está corriendo
echo "Checking if server is running..."
if ! curl -s "$SERVER_URL/status" > /dev/null; then
    echo "❌ Server is not running on $SERVER_URL"
    echo "   Start it with: ./target/release/http_server"
    exit 1
fi
echo "✅ Server is running"
echo ""

# Pruebas básicas
echo "📋 Basic Commands"
echo "─────────────────"
test_endpoint "fibonacci" "/fibonacci?num=10" "55"
test_endpoint "reverse" "/reverse?text=hello" "olleh"
test_endpoint "status" "/status" "running"
echo ""

# Pruebas CPU-bound
echo "🔢 CPU-bound Commands"
echo "─────────────────────"
test_endpoint "isprime" "/isprime?n=97" "true"
test_endpoint "factor" "/factor?n=12" '"factors"'  # CORREGIDO
echo ""

# Pruebas de Jobs
echo "⚙️  Job System"
echo "──────────────"
echo -n "Submitting job... "
JOB_RESPONSE=$(curl -s "$SERVER_URL/jobs/submit?task=isprime&n=104729&prio=normal")

# Debug: mostrar respuesta
# echo "Debug - Response: $JOB_RESPONSE"

# Extraer job_id de manera más robusta
JOB_ID=$(echo "$JOB_RESPONSE" | grep -o 'job-[a-f0-9]*' | head -n1)

if [ -n "$JOB_ID" ]; then
    echo "✅ PASS (Job ID: $JOB_ID)"
    
    echo -n "Checking job status... "
    sleep 1
    STATUS_RESPONSE=$(curl -s "$SERVER_URL/jobs/status?id=$JOB_ID")
    if echo "$STATUS_RESPONSE" | grep -q "status"; then
        echo "✅ PASS"
        
        # Intentar obtener resultado
        echo -n "Getting job result... "
        sleep 2
        RESULT_RESPONSE=$(curl -s "$SERVER_URL/jobs/result?id=$JOB_ID")
        if echo "$RESULT_RESPONSE" | grep -q "is_prime"; then
            echo "✅ PASS"
        else
            echo "⏳ PENDING (job still running)"
        fi
    else
        echo "❌ FAIL"
    fi
else
    echo "❌ FAIL - No job ID received"
    echo "   Response was: $JOB_RESPONSE"
fi
echo ""

# Verificar headers
echo "📊 Headers Verification"
echo "───────────────────────"
HEADERS=$(curl -s -v "$SERVER_URL/status" 2>&1)

echo -n "X-Request-Id... "
if echo "$HEADERS" | grep -q "X-Request-Id:"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo -n "X-Worker-Thread... "
if echo "$HEADERS" | grep -q "X-Worker-Thread:"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo -n "X-Worker-Pid... "
if echo "$HEADERS" | grep -q "X-Worker-Pid:"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi
echo ""

# Verificar métricas
echo "📈 Metrics Verification"
echo "───────────────────────"
METRICS=$(curl -s "$SERVER_URL/metrics")

echo -n "Has latency stats... "
if echo "$METRICS" | grep -q "p50"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo -n "Has stddev... "
if echo "$METRICS" | grep -q "stddev"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo -n "Has job queues... "
if echo "$METRICS" | grep -q "job_queues"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo ""
echo "==============================================="
echo "✅ Verification complete!"
echo ""
echo "💡 Tip: Check detailed metrics at /metrics"
echo "   curl -s http://127.0.0.1:8080/metrics | python3 -m json.tool"