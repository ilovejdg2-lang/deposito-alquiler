#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};

fn preparar(env: &Env) -> (DepositoAlquilerClient<'_>, Address, Address, Address) {
    env.mock_all_auths();

    let client = DepositoAlquilerClient::new(env, &env.register(DepositoAlquiler, ()));
    let arrendador = Address::generate(env);
    let inquilino = Address::generate(env);
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin).address();

    StellarAssetClient::new(env, &token).mint(&inquilino, &1_000_000);

    (client, arrendador, inquilino, token)
}

fn saldo_de(env: &Env, token: &Address, quien: &Address) -> i128 {
    TokenClient::new(env, token).balance(quien)
}

#[test]
fn crear_guarda_las_partes_y_el_monto() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);

    client.crear(&arrendador, &inquilino, &token, &400_000);

    assert_eq!(client.arrendador(), arrendador);
    assert_eq!(client.inquilino(), inquilino);
    assert_eq!(client.token(), token);
    assert_eq!(client.monto(), 400_000);
    assert_eq!(client.estado(), Estado::EsperandoDeposito);
    assert_eq!(client.saldo(), 0);
}

#[test]
fn depositar_mueve_el_token_y_cambia_el_estado() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);
    client.crear(&arrendador, &inquilino, &token, &400_000);

    client.depositar();

    assert_eq!(client.estado(), Estado::EnCustodia);
    assert_eq!(client.saldo(), 400_000);
    assert_eq!(saldo_de(&env, &token, &inquilino), 600_000);
    assert_eq!(saldo_de(&env, &token, &client.address), 400_000);
}

#[test]
fn no_se_puede_depositar_dos_veces() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);
    client.crear(&arrendador, &inquilino, &token, &400_000);
    client.depositar();

    assert_eq!(client.try_depositar(), Err(Ok(Error::EstadoIncorrecto)));
    assert_eq!(client.saldo(), 400_000);
}

#[test]
fn monto_en_cero_falla() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);

    assert_eq!(
        client.try_crear(&arrendador, &inquilino, &token, &0),
        Err(Ok(Error::MontoInvalido))
    );
    assert_eq!(client.try_estado(), Err(Ok(Error::NoIniciado)));
}

#[test]
fn las_dos_partes_no_pueden_ser_la_misma() {
    let env = Env::default();
    let (client, arrendador, _, token) = preparar(&env);

    assert_eq!(
        client.try_crear(&arrendador, &arrendador, &token, &400_000),
        Err(Ok(Error::MismasPartes))
    );
}

#[test]
fn no_se_puede_crear_dos_veces() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);
    client.crear(&arrendador, &inquilino, &token, &400_000);

    assert_eq!(
        client.try_crear(&arrendador, &inquilino, &token, &400_000),
        Err(Ok(Error::YaIniciado))
    );
}

#[test]
fn devolver_regresa_el_deposito_completo() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);
    client.crear(&arrendador, &inquilino, &token, &400_000);
    client.depositar();

    client.devolver();

    assert_eq!(client.estado(), Estado::Devuelto);
    assert_eq!(client.saldo(), 0);
    assert_eq!(saldo_de(&env, &token, &inquilino), 1_000_000);
    assert_eq!(saldo_de(&env, &token, &arrendador), 0);
}

#[test]
fn acordar_reparto_divide_el_deposito() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);
    client.crear(&arrendador, &inquilino, &token, &400_000);
    client.depositar();

    client.acordar_reparto(&150_000);

    assert_eq!(client.estado(), Estado::Repartido);
    assert_eq!(client.saldo(), 0);
    assert_eq!(saldo_de(&env, &token, &arrendador), 150_000);
    assert_eq!(saldo_de(&env, &token, &inquilino), 850_000);
}

#[test]
fn un_reclamo_mayor_al_deposito_falla() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);
    client.crear(&arrendador, &inquilino, &token, &400_000);
    client.depositar();

    assert_eq!(
        client.try_acordar_reparto(&400_001),
        Err(Ok(Error::MontoInvalido))
    );
    assert_eq!(client.estado(), Estado::EnCustodia);
    assert_eq!(client.saldo(), 400_000);
}

#[test]
fn no_se_mueve_dinero_si_ya_esta_cerrado() {
    let env = Env::default();
    let (client, arrendador, inquilino, token) = preparar(&env);
    client.crear(&arrendador, &inquilino, &token, &400_000);
    client.depositar();
    client.devolver();

    assert_eq!(client.try_devolver(), Err(Ok(Error::EstadoIncorrecto)));
    assert_eq!(
        client.try_acordar_reparto(&100_000),
        Err(Ok(Error::EstadoIncorrecto))
    );
    assert_eq!(client.try_depositar(), Err(Ok(Error::EstadoIncorrecto)));
    assert_eq!(saldo_de(&env, &token, &inquilino), 1_000_000);
    assert_eq!(client.saldo(), 0);
}
