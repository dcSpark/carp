--
-- PostgreSQL database dump
--

-- Dumped from database version 12.9 (Ubuntu 12.9-0ubuntu0.20.04.1)
-- Dumped by pg_dump version 12.9 (Ubuntu 12.9-0ubuntu0.20.04.1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: Address; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."Address" (
    id bigint NOT NULL,
    payload bytea NOT NULL
);


--
-- Name: AddressCredentialRelation; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."AddressCredentialRelation" (
    address_id bigint NOT NULL,
    credential_id bigint NOT NULL,
    relation integer NOT NULL
);


--
-- Name: Address_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."Address_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: Address_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."Address_id_seq" OWNED BY public."Address".id;


--
-- Name: Block; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."Block" (
    id integer NOT NULL,
    era integer NOT NULL,
    hash bytea NOT NULL,
    height integer NOT NULL,
    epoch integer NOT NULL,
    slot integer NOT NULL
);


--
-- Name: Block_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."Block_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: Block_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."Block_id_seq" OWNED BY public."Block".id;


--
-- Name: StakeCredential; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."StakeCredential" (
    id bigint NOT NULL,
    credential bytea NOT NULL
);


--
-- Name: StakeCredential_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."StakeCredential_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: StakeCredential_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."StakeCredential_id_seq" OWNED BY public."StakeCredential".id;


--
-- Name: Transaction; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."Transaction" (
    id bigint NOT NULL,
    hash bytea NOT NULL,
    block_id integer NOT NULL,
    tx_index integer NOT NULL,
    payload bytea NOT NULL,
    is_valid boolean NOT NULL
);


--
-- Name: TransactionInput; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."TransactionInput" (
    id bigint NOT NULL,
    utxo_id bigint NOT NULL,
    tx_id bigint NOT NULL,
    input_index integer NOT NULL
);


--
-- Name: TransactionInput_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."TransactionInput_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: TransactionInput_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."TransactionInput_id_seq" OWNED BY public."TransactionInput".id;


--
-- Name: TransactionOutput; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."TransactionOutput" (
    id bigint NOT NULL,
    payload bytea NOT NULL,
    address_id bigint NOT NULL,
    tx_id bigint NOT NULL,
    output_index integer NOT NULL
);


--
-- Name: TransactionOutput_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."TransactionOutput_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: TransactionOutput_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."TransactionOutput_id_seq" OWNED BY public."TransactionOutput".id;


--
-- Name: Transaction_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."Transaction_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: Transaction_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."Transaction_id_seq" OWNED BY public."Transaction".id;


--
-- Name: TxCredentialRelation; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."TxCredentialRelation" (
    credential_id bigint NOT NULL,
    tx_id bigint NOT NULL,
    relation integer NOT NULL
);


--
-- Name: seaql_migrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.seaql_migrations (
    version character varying NOT NULL,
    applied_at bigint NOT NULL
);


--
-- Name: Address id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Address" ALTER COLUMN id SET DEFAULT nextval('public."Address_id_seq"'::regclass);


--
-- Name: Block id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Block" ALTER COLUMN id SET DEFAULT nextval('public."Block_id_seq"'::regclass);


--
-- Name: StakeCredential id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeCredential" ALTER COLUMN id SET DEFAULT nextval('public."StakeCredential_id_seq"'::regclass);


--
-- Name: Transaction id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Transaction" ALTER COLUMN id SET DEFAULT nextval('public."Transaction_id_seq"'::regclass);


--
-- Name: TransactionInput id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionInput" ALTER COLUMN id SET DEFAULT nextval('public."TransactionInput_id_seq"'::regclass);


--
-- Name: TransactionOutput id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionOutput" ALTER COLUMN id SET DEFAULT nextval('public."TransactionOutput_id_seq"'::regclass);


--
-- Name: Address Address_payload_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Address"
    ADD CONSTRAINT "Address_payload_key" UNIQUE (payload);


--
-- Name: Address Address_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Address"
    ADD CONSTRAINT "Address_pkey" PRIMARY KEY (id);


--
-- Name: Block Block_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Block"
    ADD CONSTRAINT "Block_pkey" PRIMARY KEY (id);


--
-- Name: StakeCredential StakeCredential_credential_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeCredential"
    ADD CONSTRAINT "StakeCredential_credential_key" UNIQUE (credential);


--
-- Name: StakeCredential StakeCredential_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeCredential"
    ADD CONSTRAINT "StakeCredential_pkey" PRIMARY KEY (id);


--
-- Name: TransactionInput TransactionInput_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionInput"
    ADD CONSTRAINT "TransactionInput_pkey" PRIMARY KEY (id);


--
-- Name: TransactionOutput TransactionOutput_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionOutput"
    ADD CONSTRAINT "TransactionOutput_pkey" PRIMARY KEY (id);


--
-- Name: Transaction Transaction_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Transaction"
    ADD CONSTRAINT "Transaction_hash_key" UNIQUE (hash);


--
-- Name: Transaction Transaction_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Transaction"
    ADD CONSTRAINT "Transaction_pkey" PRIMARY KEY (id);


--
-- Name: AddressCredentialRelation address_credential-pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AddressCredentialRelation"
    ADD CONSTRAINT "address_credential-pk" PRIMARY KEY (address_id, credential_id);


--
-- Name: seaql_migrations seaql_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.seaql_migrations
    ADD CONSTRAINT seaql_migrations_pkey PRIMARY KEY (version);


--
-- Name: TxCredentialRelation tx_credential-pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TxCredentialRelation"
    ADD CONSTRAINT "tx_credential-pk" PRIMARY KEY (tx_id, credential_id);


--
-- Name: index-address_credential-address; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX "index-address_credential-address" ON public."AddressCredentialRelation" USING btree (address_id, credential_id, relation);


--
-- Name: index-transaction-block; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction-block" ON public."Transaction" USING btree (block_id);


--
-- Name: index-transaction_input-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction_input-transaction" ON public."TransactionInput" USING btree (tx_id);


--
-- Name: index-transaction_input-transaction_output; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction_input-transaction_output" ON public."TransactionInput" USING btree (utxo_id);


--
-- Name: index-transaction_output-address; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction_output-address" ON public."TransactionOutput" USING btree (address_id);


--
-- Name: index-transaction_output-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction_output-transaction" ON public."TransactionOutput" USING btree (tx_id);


--
-- Name: AddressCredentialRelation fk-address_credential-address_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AddressCredentialRelation"
    ADD CONSTRAINT "fk-address_credential-address_id" FOREIGN KEY (address_id) REFERENCES public."Address"(id);


--
-- Name: AddressCredentialRelation fk-address_credential-credential_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AddressCredentialRelation"
    ADD CONSTRAINT "fk-address_credential-credential_id" FOREIGN KEY (credential_id) REFERENCES public."StakeCredential"(id);


--
-- Name: Transaction fk-transaction-block_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Transaction"
    ADD CONSTRAINT "fk-transaction-block_id" FOREIGN KEY (block_id) REFERENCES public."Block"(id) ON DELETE CASCADE;


--
-- Name: TransactionInput fk-transaction_input-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionInput"
    ADD CONSTRAINT "fk-transaction_input-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: TransactionInput fk-transaction_input-utxo_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionInput"
    ADD CONSTRAINT "fk-transaction_input-utxo_id" FOREIGN KEY (utxo_id) REFERENCES public."TransactionOutput"(id);


--
-- Name: TransactionOutput fk-transaction_output-address_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionOutput"
    ADD CONSTRAINT "fk-transaction_output-address_id" FOREIGN KEY (address_id) REFERENCES public."Address"(id);


--
-- Name: TransactionOutput fk-transaction_output-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionOutput"
    ADD CONSTRAINT "fk-transaction_output-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: TxCredentialRelation fk-tx_credential-credential_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TxCredentialRelation"
    ADD CONSTRAINT "fk-tx_credential-credential_id" FOREIGN KEY (credential_id) REFERENCES public."StakeCredential"(id);


--
-- Name: TxCredentialRelation fk-tx_credential-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TxCredentialRelation"
    ADD CONSTRAINT "fk-tx_credential-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

