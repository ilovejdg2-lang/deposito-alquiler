#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

const EXTENSION_UMBRAL: u32 = 518_400;
const EXTENSION_CANTIDAD: u32 = 535_680;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    YaIniciado = 1,
    NoIniciado = 2,
    MontoInvalido = 3,
    MismasPartes = 4,
    EstadoIncorrecto = 5,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Estado {
    EsperandoDeposito = 0,
    EnCustodia = 1,
    Devuelto = 2,
    Repartido = 3,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Arrendador,
    Inquilino,
    Token,
    Monto,
    Estado,
}

#[contract]
pub struct DepositoAlquiler;

#[contractimpl]
impl DepositoAlquiler {
    /// El arrendador abre la custodia de un solo alquiler.
    pub fn crear(
        env: Env,
        arrendador: Address,
        inquilino: Address,
        token: Address,
        monto: i128,
    ) -> Result<(), Error> {
        arrendador.require_auth();

        if env.storage().instance().has(&DataKey::Estado) {
            return Err(Error::YaIniciado);
        }
        if monto <= 0 {
            return Err(Error::MontoInvalido);
        }
        if arrendador == inquilino {
            return Err(Error::MismasPartes);
        }

        env.storage().instance().set(&DataKey::Arrendador, &arrendador);
        env.storage().instance().set(&DataKey::Inquilino, &inquilino);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Monto, &monto);
        env.storage()
            .instance()
            .set(&DataKey::Estado, &Estado::EsperandoDeposito);
        extender(&env);
        Ok(())
    }

    /// El inquilino transfiere el depósito al contrato.
    pub fn depositar(env: Env) -> Result<(), Error> {
        exigir_estado(&env, Estado::EsperandoDeposito)?;

        let inquilino = leer_inquilino(&env)?;
        inquilino.require_auth();

        let token = leer_token(&env)?;
        let monto = leer_monto(&env)?;
        token::Client::new(&env, &token).transfer(
            &inquilino,
            &env.current_contract_address(),
            &monto,
        );

        env.storage()
            .instance()
            .set(&DataKey::Estado, &Estado::EnCustodia);
        extender(&env);
        Ok(())
    }

    /// El arrendador devuelve el depósito completo al inquilino.
    pub fn devolver(env: Env) -> Result<(), Error> {
        exigir_estado(&env, Estado::EnCustodia)?;
        leer_arrendador(&env)?.require_auth();

        let token = leer_token(&env)?;
        let monto = leer_monto(&env)?;
        pagar(&env, &token, &leer_inquilino(&env)?, monto);

        env.storage()
            .instance()
            .set(&DataKey::Estado, &Estado::Devuelto);
        extender(&env);
        Ok(())
    }

    /// Las dos partes firman cuánto se queda el arrendador. El resto vuelve al inquilino.
    pub fn acordar_reparto(env: Env, monto_arrendador: i128) -> Result<(), Error> {
        exigir_estado(&env, Estado::EnCustodia)?;

        let arrendador = leer_arrendador(&env)?;
        let inquilino = leer_inquilino(&env)?;
        arrendador.require_auth();
        inquilino.require_auth();

        let monto = leer_monto(&env)?;
        if monto_arrendador < 0 || monto_arrendador > monto {
            return Err(Error::MontoInvalido);
        }

        let token = leer_token(&env)?;
        pagar(&env, &token, &arrendador, monto_arrendador);
        pagar(&env, &token, &inquilino, monto - monto_arrendador);

        env.storage()
            .instance()
            .set(&DataKey::Estado, &Estado::Repartido);
        extender(&env);
        Ok(())
    }

    pub fn arrendador(env: Env) -> Result<Address, Error> {
        leer_arrendador(&env)
    }

    pub fn inquilino(env: Env) -> Result<Address, Error> {
        leer_inquilino(&env)
    }

    pub fn token(env: Env) -> Result<Address, Error> {
        leer_token(&env)
    }

    pub fn monto(env: Env) -> Result<i128, Error> {
        leer_monto(&env)
    }

    pub fn estado(env: Env) -> Result<Estado, Error> {
        leer_estado(&env)
    }

    pub fn saldo(env: Env) -> Result<i128, Error> {
        let token = leer_token(&env)?;
        Ok(token::Client::new(&env, &token).balance(&env.current_contract_address()))
    }
}

fn leer_arrendador(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Arrendador)
        .ok_or(Error::NoIniciado)
}

fn leer_inquilino(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Inquilino)
        .ok_or(Error::NoIniciado)
}

fn leer_token(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Token)
        .ok_or(Error::NoIniciado)
}

fn leer_monto(env: &Env) -> Result<i128, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Monto)
        .ok_or(Error::NoIniciado)
}

fn leer_estado(env: &Env) -> Result<Estado, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Estado)
        .ok_or(Error::NoIniciado)
}

fn exigir_estado(env: &Env, esperado: Estado) -> Result<(), Error> {
    if leer_estado(env)? != esperado {
        return Err(Error::EstadoIncorrecto);
    }
    Ok(())
}

fn pagar(env: &Env, token: &Address, destino: &Address, monto: i128) {
    if monto == 0 {
        return;
    }
    token::Client::new(env, token).transfer(&env.current_contract_address(), destino, &monto);
}

fn extender(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(EXTENSION_UMBRAL, EXTENSION_CANTIDAD);
}

mod test;
