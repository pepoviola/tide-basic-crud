--
-- PostgreSQL database dump
--

-- Dumped from database version 9.6.19
-- Dumped by pg_dump version 9.6.2

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET client_min_messages = warning;
SET row_security = off;


--
-- Name: EXTENSION plpgsql; Type: COMMENT; Schema: -; Owner:
--

COMMENT ON EXTENSION plpgsql IS 'PL/pgSQL procedural language';


SET search_path = public, pg_catalog;

SET default_tablespace = '';

SET default_with_oids = false;

--
-- Name: dinos; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE dinos (
    id uuid NOT NULL,
    name text NOT NULL,
    weight integer NOT NULL,
    diet text NOT NULL,
    user_id text
);


ALTER TABLE dinos OWNER TO postgres;

--
-- Name: dinos dinos_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY dinos
    ADD CONSTRAINT dinos_pkey PRIMARY KEY (id);


--
-- PostgreSQL database dump complete
--

