#!/bin/bash

# Script para verificar coverage en WSL/Ubuntu sin tarpaulin
# Usa cargo-llvm-cov como alternativa

echo "=========================================="
echo "VERIFICACIÓN DE COVERAGE - WSL/Ubuntu"
echo "=========================================="
echo ""

# Colores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Verificar que estamos en el directorio correcto
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: No se encuentra Cargo.toml${NC}"
    echo "Ejecutar desde el directorio raíz del proyecto"
    exit 1
fi

echo "📁 Directorio actual: $(pwd)"
echo ""

# Paso 1: Verificar Rust
echo "1️⃣  Verificando instalación de Rust..."
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust no está instalado${NC}"
    echo "Instalar con: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

RUST_VERSION=$(rustc --version)
echo -e "${GREEN}✅ Rust instalado: $RUST_VERSION${NC}"
echo ""

# Paso 2: Instalar cargo-llvm-cov
echo "2️⃣  Verificando cargo-llvm-cov..."
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo -e "${YELLOW}⚠️  cargo-llvm-cov no está instalado${NC}"
    echo ""
    echo "Instalando cargo-llvm-cov..."
    cargo install cargo-llvm-cov
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Error al instalar cargo-llvm-cov${NC}"
        echo ""
        echo "Alternativa: Usar grcov"
        echo "  cargo install grcov"
        echo "  export CARGO_INCREMENTAL=0"
        echo "  export RUSTFLAGS='-Cinstrument-coverage'"
        echo "  cargo build"
        echo "  cargo test"
        echo "  grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing -o target/coverage/"
        exit 1
    fi
else
    LLVM_COV_VERSION=$(cargo-llvm-cov --version)
    echo -e "${GREEN}✅ cargo-llvm-cov instalado: $LLVM_COV_VERSION${NC}"
fi
echo ""

# Paso 3: Limpiar builds anteriores
echo "3️⃣  Limpiando builds anteriores..."
cargo clean
echo -e "${GREEN}✅ Build limpiado${NC}"
echo ""

# Paso 4: Ejecutar tests con coverage
echo "4️⃣  Ejecutando tests con coverage..."
echo "Esto puede tomar varios minutos..."
echo ""

# Ejecutar con HTML output
cargo llvm-cov --html --output-dir target/coverage

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ Coverage generado exitosamente${NC}"
    echo ""
    
    # Buscar el archivo index.html
    if [ -f "target/coverage/html/index.html" ]; then
        COVERAGE_FILE="target/coverage/html/index.html"
    elif [ -f "target/coverage/index.html" ]; then
        COVERAGE_FILE="target/coverage/index.html"
    else
        COVERAGE_FILE=$(find target/coverage -name "index.html" | head -1)
    fi
    
    if [ -n "$COVERAGE_FILE" ]; then
        echo "📊 Reporte de coverage generado en:"
        echo "   $COVERAGE_FILE"
        echo ""
        
        # Intentar extraer el porcentaje de coverage
        if command -v grep &> /dev/null; then
            COVERAGE_PCT=$(grep -oP 'Coverage: \K[0-9.]+%' "$COVERAGE_FILE" 2>/dev/null || \
                          grep -oP '[0-9.]+%' "$COVERAGE_FILE" 2>/dev/null | head -1)
            
            if [ -n "$COVERAGE_PCT" ]; then
                echo "📈 Coverage total: $COVERAGE_PCT"
                echo ""
                
                # Verificar si cumple con el requisito
                COVERAGE_NUM=$(echo $COVERAGE_PCT | sed 's/%//')
                if (( $(echo "$COVERAGE_NUM >= 90" | bc -l 2>/dev/null || echo "0") )); then
                    echo -e "${GREEN}✅ CUMPLE con el requisito de >= 90%${NC}"
                elif (( $(echo "$COVERAGE_NUM >= 80" | bc -l 2>/dev/null || echo "0") )); then
                    echo -e "${YELLOW}⚠️  CERCA del objetivo (>= 80%, objetivo 90%)${NC}"
                else
                    echo -e "${RED}❌ Por debajo del objetivo de 90%${NC}"
                fi
                echo ""
            fi
        fi
        
        echo "Para ver el reporte completo:"
        echo "  1. Abrir en navegador:"
        if command -v wslview &> /dev/null; then
            echo "     wslview $COVERAGE_FILE"
        else
            echo "     xdg-open $COVERAGE_FILE"
            echo "     o copiar la ruta y abrir en navegador Windows"
        fi
        echo ""
        echo "  2. O ver en terminal:"
        echo "     cargo llvm-cov --text"
        echo ""
    fi
    
    # Generar también reporte en texto
    echo "5️⃣  Generando reporte de texto..."
    cargo llvm-cov --text > target/coverage/coverage_report.txt
    
    if [ -f "target/coverage/coverage_report.txt" ]; then
        echo -e "${GREEN}✅ Reporte de texto generado${NC}"
        echo "   target/coverage/coverage_report.txt"
        echo ""
        echo "Primeras líneas del reporte:"
        head -30 target/coverage/coverage_report.txt
    fi
    
else
    echo -e "${RED}❌ Error al generar coverage${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Verificar que todos los tests pasen: cargo test"
    echo "  2. Verificar instalación de LLVM: llvm-config --version"
    echo "  3. Reinstalar cargo-llvm-cov: cargo install --force cargo-llvm-cov"
    echo ""
    exit 1
fi

echo ""
echo "=========================================="
echo "RESUMEN"
echo "=========================================="
echo ""
echo "✅ Tests ejecutados con instrumentación de coverage"
echo "📊 Reporte HTML generado en target/coverage/"
echo "📄 Reporte de texto en target/coverage/coverage_report.txt"
echo ""
echo "Siguiente paso:"
echo "  Revisar el reporte HTML para verificar que se cumple con >= 90%"
echo ""