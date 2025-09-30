// Product module, for defining products and related functions

struct Producto {
    id: u32,
    nombre: String,
    tiempo_llegada: Instant,
    tiempo_entrada_corte: Option<Instant>,
    tiempo_salida_corte: Option<Instant>,
    tiempo_entrada_ensamblaje: Option<Instant>,
    tiempo_salida_ensamblaje: Option<Instant>,
    tiempo_entrada_empaque: Option<Instant>,
    tiempo_salida_empaque: Option<Instant>,
}

impl Producto {
    fn new(id: u32, nombre: String) -> Self {
        Producto {
            id,
            nombre,
            tiempo_llegada: Instant::now(),
            tiempo_entrada_corte: None,
            tiempo_salida_corte: None,
            tiempo_entrada_ensamblaje: None,
            tiempo_salida_ensamblaje: None,
            tiempo_entrada_empaque: None,
            tiempo_salida_empaque: None,
        }
    }

    fn display(&self) {
        println!(
            "Product ID: {}, Name: {}",
            self.id, self.nombre
        );
    }
}