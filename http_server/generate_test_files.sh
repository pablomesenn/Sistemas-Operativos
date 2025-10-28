#!/bin/bash
# Script para generar archivos de prueba para io_bound tests

echo "🔧 Generating test files for io_bound tests..."

# Crear directorio data si no existe
mkdir -p ./data

# 1. Archivo de números para sortfile (pequeño para tests)
echo "📝 Creating numbers file..."
for i in $(seq 1 100); do
    echo $((RANDOM % 1000))
done > ./data/test_numbers.txt

# 2. Archivo de texto para wordcount
echo "📝 Creating text file..."
cat > ./data/test_text.txt << 'EOF'
Hello world this is a test file.
This file contains multiple lines.
We will use this for wordcount and grep tests.
The quick brown fox jumps over the lazy dog.
Testing testing one two three.
EOF

# 3. Archivo con patrón específico para grep
echo "📝 Creating grep test file..."
cat > ./data/test_grep.txt << 'EOF'
ERROR: Something went wrong
INFO: Starting process
ERROR: Another error occurred
WARNING: Check this
INFO: Process completed
ERROR: Third error
DEBUG: Debug information
EOF

# 4. Archivo para compress (más grande)
echo "📝 Creating file for compression..."
cat > ./data/test_compress.txt << 'EOF'
This is a test file for compression.
Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.
This line is repeated multiple times to improve compression ratio.
This line is repeated multiple times to improve compression ratio.
This line is repeated multiple times to improve compression ratio.
This line is repeated multiple times to improve compression ratio.
This line is repeated multiple times to improve compression ratio.
EOF

# 5. Archivo para hash
echo "📝 Creating file for hash..."
echo "Hello SHA256!" > ./data/test_hash.txt

echo "✅ Test files created successfully!"
echo ""
echo "Files created:"
ls -lh ./data/test_*.txt