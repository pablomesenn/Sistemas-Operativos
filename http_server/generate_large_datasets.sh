#!/bin/bash
# Script OPTIMIZADO para generar datasets grandes
set -e

DATA_DIR="./data"
mkdir -p "$DATA_DIR"

echo "================================================"
echo "  Generando Datasets Grandes (VERSIÓN RÁPIDA)"
echo "================================================"
echo ""

# =============================================================================
# 1. ARCHIVO DE NÚMEROS (50MB) - OPTIMIZADO
# =============================================================================
echo "[1/4] Generando large_numbers.txt (50MB)..."

# Generar en bloques grandes es MUCHO más rápido
{
    for i in {1..50000}; do
        # Generar 100 números por iteración
        for j in {1..100}; do
            echo $((RANDOM * RANDOM))
        done
    done
} > "$DATA_DIR/large_numbers.txt"

SIZE=$(du -h "$DATA_DIR/large_numbers.txt" | cut -f1)
echo "   ✓ Creado: large_numbers.txt ($SIZE)"

# =============================================================================
# 2. ARCHIVO DE TEXTO (50MB) - OPTIMIZADO
# =============================================================================
echo "[2/4] Generando large_text.txt (50MB)..."

{
    for i in {1..100000}; do
        echo "[ERROR] 2024-10-27 Line $i: Processing failed in module_$((i % 100))"
        echo "[WARN] 2024-10-27 Line $i: Warning message in operation $((i % 50))"
        echo "[INFO] 2024-10-27 Line $i: Successfully completed task number $i"
        echo "[DEBUG] 2024-10-27 Line $i: Debug information for request $((i * 7))"
        echo "[TRACE] 2024-10-27 Line $i: Trace details for transaction $((i + 1000))"
    done
} > "$DATA_DIR/large_text.txt"

SIZE=$(du -h "$DATA_DIR/large_text.txt" | cut -f1)
echo "   ✓ Creado: large_text.txt ($SIZE)"

# =============================================================================
# 3. ARCHIVO BINARIO (50MB) - OPTIMIZADO
# =============================================================================
echo "[3/4] Generando large_binary.txt (50MB)..."

{
    for i in {1..50000}; do
        echo "Este es un texto repetitivo número $i que será comprimido eficientemente por gzip."
        echo "Los datos repetitivos comprimen muy bien con algoritmos como gzip y deflate."
        echo "Línea adicional para incrementar el tamaño del archivo de prueba de compresión."
        echo "Compresión GZIP es excelente para texto con patrones y datos repetitivos."
        echo "Más contenido repetitivo para testing: $i $i $i $i $i $i $i $i $i $i"
        echo "Datos adicionales para alcanzar el tamaño objetivo de 50MB en el archivo."
        echo "Contenido $i con información que se repite para mejorar ratio de compresión."
        echo "Testing testing 123 testing testing 456 testing testing 789 for file $i"
        echo "Final line for block $i with some additional padding content here now."
        echo "========================================================================="
    done
} > "$DATA_DIR/large_binary.txt"

SIZE=$(du -h "$DATA_DIR/large_binary.txt" | cut -f1)
echo "   ✓ Creado: large_binary.txt ($SIZE)"

# =============================================================================
# 4. ARCHIVO PARA HASH (50MB) - OPTIMIZADO
# =============================================================================
echo "[4/4] Generando large_hash.txt (50MB)..."

{
    for i in {1..100000}; do
        echo "Hash test data block $i with unique content timestamp $((i * 12345))"
        echo "Additional hash content line $i for SHA256 computation testing purposes"
        echo "Random-like data: $((RANDOM))$((RANDOM))$((RANDOM))$((RANDOM))$((RANDOM))"
        echo "More content for hashing algorithm verification at block number $i here"
        echo "Final hash line $i =============================================="
    done
} > "$DATA_DIR/large_hash.txt"

SIZE=$(du -h "$DATA_DIR/large_hash.txt" | cut -f1)
echo "   ✓ Creado: large_hash.txt ($SIZE)"

# =============================================================================
# RESUMEN
# =============================================================================
echo ""
echo "================================================"
echo "  ✓ Datasets Generados Exitosamente"
echo "================================================"
echo ""
ls -lh "$DATA_DIR"/large_*.txt
echo ""
echo "Comandos de prueba:"
echo "  curl 'http://localhost:8080/sortfile?name=large_numbers.txt&algo=merge'"
echo "  curl 'http://localhost:8080/wordcount?name=large_text.txt'"
echo "  curl 'http://localhost:8080/grep?name=large_text.txt&pattern=ERROR'"
echo "  curl 'http://localhost:8080/compress?name=large_binary.txt&codec=gzip'"
echo "  curl 'http://localhost:8080/hashfile?name=large_hash.txt&algo=sha256'"
echo ""