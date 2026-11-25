# Depósito de alquiler

Un contrato en Stellar que guarda el depósito de un alquiler hasta que las dos partes deciden cómo se devuelve.

Esta entrega es una práctica individual. No es la idea de la hackathon.

## El problema

En Costa Rica, cuando alguien alquila un cuarto o un apartamento, deja un depósito. Casi siempre es un mes de alquiler.

Ese dinero queda en la cuenta de quien alquila. Si al final hay un pleito —un daño, un recibo pendiente, o un “no te lo devuelvo”— no hay un lugar neutral que tenga la plata mientras se ponen de acuerdo. Una de las dos personas depende de la buena fe de la otra.

## Por qué este problema

Pasa de verdad, y es pequeño. Caben dos personas, un monto y dos finales posibles. No hace falta un sistema de muchos alquileres: un despliegue del contrato es un solo contrato de arrendamiento.

## Qué hace

El contrato recibe el depósito en un token de Stellar y solo lo suelta de dos maneras: el arrendador lo devuelve completo, o arrendador e inquilino firman un reparto.

## Cómo funciona

1. El arrendador llama a `crear` con su dirección, la del inquilino, el token y el monto del depósito. El monto va en la unidad mínima del token. Si el token es XLM, 1 XLM son 10.000.000.
2. El inquilino llama a `depositar`. El token pasa de su cuenta al contrato. El estado queda en custodia.
3. A partir de ahí solo hay dos salidas:
   - `devolver`: la firma el arrendador. El inquilino recibe todo.
   - `acordar_reparto`: la firman los dos. Se indica cuánto se queda el arrendador (por daños u otra deuda). El resto vuelve al inquilino.

Nadie más puede abrir el depósito, depositar ni mover el dinero. Si el contrato ya se cerró, no se puede volver a mover.

## Cómo correrlo

Hace falta [Rust](https://rustup.rs/) estable. Las pruebas no necesitan desplegar nada en la red.

En la carpeta del proyecto:

```bash
cargo test
```

Para compilar el Wasm del contrato hace falta, además, la [CLI de Stellar](https://developers.stellar.org/docs/tools/cli/install) de la misma serie que el SDK (aquí `soroban-sdk` 28) y el target `wasm32v1-none`:

```bash
rustup target add wasm32v1-none
stellar contract build
```

El archivo queda en `target/wasm32v1-none/release/deposito_alquiler.wasm`.

## Pruebas

`cargo test` corre estos casos:

- `crear` guarda al arrendador, al inquilino, el token y el monto.
- `depositar` mueve el token al contrato y deja el estado en custodia.
- Un segundo depósito falla.
- Un monto en cero falla, y el contrato no queda iniciado.
- Arrendador e inquilino no pueden ser la misma dirección.
- No se puede llamar a `crear` dos veces.
- `devolver` regresa el depósito completo al inquilino y el contrato queda en cero.
- `acordar_reparto` parte el dinero: una parte para el arrendador y el resto para el inquilino.
- Un reclamo mayor que el depósito falla y el dinero sigue en custodia.
- Después de cerrado, `devolver`, `acordar_reparto` y `depositar` fallan y los saldos no cambian.
