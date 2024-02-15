-- Database Schema
CREATE TABLE IF NOT EXISTS clientes (
    id SERIAL PRIMARY KEY,
    limite INTEGER NOT NULL,
    nome VARCHAR(256),
    saldo INTEGER NOT NULL DEFAULT 0
);
CREATE TYPE tipoTransacao as ENUM ('c', 'd');
CREATE TABLE IF NOT EXISTS transacoes (
    id SERIAL PRIMARY KEY,
    cliente_id INTEGER NOT NULL REFERENCES clientes(id),
    tipo tipoTransacao NOT NULL,
    valor INTEGER NOT NULL,
    descricao VARCHAR(1024),
    realizada_em TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX CONCURRENTLY transacoes_realizada_em_idx ON transacoes(cliente_id, realizada_em DESC);

-- Initial Data
INSERT INTO clientes (nome, limite)
VALUES
('o barato sai caro', 1000 * 100),
('zan corp ltda', 800 * 100),
('les cruders', 10000 * 100),
('padaria joia de cocaia', 100000 * 100),
('kid mais', 5000 * 100);
