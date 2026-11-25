# Depósito de alquiler

Un contrato en Stellar que guarda el depósito de un alquiler hasta que las dos partes deciden cómo se devuelve.

Esta entrega es una práctica individual. No es la idea de la hackathon.

## El problema

En Costa Rica, quien alquila un cuarto o un apartamento (el inquilino) deja un depósito. Casi siempre es un mes de alquiler.

Hoy ese dinero se transfiere a la cuenta personal de quien alquila el lugar (el arrendador). La devolución queda en una promesa: “al final te lo devuelvo si todo está bien”.

Si después hay un daño, un recibo pendiente o un “no te lo devuelvo”, no existe un tercero que tenga la plata. Todo el riesgo lo carga una de las dos personas. El problema es real: pasa en alquileres de todos los días y no hay un lugar neutral donde dejar el depósito mientras se ponen de acuerdo.

## Por qué lo elegí

Es concreto y pequeño. Hay dos personas, un monto y solo dos finales. Cabe en un solo contrato: un despliegue es un solo alquiler. No intenta resolver todos los alquileres del país, ni es la idea de la hackathon.

## Qué hace

El contrato hace de tercero. Recibe el depósito en un token de Stellar y solo lo suelta de dos maneras: el arrendador lo devuelve completo, o arrendador e inquilino firman cuánto se queda cada uno.

## Cómo funciona

El contrato usa lo visto en clase: las claves van en un `DataKey`, los datos viven en el storage de la instancia, se renueva el tiempo de vida de esos datos y cada acción la firma quien corresponde con `require_auth`.

1. El arrendador llama a `crear` con su dirección, la del inquilino, el token y el monto. El monto va en la unidad mínima del token. Si el token es XLM, 1 XLM son 10.000.000. Solo el arrendador puede firmar este paso, y las dos direcciones tienen que ser distintas.
2. El inquilino llama a `depositar`. El token pasa de su cuenta al contrato. El estado queda en custodia.
3. A partir de ahí solo hay dos salidas:
   - `devolver`: la firma el arrendador. El inquilino recibe todo.
   - `acordar_reparto`: la firman los dos. Se indica cuánto se queda el arrendador. El resto vuelve al inquilino. Ese monto no puede ser mayor que el depósito.

Si el contrato ya se cerró, no se puede volver a mover el dinero.

## Verlo en la red

Ya hay un despliegue en la testnet, con 5 XLM en custodia. En el explorador el estado sale como `1`, que es `EnCustodia`, y el saldo es `50000000` (5 XLM).

- [Ver el contrato en Stellar Expert](https://stellar.expert/explorer/testnet/contract/CDRJTDQO3BZSCNDT2BMINXB34A4XQJ4AZRXQJ3IKH2KCIJEBXMTFOJIK)
- [Verlo en Stellar Lab](https://lab.stellar.org/r/testnet/contract/CDRJTDQO3BZSCNDT2BMINXB34A4XQJ4AZRXQJ3IKH2KCIJEBXMTFOJIK)

Contrato: `CDRJTDQO3BZSCNDT2BMINXB34A4XQJ4AZRXQJ3IKH2KCIJEBXMTFOJIK`

- Arrendador: `GCZDY6BSRP65RGUS43YBE2S4GPAYKIXGUKDCQOPFXQPGYWFRUPQLIC5R`
- Inquilino: `GDNR2FLJ2SW4Q7W3S46FDA27IM3YYAJLMIYUG55I32G6SAAMFRVWA2YG`

## Cómo correrlo

Hace falta [Rust](https://rustup.rs/) estable. En la carpeta del proyecto:

```bash
cargo test
```

Eso ejecuta las pruebas sin desplegar nada. Para compilar el Wasm también hace falta la [CLI de Stellar](https://developers.stellar.org/docs/tools/cli/install) 28 y el target `wasm32v1-none`:

```bash
rustup target add wasm32v1-none
stellar contract build
```

El archivo queda en `target/wasm32v1-none/release/deposito_alquiler.wasm`.

## Pruebas

`cargo test` corre estos casos y todos pasan:

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
