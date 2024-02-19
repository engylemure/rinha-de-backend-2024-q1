CREATE UNLOGGED TABLE IF NOT EXISTS clientes (
    id SERIAL PRIMARY KEY,
    limite INTEGER NOT NULL,
    nome VARCHAR(100),
    saldo INTEGER NOT NULL DEFAULT 0
);
CREATE TYPE TIPO_TRANSACAO as ENUM ('c', 'd');
CREATE UNLOGGED TABLE IF NOT EXISTS transacoes (
    id SERIAL PRIMARY KEY,
    cliente_id INTEGER NOT NULL REFERENCES clientes(id),
    tipo TIPO_TRANSACAO NOT NULL,
    valor INTEGER NOT NULL,
    descricao VARCHAR(25),
    realizada_em TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX CONCURRENTLY transacoes_realizada_em_idx ON transacoes(cliente_id, realizada_em DESC);

INSERT INTO clientes (nome, limite)
VALUES
('o barato sai caro', 1000 * 100),
('zan corp ltda', 800 * 100),
('les cruders', 10000 * 100),
('padaria joia de cocaia', 100000 * 100),
('kid mais', 5000 * 100);

CREATE OR REPLACE FUNCTION criarTransacao(
    in cliente_id INT,
    in valor INT,
    in tipo TIPO_TRANSACAO,
    in descricao VARCHAR(10),
    out res int,
    out new_saldo int,
    out limite int
)
LANGUAGE plpgsql
AS $$
BEGIN
    IF NOT EXISTS (
                SELECT
                    c.saldo, c.limite
                FROM clientes as c
                WHERE c.id = cliente_id
                FOR UPdATE
            ) THEN
        SELECT -1, 0, 0 INTO res, new_saldo, limite;
        RETURN;
    ELSE 
        SELECT c.saldo, c.limite INTO new_saldo, limite FROM clientes as c WHERE c.id = cliente_id;
    END IF;
    IF tipo = 'd' THEN
        IF new_saldo + limite < valor THEN 
            SELECT -2, 0, 0 INTO res, new_saldo, limite;
        ELSE 
            UPDATE clientes SET saldo = saldo - valor WHERE id = cliente_id;
            SELECT 0, c.saldo, c.limite INTO res, new_saldo, limite FROM clientes as c WHERE c.id = cliente_id;
        END IF;
    ELSE 
        UPDATE clientes SET saldo = saldo + valor WHERE id = cliente_id;
        SELECT 0, c.saldo, c.limite INTO res, new_saldo, limite FROM clientes as c WHERE c.id = cliente_id;
    END IF;
    INSERT INTO transacoes(cliente_id, valor, tipo, descricao) VALUES (cliente_id, valor, tipo, descricao);
    RETURN;
END;
$$
