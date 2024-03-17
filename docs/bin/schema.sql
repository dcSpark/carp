--
-- PostgreSQL database dump
--

-- Dumped from database version 14.9 (Ubuntu 14.9-0ubuntu0.22.04.1)
-- Dumped by pg_dump version 14.9 (Ubuntu 14.9-0ubuntu0.22.04.1)

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
    payload bytea NOT NULL,
    first_tx bigint NOT NULL
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
-- Name: AssetMint; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."AssetMint" (
    tx_id bigint NOT NULL,
    asset_id bigint NOT NULL,
    amount bigint NOT NULL
);


--
-- Name: AssetUtxo; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."AssetUtxo" (
    id bigint NOT NULL,
    asset_id bigint NOT NULL,
    utxo_id bigint NOT NULL,
    tx_id bigint NOT NULL,
    amount bigint
);


--
-- Name: AssetUtxo_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."AssetUtxo_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: AssetUtxo_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."AssetUtxo_id_seq" OWNED BY public."AssetUtxo".id;


--
-- Name: Block; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."Block" (
    id integer NOT NULL,
    era integer NOT NULL,
    hash bytea NOT NULL,
    height integer NOT NULL,
    epoch integer NOT NULL,
    slot integer NOT NULL,
    payload bytea,
    tx_count integer
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
-- Name: Cip25Entry; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."Cip25Entry" (
    id bigint NOT NULL,
    metadata_id bigint NOT NULL,
    asset_id bigint NOT NULL,
    version text NOT NULL,
    payload bytea NOT NULL
);


--
-- Name: Cip25Entry_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."Cip25Entry_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: Cip25Entry_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."Cip25Entry_id_seq" OWNED BY public."Cip25Entry".id;


--
-- Name: Dex; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."Dex" (
    id bigint NOT NULL,
    tx_id bigint NOT NULL,
    address_id bigint NOT NULL,
    dex bigint NOT NULL,
    asset1_id bigint,
    asset2_id bigint,
    amount1 bigint NOT NULL,
    amount2 bigint NOT NULL,
    operation bigint NOT NULL
);


--
-- Name: Dex_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."Dex_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: Dex_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."Dex_id_seq" OWNED BY public."Dex".id;


--
-- Name: NativeAsset; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."NativeAsset" (
    id bigint NOT NULL,
    policy_id bytea NOT NULL,
    asset_name bytea NOT NULL,
    cip14_fingerprint bytea NOT NULL,
    first_tx bigint NOT NULL
);


--
-- Name: NativeAsset_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."NativeAsset_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: NativeAsset_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."NativeAsset_id_seq" OWNED BY public."NativeAsset".id;


--
-- Name: PlutusData; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."PlutusData" (
    id bigint NOT NULL,
    data bytea NOT NULL
);


--
-- Name: PlutusDataHash; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."PlutusDataHash" (
    id bigint NOT NULL,
    hash bytea NOT NULL,
    first_tx bigint NOT NULL
);


--
-- Name: PlutusDataHash_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."PlutusDataHash_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: PlutusDataHash_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."PlutusDataHash_id_seq" OWNED BY public."PlutusDataHash".id;


--
-- Name: PlutusData_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."PlutusData_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: PlutusData_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."PlutusData_id_seq" OWNED BY public."PlutusData".id;


--
-- Name: ProjectedNFT; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."ProjectedNFT" (
    id bigint NOT NULL,
    owner_address bytea NOT NULL,
    previous_utxo_tx_hash bytea NOT NULL,
    previous_utxo_tx_output_index bigint,
    hololocker_utxo_id bigint,
    tx_id bigint NOT NULL,
    asset_name text NOT NULL,
    policy_id text NOT NULL,
    amount bigint NOT NULL,
    operation integer NOT NULL,
    plutus_datum bytea NOT NULL,
    for_how_long bigint
);


--
-- Name: ProjectedNFT_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."ProjectedNFT_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ProjectedNFT_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."ProjectedNFT_id_seq" OWNED BY public."ProjectedNFT".id;


--
-- Name: StakeCredential; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."StakeCredential" (
    id bigint NOT NULL,
    credential bytea NOT NULL,
    first_tx bigint NOT NULL
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
-- Name: StakeDelegationCredentialRelation; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."StakeDelegationCredentialRelation" (
    id bigint NOT NULL,
    stake_credential bigint NOT NULL,
    tx_id bigint NOT NULL,
    pool_credential bytea,
    previous_pool bytea
);


--
-- Name: StakeDelegationCredentialRelation_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."StakeDelegationCredentialRelation_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: StakeDelegationCredentialRelation_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."StakeDelegationCredentialRelation_id_seq" OWNED BY public."StakeDelegationCredentialRelation".id;


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
    address_id bigint NOT NULL,
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
-- Name: TransactionMetadata; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."TransactionMetadata" (
    id bigint NOT NULL,
    tx_id bigint NOT NULL,
    label bytea NOT NULL,
    payload bytea NOT NULL
);


--
-- Name: TransactionMetadata_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."TransactionMetadata_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: TransactionMetadata_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."TransactionMetadata_id_seq" OWNED BY public."TransactionMetadata".id;


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
-- Name: TransactionReferenceInput; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."TransactionReferenceInput" (
    id bigint NOT NULL,
    utxo_id bigint NOT NULL,
    tx_id bigint NOT NULL,
    address_id bigint NOT NULL,
    input_index integer NOT NULL
);


--
-- Name: TransactionReferenceInput_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."TransactionReferenceInput_id_seq"
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: TransactionReferenceInput_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."TransactionReferenceInput_id_seq" OWNED BY public."TransactionReferenceInput".id;


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
-- Name: AssetUtxo id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AssetUtxo" ALTER COLUMN id SET DEFAULT nextval('public."AssetUtxo_id_seq"'::regclass);


--
-- Name: Block id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Block" ALTER COLUMN id SET DEFAULT nextval('public."Block_id_seq"'::regclass);


--
-- Name: Cip25Entry id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Cip25Entry" ALTER COLUMN id SET DEFAULT nextval('public."Cip25Entry_id_seq"'::regclass);


--
-- Name: Dex id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Dex" ALTER COLUMN id SET DEFAULT nextval('public."Dex_id_seq"'::regclass);


--
-- Name: NativeAsset id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."NativeAsset" ALTER COLUMN id SET DEFAULT nextval('public."NativeAsset_id_seq"'::regclass);


--
-- Name: PlutusData id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."PlutusData" ALTER COLUMN id SET DEFAULT nextval('public."PlutusData_id_seq"'::regclass);


--
-- Name: PlutusDataHash id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."PlutusDataHash" ALTER COLUMN id SET DEFAULT nextval('public."PlutusDataHash_id_seq"'::regclass);


--
-- Name: ProjectedNFT id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."ProjectedNFT" ALTER COLUMN id SET DEFAULT nextval('public."ProjectedNFT_id_seq"'::regclass);


--
-- Name: StakeCredential id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeCredential" ALTER COLUMN id SET DEFAULT nextval('public."StakeCredential_id_seq"'::regclass);


--
-- Name: StakeDelegationCredentialRelation id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeDelegationCredentialRelation" ALTER COLUMN id SET DEFAULT nextval('public."StakeDelegationCredentialRelation_id_seq"'::regclass);


--
-- Name: Transaction id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Transaction" ALTER COLUMN id SET DEFAULT nextval('public."Transaction_id_seq"'::regclass);


--
-- Name: TransactionInput id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionInput" ALTER COLUMN id SET DEFAULT nextval('public."TransactionInput_id_seq"'::regclass);


--
-- Name: TransactionMetadata id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionMetadata" ALTER COLUMN id SET DEFAULT nextval('public."TransactionMetadata_id_seq"'::regclass);


--
-- Name: TransactionOutput id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionOutput" ALTER COLUMN id SET DEFAULT nextval('public."TransactionOutput_id_seq"'::regclass);


--
-- Name: TransactionReferenceInput id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionReferenceInput" ALTER COLUMN id SET DEFAULT nextval('public."TransactionReferenceInput_id_seq"'::regclass);


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
-- Name: Block Block_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Block"
    ADD CONSTRAINT "Block_hash_key" UNIQUE (hash);


--
-- Name: Block Block_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Block"
    ADD CONSTRAINT "Block_pkey" PRIMARY KEY (id);


--
-- Name: Cip25Entry Cip25Entry_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Cip25Entry"
    ADD CONSTRAINT "Cip25Entry_pkey" PRIMARY KEY (id);


--
-- Name: Dex Dex_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Dex"
    ADD CONSTRAINT "Dex_pkey" PRIMARY KEY (id);


--
-- Name: NativeAsset NativeAsset_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."NativeAsset"
    ADD CONSTRAINT "NativeAsset_pkey" PRIMARY KEY (id);


--
-- Name: PlutusDataHash PlutusDataHash_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."PlutusDataHash"
    ADD CONSTRAINT "PlutusDataHash_pkey" PRIMARY KEY (id);


--
-- Name: PlutusData PlutusData_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."PlutusData"
    ADD CONSTRAINT "PlutusData_pkey" PRIMARY KEY (id);


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
-- Name: TransactionMetadata TransactionMetadata_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionMetadata"
    ADD CONSTRAINT "TransactionMetadata_pkey" PRIMARY KEY (id);


--
-- Name: TransactionOutput TransactionOutput_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionOutput"
    ADD CONSTRAINT "TransactionOutput_pkey" PRIMARY KEY (id);


--
-- Name: TransactionReferenceInput TransactionReferenceInput_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionReferenceInput"
    ADD CONSTRAINT "TransactionReferenceInput_pkey" PRIMARY KEY (id);


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
    ADD CONSTRAINT "address_credential-pk" PRIMARY KEY (address_id, credential_id, relation);


--
-- Name: AssetMint asset_mint-pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AssetMint"
    ADD CONSTRAINT "asset_mint-pk" PRIMARY KEY (tx_id, asset_id);


--
-- Name: AssetUtxo asset_utxo-pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AssetUtxo"
    ADD CONSTRAINT "asset_utxo-pk" PRIMARY KEY (id);


--
-- Name: ProjectedNFT projected_nft-pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."ProjectedNFT"
    ADD CONSTRAINT "projected_nft-pk" PRIMARY KEY (id);


--
-- Name: seaql_migrations seaql_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.seaql_migrations
    ADD CONSTRAINT seaql_migrations_pkey PRIMARY KEY (version);


--
-- Name: StakeDelegationCredentialRelation stake_delegation_credential-pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeDelegationCredentialRelation"
    ADD CONSTRAINT "stake_delegation_credential-pk" PRIMARY KEY (id);


--
-- Name: TxCredentialRelation tx_credential-pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TxCredentialRelation"
    ADD CONSTRAINT "tx_credential-pk" PRIMARY KEY (tx_id, credential_id);


--
-- Name: index-address-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-address-transaction" ON public."Address" USING btree (first_tx);


--
-- Name: index-address_credential-credential; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-address_credential-credential" ON public."AddressCredentialRelation" USING btree (credential_id);


--
-- Name: index-asset_mint-native_asset; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-asset_mint-native_asset" ON public."AssetMint" USING btree (asset_id);


--
-- Name: index-asset_utxo-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-asset_utxo-transaction" ON public."AssetUtxo" USING btree (tx_id);


--
-- Name: index-cip25_entry-metadata; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-cip25_entry-metadata" ON public."Cip25Entry" USING btree (metadata_id);


--
-- Name: index-cip25_entry-native_asset; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-cip25_entry-native_asset" ON public."Cip25Entry" USING btree (asset_id);


--
-- Name: index-dex-address-native_asset1-native_asset2-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-dex-address-native_asset1-native_asset2-transaction" ON public."Dex" USING btree (dex, asset1_id, asset2_id, tx_id);


--
-- Name: index-dex-operation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-dex-operation" ON public."Dex" USING btree (operation);


--
-- Name: index-metadata-label; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-metadata-label" ON public."TransactionMetadata" USING btree (label);


--
-- Name: index-metadata-txid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-metadata-txid" ON public."TransactionMetadata" USING btree (tx_id);


--
-- Name: index-native_asset-fingerprint; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-native_asset-fingerprint" ON public."NativeAsset" USING btree (cip14_fingerprint);


--
-- Name: index-native_asset-pair; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX "index-native_asset-pair" ON public."NativeAsset" USING btree (policy_id, asset_name);


--
-- Name: index-native_asset-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-native_asset-transaction" ON public."NativeAsset" USING btree (first_tx);


--
-- Name: index-native_asset_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-native_asset_name" ON public."NativeAsset" USING btree (asset_name);


--
-- Name: index-plutus_data_hash-hash; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX "index-plutus_data_hash-hash" ON public."PlutusDataHash" USING btree (hash);


--
-- Name: index-plutus_data_hash-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-plutus_data_hash-transaction" ON public."PlutusDataHash" USING btree (first_tx);


--
-- Name: index-stake_credential-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-stake_credential-transaction" ON public."StakeCredential" USING btree (first_tx);


--
-- Name: index-stake_delegation_credential-stake_credential; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-stake_delegation_credential-stake_credential" ON public."StakeDelegationCredentialRelation" USING btree (stake_credential);


--
-- Name: index-transaction-block; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction-block" ON public."Transaction" USING btree (block_id);


--
-- Name: index-transaction_input-address; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction_input-address" ON public."TransactionInput" USING btree (address_id);


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
-- Name: index-transaction_reference_input-address; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction_reference_input-address" ON public."TransactionReferenceInput" USING btree (address_id);


--
-- Name: index-transaction_reference_input-transaction; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction_reference_input-transaction" ON public."TransactionReferenceInput" USING btree (tx_id);


--
-- Name: index-transaction_reference_input-transaction_output; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-transaction_reference_input-transaction_output" ON public."TransactionReferenceInput" USING btree (utxo_id);


--
-- Name: index-tx_credential-credential; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX "index-tx_credential-credential" ON public."TxCredentialRelation" USING btree (credential_id);


--
-- Name: Address fk-address-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Address"
    ADD CONSTRAINT "fk-address-tx_id" FOREIGN KEY (first_tx) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: AddressCredentialRelation fk-address_credential-address_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AddressCredentialRelation"
    ADD CONSTRAINT "fk-address_credential-address_id" FOREIGN KEY (address_id) REFERENCES public."Address"(id) ON DELETE CASCADE;


--
-- Name: AddressCredentialRelation fk-address_credential-credential_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AddressCredentialRelation"
    ADD CONSTRAINT "fk-address_credential-credential_id" FOREIGN KEY (credential_id) REFERENCES public."StakeCredential"(id) ON DELETE CASCADE;


--
-- Name: AssetMint fk-asset_mint-asset_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AssetMint"
    ADD CONSTRAINT "fk-asset_mint-asset_id" FOREIGN KEY (asset_id) REFERENCES public."NativeAsset"(id) ON DELETE CASCADE;


--
-- Name: AssetMint fk-asset_mint-transaction_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AssetMint"
    ADD CONSTRAINT "fk-asset_mint-transaction_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: AssetUtxo fk-asset_utxo-asset_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AssetUtxo"
    ADD CONSTRAINT "fk-asset_utxo-asset_id" FOREIGN KEY (asset_id) REFERENCES public."NativeAsset"(id) ON DELETE CASCADE;


--
-- Name: AssetUtxo fk-asset_utxo-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AssetUtxo"
    ADD CONSTRAINT "fk-asset_utxo-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: AssetUtxo fk-asset_utxo-utxo_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."AssetUtxo"
    ADD CONSTRAINT "fk-asset_utxo-utxo_id" FOREIGN KEY (utxo_id) REFERENCES public."TransactionOutput"(id) ON DELETE CASCADE;


--
-- Name: Cip25Entry fk-cip25_entry-asset_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Cip25Entry"
    ADD CONSTRAINT "fk-cip25_entry-asset_id" FOREIGN KEY (asset_id) REFERENCES public."NativeAsset"(id) ON DELETE CASCADE;


--
-- Name: Cip25Entry fk-cip25_entry-metadata; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Cip25Entry"
    ADD CONSTRAINT "fk-cip25_entry-metadata" FOREIGN KEY (metadata_id) REFERENCES public."TransactionMetadata"(id) ON DELETE CASCADE;


--
-- Name: Dex fk-dex-address_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Dex"
    ADD CONSTRAINT "fk-dex-address_id" FOREIGN KEY (address_id) REFERENCES public."Address"(id) ON DELETE CASCADE;


--
-- Name: Dex fk-dex-asset1_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Dex"
    ADD CONSTRAINT "fk-dex-asset1_id" FOREIGN KEY (asset1_id) REFERENCES public."NativeAsset"(id) ON DELETE CASCADE;


--
-- Name: Dex fk-dex-asset2_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Dex"
    ADD CONSTRAINT "fk-dex-asset2_id" FOREIGN KEY (asset2_id) REFERENCES public."NativeAsset"(id) ON DELETE CASCADE;


--
-- Name: Dex fk-dex-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Dex"
    ADD CONSTRAINT "fk-dex-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: TransactionMetadata fk-metadata-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionMetadata"
    ADD CONSTRAINT "fk-metadata-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: NativeAsset fk-native_asset-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."NativeAsset"
    ADD CONSTRAINT "fk-native_asset-tx_id" FOREIGN KEY (first_tx) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: PlutusData fk-plutus_data-plutus_data_hash-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."PlutusData"
    ADD CONSTRAINT "fk-plutus_data-plutus_data_hash-tx_id" FOREIGN KEY (id) REFERENCES public."PlutusDataHash"(id) ON DELETE CASCADE;


--
-- Name: PlutusDataHash fk-plutus_data_hash-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."PlutusDataHash"
    ADD CONSTRAINT "fk-plutus_data_hash-tx_id" FOREIGN KEY (first_tx) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: ProjectedNFT fk-projected_nft-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."ProjectedNFT"
    ADD CONSTRAINT "fk-projected_nft-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: ProjectedNFT fk-projected_nft-utxo_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."ProjectedNFT"
    ADD CONSTRAINT "fk-projected_nft-utxo_id" FOREIGN KEY (hololocker_utxo_id) REFERENCES public."TransactionOutput"(id) ON DELETE CASCADE;


--
-- Name: StakeCredential fk-stake_credential-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeCredential"
    ADD CONSTRAINT "fk-stake_credential-tx_id" FOREIGN KEY (first_tx) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: StakeDelegationCredentialRelation fk-stake_delegation-credential_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeDelegationCredentialRelation"
    ADD CONSTRAINT "fk-stake_delegation-credential_id" FOREIGN KEY (stake_credential) REFERENCES public."StakeCredential"(id) ON DELETE CASCADE;


--
-- Name: StakeDelegationCredentialRelation fk-stake_delegation-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."StakeDelegationCredentialRelation"
    ADD CONSTRAINT "fk-stake_delegation-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: Transaction fk-transaction-block_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."Transaction"
    ADD CONSTRAINT "fk-transaction-block_id" FOREIGN KEY (block_id) REFERENCES public."Block"(id) ON DELETE CASCADE;


--
-- Name: TransactionInput fk-transaction_input-address_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionInput"
    ADD CONSTRAINT "fk-transaction_input-address_id" FOREIGN KEY (address_id) REFERENCES public."Address"(id) ON DELETE CASCADE;


--
-- Name: TransactionInput fk-transaction_input-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionInput"
    ADD CONSTRAINT "fk-transaction_input-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: TransactionInput fk-transaction_input-utxo_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionInput"
    ADD CONSTRAINT "fk-transaction_input-utxo_id" FOREIGN KEY (utxo_id) REFERENCES public."TransactionOutput"(id) ON DELETE CASCADE;


--
-- Name: TransactionOutput fk-transaction_output-address_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionOutput"
    ADD CONSTRAINT "fk-transaction_output-address_id" FOREIGN KEY (address_id) REFERENCES public."Address"(id) ON DELETE CASCADE;


--
-- Name: TransactionOutput fk-transaction_output-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionOutput"
    ADD CONSTRAINT "fk-transaction_output-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: TransactionReferenceInput fk-transaction_reference-input-address_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionReferenceInput"
    ADD CONSTRAINT "fk-transaction_reference-input-address_id" FOREIGN KEY (address_id) REFERENCES public."Address"(id) ON DELETE CASCADE;


--
-- Name: TransactionReferenceInput fk-transaction_reference-input-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionReferenceInput"
    ADD CONSTRAINT "fk-transaction_reference-input-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- Name: TransactionReferenceInput fk-transaction_reference-input-utxo_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TransactionReferenceInput"
    ADD CONSTRAINT "fk-transaction_reference-input-utxo_id" FOREIGN KEY (utxo_id) REFERENCES public."TransactionOutput"(id) ON DELETE CASCADE;


--
-- Name: TxCredentialRelation fk-tx_credential-credential_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TxCredentialRelation"
    ADD CONSTRAINT "fk-tx_credential-credential_id" FOREIGN KEY (credential_id) REFERENCES public."StakeCredential"(id) ON DELETE CASCADE;


--
-- Name: TxCredentialRelation fk-tx_credential-tx_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."TxCredentialRelation"
    ADD CONSTRAINT "fk-tx_credential-tx_id" FOREIGN KEY (tx_id) REFERENCES public."Transaction"(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

