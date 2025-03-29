--
-- PostgreSQL database dump
--

-- Dumped from database version 15.6
-- Dumped by pg_dump version 15.6

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

--
-- Name: antenna_src_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.antenna_src_enum AS ENUM (
    'home',
    'all',
    'users',
    'list',
    'users_blacklist'
);


ALTER TYPE public.antenna_src_enum OWNER TO "example-misskey-user";

--
-- Name: log_level_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.log_level_enum AS ENUM (
    'error',
    'warning',
    'info',
    'success',
    'debug'
);


ALTER TYPE public.log_level_enum OWNER TO "example-misskey-user";

--
-- Name: meta_sensitivemediadetection_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.meta_sensitivemediadetection_enum AS ENUM (
    'none',
    'all',
    'local',
    'remote'
);


ALTER TYPE public.meta_sensitivemediadetection_enum OWNER TO "example-misskey-user";

--
-- Name: meta_sensitivemediadetectionsensitivity_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.meta_sensitivemediadetectionsensitivity_enum AS ENUM (
    'medium',
    'low',
    'high',
    'veryLow',
    'veryHigh'
);


ALTER TYPE public.meta_sensitivemediadetectionsensitivity_enum OWNER TO "example-misskey-user";

--
-- Name: muted_note_reason_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.muted_note_reason_enum AS ENUM (
    'word',
    'manual',
    'spam',
    'other'
);


ALTER TYPE public.muted_note_reason_enum OWNER TO "example-misskey-user";

--
-- Name: note_visibility_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.note_visibility_enum AS ENUM (
    'public',
    'home',
    'followers',
    'specified'
);


ALTER TYPE public.note_visibility_enum OWNER TO "example-misskey-user";

--
-- Name: notification_type_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.notification_type_enum AS ENUM (
    'follow',
    'mention',
    'reply',
    'renote',
    'quote',
    'reaction',
    'pollVote',
    'pollEnded',
    'receiveFollowRequest',
    'followRequestAccepted',
    'groupInvited',
    'achievementEarned',
    'app'
);


ALTER TYPE public.notification_type_enum OWNER TO "example-misskey-user";

--
-- Name: page_visibility_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.page_visibility_enum AS ENUM (
    'public',
    'followers',
    'specified'
);


ALTER TYPE public.page_visibility_enum OWNER TO "example-misskey-user";

--
-- Name: poll_notevisibility_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.poll_notevisibility_enum AS ENUM (
    'public',
    'home',
    'followers',
    'specified'
);


ALTER TYPE public.poll_notevisibility_enum OWNER TO "example-misskey-user";

--
-- Name: relay_status_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.relay_status_enum AS ENUM (
    'requesting',
    'accepted',
    'rejected'
);


ALTER TYPE public.relay_status_enum OWNER TO "example-misskey-user";

--
-- Name: role_target_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.role_target_enum AS ENUM (
    'manual',
    'conditional'
);


ALTER TYPE public.role_target_enum OWNER TO "example-misskey-user";

--
-- Name: user_profile_followersVisibility_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public."user_profile_followersVisibility_enum" AS ENUM (
    'public',
    'followers',
    'private'
);


ALTER TYPE public."user_profile_followersVisibility_enum" OWNER TO "example-misskey-user";

--
-- Name: user_profile_followingvisibility_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.user_profile_followingvisibility_enum AS ENUM (
    'public',
    'followers',
    'private'
);


ALTER TYPE public.user_profile_followingvisibility_enum OWNER TO "example-misskey-user";

--
-- Name: user_profile_mutingnotificationtypes_enum; Type: TYPE; Schema: public; Owner: example-misskey-user
--

CREATE TYPE public.user_profile_mutingnotificationtypes_enum AS ENUM (
    'note',
    'follow',
    'mention',
    'reply',
    'renote',
    'quote',
    'reaction',
    'pollEnded',
    'receiveFollowRequest',
    'followRequestAccepted',
    'achievementEarned',
    'app',
    'test',
    'pollVote',
    'groupInvited'
);


ALTER TYPE public.user_profile_mutingnotificationtypes_enum OWNER TO "example-misskey-user";

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: __chart__active_users; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__active_users (
    id integer NOT NULL,
    date integer NOT NULL,
    "unique_temp___registeredWithinWeek" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredWithinWeek" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredWithinMonth" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredWithinMonth" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredWithinYear" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredWithinYear" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredOutsideWeek" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredOutsideWeek" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredOutsideMonth" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredOutsideMonth" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredOutsideYear" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredOutsideYear" integer DEFAULT '0'::smallint NOT NULL,
    "___readWrite" integer DEFAULT '0'::smallint NOT NULL,
    unique_temp___read character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___read integer DEFAULT '0'::smallint NOT NULL,
    unique_temp___write character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___write integer DEFAULT '0'::smallint NOT NULL
);


ALTER TABLE public.__chart__active_users OWNER TO "example-misskey-user";

--
-- Name: __chart__active_users_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__active_users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__active_users_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__active_users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__active_users_id_seq OWNED BY public.__chart__active_users.id;


--
-- Name: __chart__ap_request; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__ap_request (
    id integer NOT NULL,
    date integer NOT NULL,
    "___deliverFailed" integer DEFAULT 0 NOT NULL,
    "___deliverSucceeded" integer DEFAULT 0 NOT NULL,
    "___inboxReceived" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.__chart__ap_request OWNER TO "example-misskey-user";

--
-- Name: __chart__ap_request_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__ap_request_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__ap_request_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__ap_request_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__ap_request_id_seq OWNED BY public.__chart__ap_request.id;


--
-- Name: __chart__drive; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__drive (
    id integer NOT NULL,
    date integer NOT NULL,
    "___local_incCount" integer DEFAULT '0'::bigint NOT NULL,
    "___local_incSize" integer DEFAULT '0'::bigint NOT NULL,
    "___local_decCount" integer DEFAULT '0'::bigint NOT NULL,
    "___local_decSize" integer DEFAULT '0'::bigint NOT NULL,
    "___remote_incCount" integer DEFAULT '0'::bigint NOT NULL,
    "___remote_incSize" integer DEFAULT '0'::bigint NOT NULL,
    "___remote_decCount" integer DEFAULT '0'::bigint NOT NULL,
    "___remote_decSize" integer DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart__drive OWNER TO "example-misskey-user";

--
-- Name: __chart__drive_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__drive_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__drive_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__drive_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__drive_id_seq OWNED BY public.__chart__drive.id;


--
-- Name: __chart__federation; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__federation (
    id integer NOT NULL,
    date integer NOT NULL,
    "unique_temp___deliveredInstances" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___deliveredInstances" smallint DEFAULT '0'::smallint NOT NULL,
    "unique_temp___inboxInstances" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___inboxInstances" smallint DEFAULT '0'::smallint NOT NULL,
    unique_temp___stalled character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___stalled smallint DEFAULT '0'::smallint NOT NULL,
    ___sub smallint DEFAULT '0'::smallint NOT NULL,
    ___pub smallint DEFAULT '0'::smallint NOT NULL,
    ___pubsub smallint DEFAULT '0'::smallint NOT NULL,
    "___subActive" smallint DEFAULT '0'::smallint NOT NULL,
    "___pubActive" smallint DEFAULT '0'::smallint NOT NULL
);


ALTER TABLE public.__chart__federation OWNER TO "example-misskey-user";

--
-- Name: __chart__federation_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__federation_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__federation_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__federation_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__federation_id_seq OWNED BY public.__chart__federation.id;


--
-- Name: __chart__hashtag; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__hashtag (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___local_users integer DEFAULT 0 NOT NULL,
    ___remote_users integer DEFAULT 0 NOT NULL,
    unique_temp___local_users character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    unique_temp___remote_users character varying[] DEFAULT '{}'::character varying[] NOT NULL
);


ALTER TABLE public.__chart__hashtag OWNER TO "example-misskey-user";

--
-- Name: __chart__hashtag_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__hashtag_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__hashtag_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__hashtag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__hashtag_id_seq OWNED BY public.__chart__hashtag.id;


--
-- Name: __chart__instance; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__instance (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___requests_failed smallint DEFAULT '0'::bigint NOT NULL,
    ___requests_succeeded smallint DEFAULT '0'::bigint NOT NULL,
    ___requests_received smallint DEFAULT '0'::bigint NOT NULL,
    ___notes_total integer DEFAULT '0'::bigint NOT NULL,
    ___notes_inc integer DEFAULT '0'::bigint NOT NULL,
    ___notes_dec integer DEFAULT '0'::bigint NOT NULL,
    ___notes_diffs_normal integer DEFAULT '0'::bigint NOT NULL,
    ___notes_diffs_reply integer DEFAULT '0'::bigint NOT NULL,
    ___notes_diffs_renote integer DEFAULT '0'::bigint NOT NULL,
    ___users_total integer DEFAULT '0'::bigint NOT NULL,
    ___users_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___users_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___following_total integer DEFAULT '0'::bigint NOT NULL,
    ___following_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___following_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___followers_total integer DEFAULT '0'::bigint NOT NULL,
    ___followers_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___followers_dec smallint DEFAULT '0'::bigint NOT NULL,
    "___drive_totalFiles" integer DEFAULT '0'::bigint NOT NULL,
    "___drive_incFiles" integer DEFAULT '0'::bigint NOT NULL,
    "___drive_incUsage" integer DEFAULT '0'::bigint NOT NULL,
    "___drive_decFiles" integer DEFAULT '0'::bigint NOT NULL,
    "___drive_decUsage" integer DEFAULT '0'::bigint NOT NULL,
    "___notes_diffs_withFile" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.__chart__instance OWNER TO "example-misskey-user";

--
-- Name: __chart__instance_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__instance_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__instance_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__instance_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__instance_id_seq OWNED BY public.__chart__instance.id;


--
-- Name: __chart__network; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__network (
    id integer NOT NULL,
    date integer NOT NULL,
    "___incomingRequests" integer DEFAULT '0'::bigint NOT NULL,
    "___outgoingRequests" integer DEFAULT '0'::bigint NOT NULL,
    "___totalTime" integer DEFAULT '0'::bigint NOT NULL,
    "___incomingBytes" integer DEFAULT '0'::bigint NOT NULL,
    "___outgoingBytes" integer DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart__network OWNER TO "example-misskey-user";

--
-- Name: __chart__network_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__network_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__network_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__network_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__network_id_seq OWNED BY public.__chart__network.id;


--
-- Name: __chart__notes; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__notes (
    id integer NOT NULL,
    date integer NOT NULL,
    ___local_total integer DEFAULT '0'::bigint NOT NULL,
    ___local_inc integer DEFAULT '0'::bigint NOT NULL,
    ___local_dec integer DEFAULT '0'::bigint NOT NULL,
    ___local_diffs_normal integer DEFAULT '0'::bigint NOT NULL,
    ___local_diffs_reply integer DEFAULT '0'::bigint NOT NULL,
    ___local_diffs_renote integer DEFAULT '0'::bigint NOT NULL,
    ___remote_total integer DEFAULT '0'::bigint NOT NULL,
    ___remote_inc integer DEFAULT '0'::bigint NOT NULL,
    ___remote_dec integer DEFAULT '0'::bigint NOT NULL,
    ___remote_diffs_normal integer DEFAULT '0'::bigint NOT NULL,
    ___remote_diffs_reply integer DEFAULT '0'::bigint NOT NULL,
    ___remote_diffs_renote integer DEFAULT '0'::bigint NOT NULL,
    "___local_diffs_withFile" integer DEFAULT 0 NOT NULL,
    "___remote_diffs_withFile" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.__chart__notes OWNER TO "example-misskey-user";

--
-- Name: __chart__notes_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__notes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__notes_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__notes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__notes_id_seq OWNED BY public.__chart__notes.id;


--
-- Name: __chart__per_user_drive; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__per_user_drive (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    "___totalCount" integer DEFAULT '0'::bigint NOT NULL,
    "___totalSize" integer DEFAULT '0'::bigint NOT NULL,
    "___incCount" smallint DEFAULT '0'::bigint NOT NULL,
    "___incSize" integer DEFAULT '0'::bigint NOT NULL,
    "___decCount" smallint DEFAULT '0'::bigint NOT NULL,
    "___decSize" integer DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart__per_user_drive OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_drive_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__per_user_drive_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__per_user_drive_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_drive_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__per_user_drive_id_seq OWNED BY public.__chart__per_user_drive.id;


--
-- Name: __chart__per_user_following; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__per_user_following (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___local_followings_total integer DEFAULT '0'::bigint NOT NULL,
    ___local_followings_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___local_followings_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___local_followers_total integer DEFAULT '0'::bigint NOT NULL,
    ___local_followers_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___local_followers_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_followings_total integer DEFAULT '0'::bigint NOT NULL,
    ___remote_followings_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_followings_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_followers_total integer DEFAULT '0'::bigint NOT NULL,
    ___remote_followers_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_followers_dec smallint DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart__per_user_following OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_following_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__per_user_following_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__per_user_following_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_following_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__per_user_following_id_seq OWNED BY public.__chart__per_user_following.id;


--
-- Name: __chart__per_user_notes; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__per_user_notes (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___total integer DEFAULT '0'::bigint NOT NULL,
    ___inc smallint DEFAULT '0'::bigint NOT NULL,
    ___dec smallint DEFAULT '0'::bigint NOT NULL,
    ___diffs_normal smallint DEFAULT '0'::bigint NOT NULL,
    ___diffs_reply smallint DEFAULT '0'::bigint NOT NULL,
    ___diffs_renote smallint DEFAULT '0'::bigint NOT NULL,
    "___diffs_withFile" smallint DEFAULT '0'::smallint NOT NULL
);


ALTER TABLE public.__chart__per_user_notes OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_notes_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__per_user_notes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__per_user_notes_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_notes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__per_user_notes_id_seq OWNED BY public.__chart__per_user_notes.id;


--
-- Name: __chart__per_user_pv; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__per_user_pv (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    unique_temp___upv_user character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___upv_user smallint DEFAULT '0'::smallint NOT NULL,
    ___pv_user smallint DEFAULT '0'::smallint NOT NULL,
    unique_temp___upv_visitor character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___upv_visitor smallint DEFAULT '0'::smallint NOT NULL,
    ___pv_visitor smallint DEFAULT '0'::smallint NOT NULL
);


ALTER TABLE public.__chart__per_user_pv OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_pv_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__per_user_pv_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__per_user_pv_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_pv_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__per_user_pv_id_seq OWNED BY public.__chart__per_user_pv.id;


--
-- Name: __chart__per_user_reaction; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__per_user_reaction (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___local_count smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_count smallint DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart__per_user_reaction OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_reaction_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__per_user_reaction_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__per_user_reaction_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__per_user_reaction_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__per_user_reaction_id_seq OWNED BY public.__chart__per_user_reaction.id;


--
-- Name: __chart__test; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__test (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128),
    ___foo_total bigint NOT NULL,
    ___foo_inc bigint NOT NULL,
    ___foo_dec bigint NOT NULL
);


ALTER TABLE public.__chart__test OWNER TO "example-misskey-user";

--
-- Name: __chart__test_grouped; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__test_grouped (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128),
    ___foo_total bigint NOT NULL,
    ___foo_inc bigint NOT NULL,
    ___foo_dec bigint NOT NULL
);


ALTER TABLE public.__chart__test_grouped OWNER TO "example-misskey-user";

--
-- Name: __chart__test_grouped_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__test_grouped_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__test_grouped_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__test_grouped_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__test_grouped_id_seq OWNED BY public.__chart__test_grouped.id;


--
-- Name: __chart__test_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__test_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__test_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__test_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__test_id_seq OWNED BY public.__chart__test.id;


--
-- Name: __chart__test_unique; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__test_unique (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128),
    ___foo character varying[] DEFAULT '{}'::character varying[] NOT NULL
);


ALTER TABLE public.__chart__test_unique OWNER TO "example-misskey-user";

--
-- Name: __chart__test_unique_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__test_unique_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__test_unique_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__test_unique_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__test_unique_id_seq OWNED BY public.__chart__test_unique.id;


--
-- Name: __chart__users; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart__users (
    id integer NOT NULL,
    date integer NOT NULL,
    ___local_total integer DEFAULT '0'::bigint NOT NULL,
    ___local_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___local_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_total integer DEFAULT '0'::bigint NOT NULL,
    ___remote_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_dec smallint DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart__users OWNER TO "example-misskey-user";

--
-- Name: __chart__users_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart__users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart__users_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart__users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart__users_id_seq OWNED BY public.__chart__users.id;


--
-- Name: __chart_day__active_users; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__active_users (
    id integer NOT NULL,
    date integer NOT NULL,
    "unique_temp___registeredWithinWeek" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredWithinWeek" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredWithinMonth" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredWithinMonth" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredWithinYear" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredWithinYear" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredOutsideWeek" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredOutsideWeek" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredOutsideMonth" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredOutsideMonth" integer DEFAULT '0'::smallint NOT NULL,
    "unique_temp___registeredOutsideYear" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___registeredOutsideYear" integer DEFAULT '0'::smallint NOT NULL,
    "___readWrite" integer DEFAULT '0'::smallint NOT NULL,
    unique_temp___read character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___read integer DEFAULT '0'::smallint NOT NULL,
    unique_temp___write character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___write integer DEFAULT '0'::smallint NOT NULL
);


ALTER TABLE public.__chart_day__active_users OWNER TO "example-misskey-user";

--
-- Name: __chart_day__active_users_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__active_users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__active_users_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__active_users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__active_users_id_seq OWNED BY public.__chart_day__active_users.id;


--
-- Name: __chart_day__ap_request; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__ap_request (
    id integer NOT NULL,
    date integer NOT NULL,
    "___deliverFailed" integer DEFAULT 0 NOT NULL,
    "___deliverSucceeded" integer DEFAULT 0 NOT NULL,
    "___inboxReceived" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.__chart_day__ap_request OWNER TO "example-misskey-user";

--
-- Name: __chart_day__ap_request_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__ap_request_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__ap_request_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__ap_request_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__ap_request_id_seq OWNED BY public.__chart_day__ap_request.id;


--
-- Name: __chart_day__drive; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__drive (
    id integer NOT NULL,
    date integer NOT NULL,
    "___local_incCount" integer DEFAULT '0'::bigint NOT NULL,
    "___local_incSize" integer DEFAULT '0'::bigint NOT NULL,
    "___local_decCount" integer DEFAULT '0'::bigint NOT NULL,
    "___local_decSize" integer DEFAULT '0'::bigint NOT NULL,
    "___remote_incCount" integer DEFAULT '0'::bigint NOT NULL,
    "___remote_incSize" integer DEFAULT '0'::bigint NOT NULL,
    "___remote_decCount" integer DEFAULT '0'::bigint NOT NULL,
    "___remote_decSize" integer DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart_day__drive OWNER TO "example-misskey-user";

--
-- Name: __chart_day__drive_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__drive_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__drive_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__drive_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__drive_id_seq OWNED BY public.__chart_day__drive.id;


--
-- Name: __chart_day__federation; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__federation (
    id integer NOT NULL,
    date integer NOT NULL,
    "unique_temp___deliveredInstances" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___deliveredInstances" smallint DEFAULT '0'::smallint NOT NULL,
    "unique_temp___inboxInstances" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "___inboxInstances" smallint DEFAULT '0'::smallint NOT NULL,
    unique_temp___stalled character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___stalled smallint DEFAULT '0'::smallint NOT NULL,
    ___sub smallint DEFAULT '0'::smallint NOT NULL,
    ___pub smallint DEFAULT '0'::smallint NOT NULL,
    ___pubsub smallint DEFAULT '0'::smallint NOT NULL,
    "___subActive" smallint DEFAULT '0'::smallint NOT NULL,
    "___pubActive" smallint DEFAULT '0'::smallint NOT NULL
);


ALTER TABLE public.__chart_day__federation OWNER TO "example-misskey-user";

--
-- Name: __chart_day__federation_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__federation_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__federation_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__federation_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__federation_id_seq OWNED BY public.__chart_day__federation.id;


--
-- Name: __chart_day__hashtag; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__hashtag (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___local_users integer DEFAULT 0 NOT NULL,
    ___remote_users integer DEFAULT 0 NOT NULL,
    unique_temp___local_users character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    unique_temp___remote_users character varying[] DEFAULT '{}'::character varying[] NOT NULL
);


ALTER TABLE public.__chart_day__hashtag OWNER TO "example-misskey-user";

--
-- Name: __chart_day__hashtag_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__hashtag_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__hashtag_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__hashtag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__hashtag_id_seq OWNED BY public.__chart_day__hashtag.id;


--
-- Name: __chart_day__instance; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__instance (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___requests_failed smallint DEFAULT '0'::bigint NOT NULL,
    ___requests_succeeded smallint DEFAULT '0'::bigint NOT NULL,
    ___requests_received smallint DEFAULT '0'::bigint NOT NULL,
    ___notes_total integer DEFAULT '0'::bigint NOT NULL,
    ___notes_inc integer DEFAULT '0'::bigint NOT NULL,
    ___notes_dec integer DEFAULT '0'::bigint NOT NULL,
    ___notes_diffs_normal integer DEFAULT '0'::bigint NOT NULL,
    ___notes_diffs_reply integer DEFAULT '0'::bigint NOT NULL,
    ___notes_diffs_renote integer DEFAULT '0'::bigint NOT NULL,
    ___users_total integer DEFAULT '0'::bigint NOT NULL,
    ___users_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___users_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___following_total integer DEFAULT '0'::bigint NOT NULL,
    ___following_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___following_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___followers_total integer DEFAULT '0'::bigint NOT NULL,
    ___followers_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___followers_dec smallint DEFAULT '0'::bigint NOT NULL,
    "___drive_totalFiles" integer DEFAULT '0'::bigint NOT NULL,
    "___drive_incFiles" integer DEFAULT '0'::bigint NOT NULL,
    "___drive_incUsage" integer DEFAULT '0'::bigint NOT NULL,
    "___drive_decFiles" integer DEFAULT '0'::bigint NOT NULL,
    "___drive_decUsage" integer DEFAULT '0'::bigint NOT NULL,
    "___notes_diffs_withFile" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.__chart_day__instance OWNER TO "example-misskey-user";

--
-- Name: __chart_day__instance_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__instance_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__instance_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__instance_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__instance_id_seq OWNED BY public.__chart_day__instance.id;


--
-- Name: __chart_day__network; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__network (
    id integer NOT NULL,
    date integer NOT NULL,
    "___incomingRequests" integer DEFAULT '0'::bigint NOT NULL,
    "___outgoingRequests" integer DEFAULT '0'::bigint NOT NULL,
    "___totalTime" integer DEFAULT '0'::bigint NOT NULL,
    "___incomingBytes" integer DEFAULT '0'::bigint NOT NULL,
    "___outgoingBytes" integer DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart_day__network OWNER TO "example-misskey-user";

--
-- Name: __chart_day__network_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__network_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__network_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__network_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__network_id_seq OWNED BY public.__chart_day__network.id;


--
-- Name: __chart_day__notes; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__notes (
    id integer NOT NULL,
    date integer NOT NULL,
    ___local_total integer DEFAULT '0'::bigint NOT NULL,
    ___local_inc integer DEFAULT '0'::bigint NOT NULL,
    ___local_dec integer DEFAULT '0'::bigint NOT NULL,
    ___local_diffs_normal integer DEFAULT '0'::bigint NOT NULL,
    ___local_diffs_reply integer DEFAULT '0'::bigint NOT NULL,
    ___local_diffs_renote integer DEFAULT '0'::bigint NOT NULL,
    ___remote_total integer DEFAULT '0'::bigint NOT NULL,
    ___remote_inc integer DEFAULT '0'::bigint NOT NULL,
    ___remote_dec integer DEFAULT '0'::bigint NOT NULL,
    ___remote_diffs_normal integer DEFAULT '0'::bigint NOT NULL,
    ___remote_diffs_reply integer DEFAULT '0'::bigint NOT NULL,
    ___remote_diffs_renote integer DEFAULT '0'::bigint NOT NULL,
    "___local_diffs_withFile" integer DEFAULT 0 NOT NULL,
    "___remote_diffs_withFile" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.__chart_day__notes OWNER TO "example-misskey-user";

--
-- Name: __chart_day__notes_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__notes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__notes_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__notes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__notes_id_seq OWNED BY public.__chart_day__notes.id;


--
-- Name: __chart_day__per_user_drive; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__per_user_drive (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    "___totalCount" integer DEFAULT '0'::bigint NOT NULL,
    "___totalSize" integer DEFAULT '0'::bigint NOT NULL,
    "___incCount" smallint DEFAULT '0'::bigint NOT NULL,
    "___incSize" integer DEFAULT '0'::bigint NOT NULL,
    "___decCount" smallint DEFAULT '0'::bigint NOT NULL,
    "___decSize" integer DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart_day__per_user_drive OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_drive_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__per_user_drive_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__per_user_drive_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_drive_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__per_user_drive_id_seq OWNED BY public.__chart_day__per_user_drive.id;


--
-- Name: __chart_day__per_user_following; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__per_user_following (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___local_followings_total integer DEFAULT '0'::bigint NOT NULL,
    ___local_followings_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___local_followings_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___local_followers_total integer DEFAULT '0'::bigint NOT NULL,
    ___local_followers_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___local_followers_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_followings_total integer DEFAULT '0'::bigint NOT NULL,
    ___remote_followings_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_followings_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_followers_total integer DEFAULT '0'::bigint NOT NULL,
    ___remote_followers_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_followers_dec smallint DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart_day__per_user_following OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_following_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__per_user_following_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__per_user_following_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_following_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__per_user_following_id_seq OWNED BY public.__chart_day__per_user_following.id;


--
-- Name: __chart_day__per_user_notes; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__per_user_notes (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___total integer DEFAULT '0'::bigint NOT NULL,
    ___inc smallint DEFAULT '0'::bigint NOT NULL,
    ___dec smallint DEFAULT '0'::bigint NOT NULL,
    ___diffs_normal smallint DEFAULT '0'::bigint NOT NULL,
    ___diffs_reply smallint DEFAULT '0'::bigint NOT NULL,
    ___diffs_renote smallint DEFAULT '0'::bigint NOT NULL,
    "___diffs_withFile" smallint DEFAULT '0'::smallint NOT NULL
);


ALTER TABLE public.__chart_day__per_user_notes OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_notes_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__per_user_notes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__per_user_notes_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_notes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__per_user_notes_id_seq OWNED BY public.__chart_day__per_user_notes.id;


--
-- Name: __chart_day__per_user_pv; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__per_user_pv (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    unique_temp___upv_user character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___upv_user smallint DEFAULT '0'::smallint NOT NULL,
    ___pv_user smallint DEFAULT '0'::smallint NOT NULL,
    unique_temp___upv_visitor character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    ___upv_visitor smallint DEFAULT '0'::smallint NOT NULL,
    ___pv_visitor smallint DEFAULT '0'::smallint NOT NULL
);


ALTER TABLE public.__chart_day__per_user_pv OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_pv_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__per_user_pv_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__per_user_pv_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_pv_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__per_user_pv_id_seq OWNED BY public.__chart_day__per_user_pv.id;


--
-- Name: __chart_day__per_user_reaction; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__per_user_reaction (
    id integer NOT NULL,
    date integer NOT NULL,
    "group" character varying(128) NOT NULL,
    ___local_count smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_count smallint DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart_day__per_user_reaction OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_reaction_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__per_user_reaction_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__per_user_reaction_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__per_user_reaction_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__per_user_reaction_id_seq OWNED BY public.__chart_day__per_user_reaction.id;


--
-- Name: __chart_day__users; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.__chart_day__users (
    id integer NOT NULL,
    date integer NOT NULL,
    ___local_total integer DEFAULT '0'::bigint NOT NULL,
    ___local_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___local_dec smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_total integer DEFAULT '0'::bigint NOT NULL,
    ___remote_inc smallint DEFAULT '0'::bigint NOT NULL,
    ___remote_dec smallint DEFAULT '0'::bigint NOT NULL
);


ALTER TABLE public.__chart_day__users OWNER TO "example-misskey-user";

--
-- Name: __chart_day__users_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.__chart_day__users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.__chart_day__users_id_seq OWNER TO "example-misskey-user";

--
-- Name: __chart_day__users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.__chart_day__users_id_seq OWNED BY public.__chart_day__users.id;


--
-- Name: abuse_user_report; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.abuse_user_report (
    id character varying(32) NOT NULL,
    "targetUserId" character varying(32) NOT NULL,
    "reporterId" character varying(32) NOT NULL,
    "assigneeId" character varying(32),
    resolved boolean DEFAULT false NOT NULL,
    comment character varying(2048) NOT NULL,
    "targetUserHost" character varying(128),
    "reporterHost" character varying(128),
    forwarded boolean DEFAULT false NOT NULL
);


ALTER TABLE public.abuse_user_report OWNER TO "example-misskey-user";

--
-- Name: COLUMN abuse_user_report."targetUserHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.abuse_user_report."targetUserHost" IS '[Denormalized]';


--
-- Name: COLUMN abuse_user_report."reporterHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.abuse_user_report."reporterHost" IS '[Denormalized]';


--
-- Name: access_token; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.access_token (
    id character varying(32) NOT NULL,
    token character varying(128) NOT NULL,
    hash character varying(128) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "appId" character varying(32),
    "lastUsedAt" timestamp with time zone,
    session character varying(128),
    name character varying(128),
    description character varying(512),
    "iconUrl" character varying(512),
    permission character varying(64)[] DEFAULT '{}'::character varying[] NOT NULL,
    fetched boolean DEFAULT false NOT NULL
);


ALTER TABLE public.access_token OWNER TO "example-misskey-user";

--
-- Name: ad; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.ad (
    id character varying(32) NOT NULL,
    "expiresAt" timestamp with time zone NOT NULL,
    place character varying(32) NOT NULL,
    priority character varying(32) NOT NULL,
    url character varying(1024) NOT NULL,
    "imageUrl" character varying(1024) NOT NULL,
    memo character varying(8192) NOT NULL,
    ratio integer DEFAULT 1 NOT NULL,
    "startsAt" timestamp with time zone DEFAULT now() NOT NULL,
    "dayOfWeek" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.ad OWNER TO "example-misskey-user";

--
-- Name: COLUMN ad."expiresAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.ad."expiresAt" IS 'The expired date of the Ad.';


--
-- Name: COLUMN ad."startsAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.ad."startsAt" IS 'The expired date of the Ad.';


--
-- Name: announcement; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.announcement (
    id character varying(32) NOT NULL,
    text character varying(8192) NOT NULL,
    title character varying(256) NOT NULL,
    "imageUrl" character varying(1024),
    "updatedAt" timestamp with time zone,
    display character varying(256) DEFAULT 'normal'::character varying NOT NULL,
    "needConfirmationToRead" boolean DEFAULT false NOT NULL,
    "isActive" boolean DEFAULT true NOT NULL,
    "forExistingUsers" boolean DEFAULT false NOT NULL,
    "userId" character varying(32),
    icon character varying(256) DEFAULT 'info'::character varying NOT NULL,
    silence boolean DEFAULT false NOT NULL
);


ALTER TABLE public.announcement OWNER TO "example-misskey-user";

--
-- Name: COLUMN announcement."updatedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.announcement."updatedAt" IS 'The updated date of the Announcement.';


--
-- Name: announcement_read; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.announcement_read (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "announcementId" character varying(32) NOT NULL
);


ALTER TABLE public.announcement_read OWNER TO "example-misskey-user";

--
-- Name: antenna; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.antenna (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    name character varying(128) NOT NULL,
    src public.antenna_src_enum NOT NULL,
    "userListId" character varying(32),
    keywords jsonb DEFAULT '[]'::jsonb NOT NULL,
    "withFile" boolean NOT NULL,
    expression character varying(2048),
    notify boolean NOT NULL,
    "caseSensitive" boolean DEFAULT false NOT NULL,
    "withReplies" boolean DEFAULT false NOT NULL,
    users character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    "excludeKeywords" jsonb DEFAULT '[]'::jsonb NOT NULL,
    "lastUsedAt" timestamp with time zone NOT NULL,
    "isActive" boolean DEFAULT true NOT NULL,
    "localOnly" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.antenna OWNER TO "example-misskey-user";

--
-- Name: COLUMN antenna."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.antenna."userId" IS 'The owner ID.';


--
-- Name: COLUMN antenna.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.antenna.name IS 'The name of the Antenna.';


--
-- Name: app; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.app (
    id character varying(32) NOT NULL,
    "userId" character varying(32),
    secret character varying(64) NOT NULL,
    name character varying(128) NOT NULL,
    description character varying(512) NOT NULL,
    permission character varying(64)[] NOT NULL,
    "callbackUrl" character varying(512)
);


ALTER TABLE public.app OWNER TO "example-misskey-user";

--
-- Name: COLUMN app."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.app."userId" IS 'The owner ID.';


--
-- Name: COLUMN app.secret; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.app.secret IS 'The secret key of the App.';


--
-- Name: COLUMN app.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.app.name IS 'The name of the App.';


--
-- Name: COLUMN app.description; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.app.description IS 'The description of the App.';


--
-- Name: COLUMN app.permission; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.app.permission IS 'The permission of the App.';


--
-- Name: COLUMN app."callbackUrl"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.app."callbackUrl" IS 'The callbackUrl of the App.';


--
-- Name: auth_session; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.auth_session (
    id character varying(32) NOT NULL,
    token character varying(128) NOT NULL,
    "userId" character varying(32),
    "appId" character varying(32) NOT NULL
);


ALTER TABLE public.auth_session OWNER TO "example-misskey-user";

--
-- Name: avatar_decoration; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.avatar_decoration (
    id character varying(32) NOT NULL,
    "updatedAt" timestamp with time zone,
    url character varying(1024) NOT NULL,
    name character varying(256) NOT NULL,
    description character varying(2048) NOT NULL,
    "roleIdsThatCanBeUsedThisDecoration" character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL
);


ALTER TABLE public.avatar_decoration OWNER TO "example-misskey-user";

--
-- Name: blocking; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.blocking (
    id character varying(32) NOT NULL,
    "blockeeId" character varying(32) NOT NULL,
    "blockerId" character varying(32) NOT NULL
);


ALTER TABLE public.blocking OWNER TO "example-misskey-user";

--
-- Name: COLUMN blocking."blockeeId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.blocking."blockeeId" IS 'The blockee user ID.';


--
-- Name: COLUMN blocking."blockerId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.blocking."blockerId" IS 'The blocker user ID.';


--
-- Name: bubble_game_record; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.bubble_game_record (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "seededAt" timestamp with time zone NOT NULL,
    seed character varying(1024) NOT NULL,
    "gameVersion" integer NOT NULL,
    "gameMode" character varying(128) NOT NULL,
    score integer NOT NULL,
    logs jsonb DEFAULT '[]'::jsonb NOT NULL,
    "isVerified" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.bubble_game_record OWNER TO "example-misskey-user";

--
-- Name: channel; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.channel (
    id character varying(32) NOT NULL,
    "lastNotedAt" timestamp with time zone,
    "userId" character varying(32),
    name character varying(128) NOT NULL,
    description character varying(2048),
    "bannerId" character varying(32),
    "notesCount" integer DEFAULT 0 NOT NULL,
    "usersCount" integer DEFAULT 0 NOT NULL,
    "pinnedNoteIds" character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL,
    color character varying(16) DEFAULT '#86b300'::character varying NOT NULL,
    "isArchived" boolean DEFAULT false NOT NULL,
    "isSensitive" boolean DEFAULT false NOT NULL,
    "allowRenoteToExternal" boolean DEFAULT true NOT NULL
);


ALTER TABLE public.channel OWNER TO "example-misskey-user";

--
-- Name: COLUMN channel."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel."userId" IS 'The owner ID.';


--
-- Name: COLUMN channel.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel.name IS 'The name of the Channel.';


--
-- Name: COLUMN channel.description; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel.description IS 'The description of the Channel.';


--
-- Name: COLUMN channel."bannerId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel."bannerId" IS 'The ID of banner Channel.';


--
-- Name: COLUMN channel."notesCount"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel."notesCount" IS 'The count of notes.';


--
-- Name: COLUMN channel."usersCount"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel."usersCount" IS 'The count of users.';


--
-- Name: channel_favorite; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.channel_favorite (
    id character varying(32) NOT NULL,
    "channelId" character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL
);


ALTER TABLE public.channel_favorite OWNER TO "example-misskey-user";

--
-- Name: channel_following; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.channel_following (
    id character varying(32) NOT NULL,
    "followeeId" character varying(32) NOT NULL,
    "followerId" character varying(32) NOT NULL
);


ALTER TABLE public.channel_following OWNER TO "example-misskey-user";

--
-- Name: COLUMN channel_following."followeeId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel_following."followeeId" IS 'The followee channel ID.';


--
-- Name: COLUMN channel_following."followerId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel_following."followerId" IS 'The follower user ID.';


--
-- Name: channel_note_pining; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.channel_note_pining (
    id character varying(32) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "channelId" character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL
);


ALTER TABLE public.channel_note_pining OWNER TO "example-misskey-user";

--
-- Name: COLUMN channel_note_pining."createdAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.channel_note_pining."createdAt" IS 'The created date of the ChannelNotePining.';


--
-- Name: clip; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.clip (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    name character varying(128) NOT NULL,
    "isPublic" boolean DEFAULT false NOT NULL,
    description character varying(2048),
    "lastClippedAt" timestamp with time zone
);


ALTER TABLE public.clip OWNER TO "example-misskey-user";

--
-- Name: COLUMN clip."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.clip."userId" IS 'The owner ID.';


--
-- Name: COLUMN clip.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.clip.name IS 'The name of the Clip.';


--
-- Name: COLUMN clip.description; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.clip.description IS 'The description of the Clip.';


--
-- Name: clip_favorite; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.clip_favorite (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "clipId" character varying(32) NOT NULL
);


ALTER TABLE public.clip_favorite OWNER TO "example-misskey-user";

--
-- Name: clip_note; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.clip_note (
    id character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL,
    "clipId" character varying(32) NOT NULL
);


ALTER TABLE public.clip_note OWNER TO "example-misskey-user";

--
-- Name: COLUMN clip_note."noteId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.clip_note."noteId" IS 'The note ID.';


--
-- Name: COLUMN clip_note."clipId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.clip_note."clipId" IS 'The clip ID.';


--
-- Name: drive_file; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.drive_file (
    id character varying(32) NOT NULL,
    "userId" character varying(32),
    "userHost" character varying(128),
    md5 character varying(32) NOT NULL,
    name character varying(256) NOT NULL,
    type character varying(128) NOT NULL,
    size integer NOT NULL,
    comment character varying(512),
    properties jsonb DEFAULT '{}'::jsonb NOT NULL,
    "storedInternal" boolean NOT NULL,
    url character varying(512) NOT NULL,
    "thumbnailUrl" character varying(512),
    "webpublicUrl" character varying(512),
    "accessKey" character varying(256),
    "thumbnailAccessKey" character varying(256),
    "webpublicAccessKey" character varying(256),
    uri character varying(512),
    src character varying(512),
    "folderId" character varying(32),
    "isSensitive" boolean DEFAULT false NOT NULL,
    "isLink" boolean DEFAULT false NOT NULL,
    blurhash character varying(128),
    "webpublicType" character varying(128),
    "requestHeaders" jsonb DEFAULT '{}'::jsonb,
    "requestIp" character varying(128),
    "maybeSensitive" boolean DEFAULT false NOT NULL,
    "maybePorn" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.drive_file OWNER TO "example-misskey-user";

--
-- Name: COLUMN drive_file."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file."userId" IS 'The owner ID.';


--
-- Name: COLUMN drive_file."userHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file."userHost" IS 'The host of owner. It will be null if the user in local.';


--
-- Name: COLUMN drive_file.md5; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.md5 IS 'The MD5 hash of the DriveFile.';


--
-- Name: COLUMN drive_file.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.name IS 'The file name of the DriveFile.';


--
-- Name: COLUMN drive_file.type; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.type IS 'The content type (MIME) of the DriveFile.';


--
-- Name: COLUMN drive_file.size; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.size IS 'The file size (bytes) of the DriveFile.';


--
-- Name: COLUMN drive_file.comment; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.comment IS 'The comment of the DriveFile.';


--
-- Name: COLUMN drive_file.properties; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.properties IS 'The any properties of the DriveFile. For example, it includes image width/height.';


--
-- Name: COLUMN drive_file.url; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.url IS 'The URL of the DriveFile.';


--
-- Name: COLUMN drive_file."thumbnailUrl"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file."thumbnailUrl" IS 'The URL of the thumbnail of the DriveFile.';


--
-- Name: COLUMN drive_file."webpublicUrl"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file."webpublicUrl" IS 'The URL of the webpublic of the DriveFile.';


--
-- Name: COLUMN drive_file.uri; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.uri IS 'The URI of the DriveFile. it will be null when the DriveFile is local.';


--
-- Name: COLUMN drive_file."folderId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file."folderId" IS 'The parent folder ID. If null, it means the DriveFile is located in root.';


--
-- Name: COLUMN drive_file."isSensitive"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file."isSensitive" IS 'Whether the DriveFile is NSFW.';


--
-- Name: COLUMN drive_file."isLink"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file."isLink" IS 'Whether the DriveFile is direct link to remote server.';


--
-- Name: COLUMN drive_file.blurhash; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file.blurhash IS 'The BlurHash string.';


--
-- Name: COLUMN drive_file."maybeSensitive"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_file."maybeSensitive" IS 'Whether the DriveFile is NSFW. (predict)';


--
-- Name: drive_folder; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.drive_folder (
    id character varying(32) NOT NULL,
    name character varying(128) NOT NULL,
    "userId" character varying(32),
    "parentId" character varying(32)
);


ALTER TABLE public.drive_folder OWNER TO "example-misskey-user";

--
-- Name: COLUMN drive_folder.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_folder.name IS 'The name of the DriveFolder.';


--
-- Name: COLUMN drive_folder."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_folder."userId" IS 'The owner ID.';


--
-- Name: COLUMN drive_folder."parentId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.drive_folder."parentId" IS 'The parent folder ID. If null, it means the DriveFolder is located in root.';


--
-- Name: emoji; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.emoji (
    id character varying(32) NOT NULL,
    "updatedAt" timestamp with time zone,
    name character varying(128) NOT NULL,
    host character varying(128),
    "originalUrl" character varying(512) NOT NULL,
    uri character varying(512),
    type character varying(64),
    aliases character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL,
    category character varying(128),
    "publicUrl" character varying(512) DEFAULT ''::character varying NOT NULL,
    license character varying(1024),
    "localOnly" boolean DEFAULT false NOT NULL,
    "isSensitive" boolean DEFAULT false NOT NULL,
    "roleIdsThatCanBeUsedThisEmojiAsReaction" character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL
);


ALTER TABLE public.emoji OWNER TO "example-misskey-user";

--
-- Name: flash; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.flash (
    id character varying(32) NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL,
    title character varying(256) NOT NULL,
    summary character varying(1024) NOT NULL,
    "userId" character varying(32) NOT NULL,
    script character varying(65536) NOT NULL,
    permissions character varying(256)[] DEFAULT '{}'::character varying[] NOT NULL,
    "likedCount" integer DEFAULT 0 NOT NULL,
    visibility character varying(512) DEFAULT 'public'::character varying
);


ALTER TABLE public.flash OWNER TO "example-misskey-user";

--
-- Name: COLUMN flash."updatedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.flash."updatedAt" IS 'The updated date of the Flash.';


--
-- Name: COLUMN flash."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.flash."userId" IS 'The ID of author.';


--
-- Name: flash_like; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.flash_like (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "flashId" character varying(32) NOT NULL
);


ALTER TABLE public.flash_like OWNER TO "example-misskey-user";

--
-- Name: follow_request; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.follow_request (
    id character varying(32) NOT NULL,
    "followeeId" character varying(32) NOT NULL,
    "followerId" character varying(32) NOT NULL,
    "requestId" character varying(128),
    "followerHost" character varying(128),
    "followerInbox" character varying(512),
    "followerSharedInbox" character varying(512),
    "followeeHost" character varying(128),
    "followeeInbox" character varying(512),
    "followeeSharedInbox" character varying(512),
    "withReplies" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.follow_request OWNER TO "example-misskey-user";

--
-- Name: COLUMN follow_request."followeeId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."followeeId" IS 'The followee user ID.';


--
-- Name: COLUMN follow_request."followerId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."followerId" IS 'The follower user ID.';


--
-- Name: COLUMN follow_request."requestId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."requestId" IS 'id of Follow Activity.';


--
-- Name: COLUMN follow_request."followerHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."followerHost" IS '[Denormalized]';


--
-- Name: COLUMN follow_request."followerInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."followerInbox" IS '[Denormalized]';


--
-- Name: COLUMN follow_request."followerSharedInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."followerSharedInbox" IS '[Denormalized]';


--
-- Name: COLUMN follow_request."followeeHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."followeeHost" IS '[Denormalized]';


--
-- Name: COLUMN follow_request."followeeInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."followeeInbox" IS '[Denormalized]';


--
-- Name: COLUMN follow_request."followeeSharedInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.follow_request."followeeSharedInbox" IS '[Denormalized]';


--
-- Name: following; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.following (
    id character varying(32) NOT NULL,
    "followeeId" character varying(32) NOT NULL,
    "followerId" character varying(32) NOT NULL,
    "followerHost" character varying(128),
    "followerInbox" character varying(512),
    "followerSharedInbox" character varying(512),
    "followeeHost" character varying(128),
    "followeeInbox" character varying(512),
    "followeeSharedInbox" character varying(512),
    notify character varying(32),
    "withReplies" boolean DEFAULT false NOT NULL,
    "isFollowerHibernated" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.following OWNER TO "example-misskey-user";

--
-- Name: COLUMN following."followeeId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.following."followeeId" IS 'The followee user ID.';


--
-- Name: COLUMN following."followerId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.following."followerId" IS 'The follower user ID.';


--
-- Name: COLUMN following."followerHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.following."followerHost" IS '[Denormalized]';


--
-- Name: COLUMN following."followerInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.following."followerInbox" IS '[Denormalized]';


--
-- Name: COLUMN following."followerSharedInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.following."followerSharedInbox" IS '[Denormalized]';


--
-- Name: COLUMN following."followeeHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.following."followeeHost" IS '[Denormalized]';


--
-- Name: COLUMN following."followeeInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.following."followeeInbox" IS '[Denormalized]';


--
-- Name: COLUMN following."followeeSharedInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.following."followeeSharedInbox" IS '[Denormalized]';


--
-- Name: gallery_like; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.gallery_like (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "postId" character varying(32) NOT NULL
);


ALTER TABLE public.gallery_like OWNER TO "example-misskey-user";

--
-- Name: gallery_post; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.gallery_post (
    id character varying(32) NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL,
    title character varying(256) NOT NULL,
    description character varying(2048),
    "userId" character varying(32) NOT NULL,
    "fileIds" character varying(32)[] DEFAULT '{}'::character varying[] NOT NULL,
    "isSensitive" boolean DEFAULT false NOT NULL,
    "likedCount" integer DEFAULT 0 NOT NULL,
    tags character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL
);


ALTER TABLE public.gallery_post OWNER TO "example-misskey-user";

--
-- Name: COLUMN gallery_post."updatedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.gallery_post."updatedAt" IS 'The updated date of the GalleryPost.';


--
-- Name: COLUMN gallery_post."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.gallery_post."userId" IS 'The ID of author.';


--
-- Name: COLUMN gallery_post."isSensitive"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.gallery_post."isSensitive" IS 'Whether the post is sensitive.';


--
-- Name: hashtag; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.hashtag (
    id character varying(32) NOT NULL,
    name character varying(128) NOT NULL,
    "mentionedUserIds" character varying(32)[] NOT NULL,
    "mentionedUsersCount" integer DEFAULT 0 NOT NULL,
    "mentionedLocalUserIds" character varying(32)[] NOT NULL,
    "mentionedLocalUsersCount" integer DEFAULT 0 NOT NULL,
    "mentionedRemoteUserIds" character varying(32)[] NOT NULL,
    "mentionedRemoteUsersCount" integer DEFAULT 0 NOT NULL,
    "attachedUserIds" character varying(32)[] NOT NULL,
    "attachedUsersCount" integer DEFAULT 0 NOT NULL,
    "attachedLocalUserIds" character varying(32)[] NOT NULL,
    "attachedLocalUsersCount" integer DEFAULT 0 NOT NULL,
    "attachedRemoteUserIds" character varying(32)[] NOT NULL,
    "attachedRemoteUsersCount" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public.hashtag OWNER TO "example-misskey-user";

--
-- Name: instance; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.instance (
    id character varying(32) NOT NULL,
    "firstRetrievedAt" timestamp with time zone NOT NULL,
    host character varying(128) NOT NULL,
    "usersCount" integer DEFAULT 0 NOT NULL,
    "notesCount" integer DEFAULT 0 NOT NULL,
    "followingCount" integer DEFAULT 0 NOT NULL,
    "followersCount" integer DEFAULT 0 NOT NULL,
    "latestRequestReceivedAt" timestamp with time zone,
    "isNotResponding" boolean DEFAULT false NOT NULL,
    "softwareName" character varying(64),
    "softwareVersion" character varying(64),
    "openRegistrations" boolean,
    name character varying(256),
    description character varying(4096),
    "maintainerName" character varying(128),
    "maintainerEmail" character varying(256),
    "infoUpdatedAt" timestamp with time zone,
    "isSuspended" boolean DEFAULT false NOT NULL,
    "iconUrl" character varying(256),
    "themeColor" character varying(64),
    "faviconUrl" character varying(256),
    "moderationNote" character varying(16384) DEFAULT ''::character varying NOT NULL
);


ALTER TABLE public.instance OWNER TO "example-misskey-user";

--
-- Name: COLUMN instance."firstRetrievedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.instance."firstRetrievedAt" IS 'The caught date of the Instance.';


--
-- Name: COLUMN instance.host; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.instance.host IS 'The host of the Instance.';


--
-- Name: COLUMN instance."usersCount"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.instance."usersCount" IS 'The count of the users of the Instance.';


--
-- Name: COLUMN instance."notesCount"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.instance."notesCount" IS 'The count of the notes of the Instance.';


--
-- Name: COLUMN instance."softwareName"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.instance."softwareName" IS 'The software of the Instance.';


--
-- Name: messaging_message; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.messaging_message (
    id character varying(32) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "userId" character varying(32) NOT NULL,
    "recipientId" character varying(32),
    text character varying(4096),
    "isRead" boolean DEFAULT false NOT NULL,
    "fileId" character varying(32),
    "groupId" character varying(32),
    reads character varying(32)[] DEFAULT '{}'::character varying[] NOT NULL,
    uri character varying(512)
);


ALTER TABLE public.messaging_message OWNER TO "example-misskey-user";

--
-- Name: COLUMN messaging_message."createdAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.messaging_message."createdAt" IS 'The created date of the MessagingMessage.';


--
-- Name: COLUMN messaging_message."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.messaging_message."userId" IS 'The sender user ID.';


--
-- Name: COLUMN messaging_message."recipientId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.messaging_message."recipientId" IS 'The recipient user ID.';


--
-- Name: COLUMN messaging_message."groupId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.messaging_message."groupId" IS 'The recipient group ID.';


--
-- Name: meta; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.meta (
    id character varying(32) NOT NULL,
    name character varying(1024),
    description character varying(1024),
    "maintainerName" character varying(1024),
    "maintainerEmail" character varying(1024),
    "disableRegistration" boolean DEFAULT true NOT NULL,
    langs character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    "hiddenTags" character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    "blockedHosts" character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    "mascotImageUrl" character varying(1024),
    "bannerUrl" character varying(1024),
    "iconUrl" character varying(1024),
    "cacheRemoteFiles" boolean DEFAULT false NOT NULL,
    "enableRecaptcha" boolean DEFAULT false NOT NULL,
    "recaptchaSiteKey" character varying(1024),
    "recaptchaSecretKey" character varying(1024),
    "summalyProxy" character varying(1024),
    "enableEmail" boolean DEFAULT false NOT NULL,
    email character varying(1024),
    "smtpSecure" boolean DEFAULT false NOT NULL,
    "smtpHost" character varying(1024),
    "smtpPort" integer,
    "smtpUser" character varying(1024),
    "smtpPass" character varying(1024),
    "enableServiceWorker" boolean DEFAULT false NOT NULL,
    "swPublicKey" character varying(1024),
    "swPrivateKey" character varying(1024),
    "pinnedUsers" character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    "termsOfServiceUrl" character varying(1024),
    "repositoryUrl" character varying(1024) DEFAULT 'https://github.com/misskey-dev/misskey'::character varying,
    "feedbackUrl" character varying(1024) DEFAULT 'https://github.com/misskey-dev/misskey/issues/new'::character varying,
    "useObjectStorage" boolean DEFAULT false NOT NULL,
    "objectStorageBucket" character varying(1024),
    "objectStoragePrefix" character varying(1024),
    "objectStorageBaseUrl" character varying(1024),
    "objectStorageEndpoint" character varying(1024),
    "objectStorageRegion" character varying(1024),
    "objectStorageAccessKey" character varying(1024),
    "objectStorageSecretKey" character varying(1024),
    "objectStoragePort" integer,
    "objectStorageUseSSL" boolean DEFAULT true NOT NULL,
    "proxyAccountId" character varying(32),
    "objectStorageUseProxy" boolean DEFAULT true NOT NULL,
    "enableHcaptcha" boolean DEFAULT false NOT NULL,
    "hcaptchaSiteKey" character varying(1024),
    "hcaptchaSecretKey" character varying(1024),
    "objectStorageSetPublicRead" boolean DEFAULT false NOT NULL,
    "backgroundImageUrl" character varying(1024),
    "logoImageUrl" character varying(1024),
    "objectStorageS3ForcePathStyle" boolean DEFAULT true NOT NULL,
    "deeplAuthKey" character varying(1024),
    "deeplIsPro" boolean DEFAULT false NOT NULL,
    "emailRequiredForSignup" boolean DEFAULT false NOT NULL,
    "themeColor" character varying(1024),
    "defaultLightTheme" character varying(8192),
    "defaultDarkTheme" character varying(8192),
    "sensitiveMediaDetection" public.meta_sensitivemediadetection_enum DEFAULT 'none'::public.meta_sensitivemediadetection_enum NOT NULL,
    "sensitiveMediaDetectionSensitivity" public.meta_sensitivemediadetectionsensitivity_enum DEFAULT 'medium'::public.meta_sensitivemediadetectionsensitivity_enum NOT NULL,
    "setSensitiveFlagAutomatically" boolean DEFAULT false NOT NULL,
    "enableIpLogging" boolean DEFAULT false NOT NULL,
    "enableSensitiveMediaDetectionForVideos" boolean DEFAULT false NOT NULL,
    "enableActiveEmailValidation" boolean DEFAULT true NOT NULL,
    "enableTurnstile" boolean DEFAULT false NOT NULL,
    "turnstileSiteKey" character varying(1024),
    "turnstileSecretKey" character varying(1024),
    policies jsonb DEFAULT '{}'::jsonb NOT NULL,
    "sensitiveWords" character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    "enableChartsForRemoteUser" boolean DEFAULT true NOT NULL,
    "enableChartsForFederatedInstances" boolean DEFAULT true NOT NULL,
    "serverRules" character varying(280)[] DEFAULT '{}'::character varying[] NOT NULL,
    "preservedUsernames" character varying(1024)[] DEFAULT '{admin,administrator,root,system,maintainer,host,mod,moderator,owner,superuser,staff,auth,i,me,everyone,all,mention,mentions,example,user,users,account,accounts,official,help,helps,support,supports,info,information,informations,announce,announces,announcement,announcements,notice,notification,notifications,dev,developer,developers,tech,misskey}'::character varying[] NOT NULL,
    "serverErrorImageUrl" character varying(1024),
    "notFoundImageUrl" character varying(1024),
    "infoImageUrl" character varying(1024),
    "enableServerMachineStats" boolean DEFAULT false NOT NULL,
    "enableIdenticonGeneration" boolean DEFAULT true NOT NULL,
    "cacheRemoteSensitiveFiles" boolean DEFAULT true NOT NULL,
    "app192IconUrl" character varying(1024),
    "app512IconUrl" character varying(1024),
    "manifestJsonOverride" character varying(8192) DEFAULT '{}'::character varying NOT NULL,
    "shortName" character varying(64),
    "impressumUrl" character varying(1024),
    "privacyPolicyUrl" character varying(1024),
    "perLocalUserUserTimelineCacheMax" integer DEFAULT 300 NOT NULL,
    "perRemoteUserUserTimelineCacheMax" integer DEFAULT 100 NOT NULL,
    "perUserHomeTimelineCacheMax" integer DEFAULT 300 NOT NULL,
    "perUserListTimelineCacheMax" integer DEFAULT 300 NOT NULL,
    "notesPerOneAd" integer DEFAULT 0 NOT NULL,
    "silencedHosts" character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    "enableFanoutTimeline" boolean DEFAULT true NOT NULL,
    "enableFanoutTimelineDbFallback" boolean DEFAULT true NOT NULL,
    "verifymailAuthKey" character varying(1024),
    "enableVerifymailApi" boolean DEFAULT false NOT NULL,
    "bannedEmailDomains" character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    "truemailInstance" character varying(1024),
    "truemailAuthKey" character varying(1024),
    "enableTruemailApi" boolean DEFAULT false NOT NULL,
    "enableMcaptcha" boolean DEFAULT false NOT NULL,
    "mcaptchaSitekey" character varying(1024),
    "mcaptchaSecretKey" character varying(1024),
    "mcaptchaInstanceUrl" character varying(1024),
    "prohibitedWords" character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL
);


ALTER TABLE public.meta OWNER TO "example-misskey-user";

--
-- Name: migrations; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.migrations (
    id integer NOT NULL,
    "timestamp" bigint NOT NULL,
    name character varying NOT NULL
);


ALTER TABLE public.migrations OWNER TO "example-misskey-user";

--
-- Name: migrations_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.migrations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.migrations_id_seq OWNER TO "example-misskey-user";

--
-- Name: migrations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.migrations_id_seq OWNED BY public.migrations.id;


--
-- Name: moderation_log; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.moderation_log (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    type character varying(128) NOT NULL,
    info jsonb NOT NULL
);


ALTER TABLE public.moderation_log OWNER TO "example-misskey-user";

--
-- Name: muting; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.muting (
    id character varying(32) NOT NULL,
    "muteeId" character varying(32) NOT NULL,
    "muterId" character varying(32) NOT NULL,
    "expiresAt" timestamp with time zone
);


ALTER TABLE public.muting OWNER TO "example-misskey-user";

--
-- Name: COLUMN muting."muteeId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.muting."muteeId" IS 'The mutee user ID.';


--
-- Name: COLUMN muting."muterId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.muting."muterId" IS 'The muter user ID.';


--
-- Name: note; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.note (
    id character varying(32) NOT NULL,
    "replyId" character varying(32),
    "renoteId" character varying(32),
    text text,
    name character varying(256),
    cw character varying(512),
    "userId" character varying(32) NOT NULL,
    "localOnly" boolean DEFAULT false NOT NULL,
    "renoteCount" smallint DEFAULT 0 NOT NULL,
    "repliesCount" smallint DEFAULT 0 NOT NULL,
    reactions jsonb DEFAULT '{}'::jsonb NOT NULL,
    visibility public.note_visibility_enum NOT NULL,
    uri character varying(512),
    "fileIds" character varying(32)[] DEFAULT '{}'::character varying[] NOT NULL,
    "attachedFileTypes" character varying(256)[] DEFAULT '{}'::character varying[] NOT NULL,
    "visibleUserIds" character varying(32)[] DEFAULT '{}'::character varying[] NOT NULL,
    mentions character varying(32)[] DEFAULT '{}'::character varying[] NOT NULL,
    "mentionedRemoteUsers" text DEFAULT '[]'::text NOT NULL,
    emojis character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL,
    tags character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL,
    "hasPoll" boolean DEFAULT false NOT NULL,
    "userHost" character varying(128),
    "replyUserId" character varying(32),
    "replyUserHost" character varying(128),
    "renoteUserId" character varying(32),
    "renoteUserHost" character varying(128),
    url character varying(512),
    "channelId" character varying(32),
    "threadId" character varying(256),
    "reactionAcceptance" character varying(64),
    "clippedCount" smallint DEFAULT '0'::smallint NOT NULL,
    "reactionAndUserPairCache" character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL
);


ALTER TABLE public.note OWNER TO "example-misskey-user";

--
-- Name: COLUMN note."replyId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."replyId" IS 'The ID of reply target.';


--
-- Name: COLUMN note."renoteId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."renoteId" IS 'The ID of renote target.';


--
-- Name: COLUMN note."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."userId" IS 'The ID of author.';


--
-- Name: COLUMN note.uri; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note.uri IS 'The URI of a note. it will be null when the note is local.';


--
-- Name: COLUMN note."userHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."userHost" IS '[Denormalized]';


--
-- Name: COLUMN note."replyUserId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."replyUserId" IS '[Denormalized]';


--
-- Name: COLUMN note."replyUserHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."replyUserHost" IS '[Denormalized]';


--
-- Name: COLUMN note."renoteUserId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."renoteUserId" IS '[Denormalized]';


--
-- Name: COLUMN note."renoteUserHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."renoteUserHost" IS '[Denormalized]';


--
-- Name: COLUMN note.url; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note.url IS 'The human readable url of a note. it will be null when the note is local.';


--
-- Name: COLUMN note."channelId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note."channelId" IS 'The ID of source channel.';


--
-- Name: note_favorite; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.note_favorite (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL
);


ALTER TABLE public.note_favorite OWNER TO "example-misskey-user";

--
-- Name: note_reaction; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.note_reaction (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL,
    reaction character varying(260) NOT NULL
);


ALTER TABLE public.note_reaction OWNER TO "example-misskey-user";

--
-- Name: note_thread_muting; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.note_thread_muting (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "threadId" character varying(256) NOT NULL
);


ALTER TABLE public.note_thread_muting OWNER TO "example-misskey-user";

--
-- Name: note_unread; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.note_unread (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL,
    "noteUserId" character varying(32) NOT NULL,
    "isSpecified" boolean NOT NULL,
    "isMentioned" boolean NOT NULL,
    "noteChannelId" character varying(32)
);


ALTER TABLE public.note_unread OWNER TO "example-misskey-user";

--
-- Name: COLUMN note_unread."noteUserId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note_unread."noteUserId" IS '[Denormalized]';


--
-- Name: COLUMN note_unread."noteChannelId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note_unread."noteChannelId" IS '[Denormalized]';


--
-- Name: note_watching; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.note_watching (
    id character varying(32) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "userId" character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL,
    "noteUserId" character varying(32) NOT NULL
);


ALTER TABLE public.note_watching OWNER TO "example-misskey-user";

--
-- Name: COLUMN note_watching."createdAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note_watching."createdAt" IS 'The created date of the NoteWatching.';


--
-- Name: COLUMN note_watching."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note_watching."userId" IS 'The watcher ID.';


--
-- Name: COLUMN note_watching."noteId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note_watching."noteId" IS 'The target Note ID.';


--
-- Name: COLUMN note_watching."noteUserId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.note_watching."noteUserId" IS '[Denormalized]';


--
-- Name: page; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.page (
    id character varying(32) NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL,
    title character varying(256) NOT NULL,
    name character varying(256) NOT NULL,
    summary character varying(256),
    "alignCenter" boolean NOT NULL,
    font character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "eyeCatchingImageId" character varying(32),
    content jsonb DEFAULT '[]'::jsonb NOT NULL,
    variables jsonb DEFAULT '[]'::jsonb NOT NULL,
    visibility public.page_visibility_enum NOT NULL,
    "visibleUserIds" character varying(32)[] DEFAULT '{}'::character varying[] NOT NULL,
    "likedCount" integer DEFAULT 0 NOT NULL,
    "hideTitleWhenPinned" boolean DEFAULT false NOT NULL,
    script character varying(16384) DEFAULT ''::character varying NOT NULL
);


ALTER TABLE public.page OWNER TO "example-misskey-user";

--
-- Name: COLUMN page."updatedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.page."updatedAt" IS 'The updated date of the Page.';


--
-- Name: COLUMN page."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.page."userId" IS 'The ID of author.';


--
-- Name: page_like; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.page_like (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "pageId" character varying(32) NOT NULL
);


ALTER TABLE public.page_like OWNER TO "example-misskey-user";

--
-- Name: password_reset_request; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.password_reset_request (
    id character varying(32) NOT NULL,
    token character varying(256) NOT NULL,
    "userId" character varying(32) NOT NULL
);


ALTER TABLE public.password_reset_request OWNER TO "example-misskey-user";

--
-- Name: poll; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.poll (
    "noteId" character varying(32) NOT NULL,
    "expiresAt" timestamp with time zone,
    multiple boolean NOT NULL,
    choices character varying(256)[] DEFAULT '{}'::character varying[] NOT NULL,
    votes integer[] NOT NULL,
    "noteVisibility" public.poll_notevisibility_enum NOT NULL,
    "userId" character varying(32) NOT NULL,
    "userHost" character varying(128)
);


ALTER TABLE public.poll OWNER TO "example-misskey-user";

--
-- Name: COLUMN poll."noteVisibility"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.poll."noteVisibility" IS '[Denormalized]';


--
-- Name: COLUMN poll."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.poll."userId" IS '[Denormalized]';


--
-- Name: COLUMN poll."userHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.poll."userHost" IS '[Denormalized]';


--
-- Name: poll_vote; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.poll_vote (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL,
    choice integer NOT NULL
);


ALTER TABLE public.poll_vote OWNER TO "example-misskey-user";

--
-- Name: promo_note; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.promo_note (
    "noteId" character varying(32) NOT NULL,
    "expiresAt" timestamp with time zone NOT NULL,
    "userId" character varying(32) NOT NULL
);


ALTER TABLE public.promo_note OWNER TO "example-misskey-user";

--
-- Name: COLUMN promo_note."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.promo_note."userId" IS '[Denormalized]';


--
-- Name: promo_read; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.promo_read (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL
);


ALTER TABLE public.promo_read OWNER TO "example-misskey-user";

--
-- Name: registration_ticket; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.registration_ticket (
    id character varying(32) NOT NULL,
    code character varying(64) NOT NULL,
    "expiresAt" timestamp with time zone,
    "usedAt" timestamp with time zone,
    "pendingUserId" character varying(32),
    "createdById" character varying(32),
    "usedById" character varying(32)
);


ALTER TABLE public.registration_ticket OWNER TO "example-misskey-user";

--
-- Name: registry_item; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.registry_item (
    id character varying(32) NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL,
    "userId" character varying(32) NOT NULL,
    key character varying(1024) NOT NULL,
    scope character varying(1024)[] DEFAULT '{}'::character varying[] NOT NULL,
    domain character varying(512),
    value jsonb DEFAULT '{}'::jsonb
);


ALTER TABLE public.registry_item OWNER TO "example-misskey-user";

--
-- Name: COLUMN registry_item."updatedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.registry_item."updatedAt" IS 'The updated date of the RegistryItem.';


--
-- Name: COLUMN registry_item."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.registry_item."userId" IS 'The owner ID.';


--
-- Name: COLUMN registry_item.key; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.registry_item.key IS 'The key of the RegistryItem.';


--
-- Name: COLUMN registry_item.value; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.registry_item.value IS 'The value of the RegistryItem.';


--
-- Name: relay; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.relay (
    id character varying(32) NOT NULL,
    inbox character varying(512) NOT NULL,
    status public.relay_status_enum NOT NULL
);


ALTER TABLE public.relay OWNER TO "example-misskey-user";

--
-- Name: renote_muting; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.renote_muting (
    id character varying(32) NOT NULL,
    "muteeId" character varying(32) NOT NULL,
    "muterId" character varying(32) NOT NULL
);


ALTER TABLE public.renote_muting OWNER TO "example-misskey-user";

--
-- Name: COLUMN renote_muting."muteeId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.renote_muting."muteeId" IS 'The mutee user ID.';


--
-- Name: COLUMN renote_muting."muterId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.renote_muting."muterId" IS 'The muter user ID.';


--
-- Name: retention_aggregation; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.retention_aggregation (
    id character varying(32) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "userIds" character varying(32)[] NOT NULL,
    data jsonb DEFAULT '{}'::jsonb NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL,
    "usersCount" integer NOT NULL,
    "dateKey" character varying(512) NOT NULL
);


ALTER TABLE public.retention_aggregation OWNER TO "example-misskey-user";

--
-- Name: COLUMN retention_aggregation."createdAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.retention_aggregation."createdAt" IS 'The created date of the Note.';


--
-- Name: COLUMN retention_aggregation."updatedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.retention_aggregation."updatedAt" IS 'The updated date of the GalleryPost.';


--
-- Name: reversi_game; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.reversi_game (
    id character varying(32) NOT NULL,
    "startedAt" timestamp with time zone,
    "user1Id" character varying(32) NOT NULL,
    "user2Id" character varying(32) NOT NULL,
    "user1Ready" boolean DEFAULT false NOT NULL,
    "user2Ready" boolean DEFAULT false NOT NULL,
    black integer,
    "isStarted" boolean DEFAULT false NOT NULL,
    "isEnded" boolean DEFAULT false NOT NULL,
    "winnerId" character varying(32),
    "surrenderedUserId" character varying(32),
    logs jsonb DEFAULT '[]'::jsonb NOT NULL,
    map character varying(64)[] NOT NULL,
    bw character varying(32) NOT NULL,
    "isLlotheo" boolean DEFAULT false NOT NULL,
    "canPutEverywhere" boolean DEFAULT false NOT NULL,
    "loopedBoard" boolean DEFAULT false NOT NULL,
    form1 jsonb,
    form2 jsonb,
    crc32 character varying(32),
    "timeoutUserId" character varying(32),
    "endedAt" timestamp with time zone,
    "timeLimitForEachTurn" smallint DEFAULT '90'::smallint NOT NULL,
    "noIrregularRules" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.reversi_game OWNER TO "example-misskey-user";

--
-- Name: COLUMN reversi_game."startedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.reversi_game."startedAt" IS 'The started date of the ReversiGame.';


--
-- Name: COLUMN reversi_game."endedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.reversi_game."endedAt" IS 'The ended date of the ReversiGame.';


--
-- Name: reversi_matching; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.reversi_matching (
    id character varying(32) NOT NULL,
    "parentId" character varying(32) NOT NULL,
    "childId" character varying(32) NOT NULL
);


ALTER TABLE public.reversi_matching OWNER TO "example-misskey-user";

--
-- Name: role; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.role (
    id character varying(32) NOT NULL,
    "updatedAt" timestamp with time zone NOT NULL,
    name character varying(256) NOT NULL,
    description character varying(1024) NOT NULL,
    "isPublic" boolean DEFAULT false NOT NULL,
    "isModerator" boolean DEFAULT false NOT NULL,
    "isAdministrator" boolean DEFAULT false NOT NULL,
    policies jsonb DEFAULT '{}'::jsonb NOT NULL,
    color character varying(256),
    "canEditMembersByModerator" boolean DEFAULT false NOT NULL,
    "lastUsedAt" timestamp with time zone NOT NULL,
    target public.role_target_enum DEFAULT 'manual'::public.role_target_enum NOT NULL,
    "condFormula" jsonb DEFAULT '{}'::jsonb NOT NULL,
    "iconUrl" character varying(512),
    "asBadge" boolean DEFAULT false NOT NULL,
    "displayOrder" integer DEFAULT 0 NOT NULL,
    "isExplorable" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.role OWNER TO "example-misskey-user";

--
-- Name: COLUMN role."updatedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.role."updatedAt" IS 'The updated date of the Role.';


--
-- Name: COLUMN role."lastUsedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.role."lastUsedAt" IS 'The last used date of the Role.';


--
-- Name: role_assignment; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.role_assignment (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "roleId" character varying(32) NOT NULL,
    "expiresAt" timestamp with time zone
);


ALTER TABLE public.role_assignment OWNER TO "example-misskey-user";

--
-- Name: COLUMN role_assignment."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.role_assignment."userId" IS 'The user ID.';


--
-- Name: COLUMN role_assignment."roleId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.role_assignment."roleId" IS 'The role ID.';


--
-- Name: signin; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.signin (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    ip character varying(128) NOT NULL,
    headers jsonb NOT NULL,
    success boolean NOT NULL
);


ALTER TABLE public.signin OWNER TO "example-misskey-user";

--
-- Name: sw_subscription; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.sw_subscription (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    endpoint character varying(512) NOT NULL,
    auth character varying(256) NOT NULL,
    publickey character varying(128) NOT NULL,
    "sendReadMessage" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.sw_subscription OWNER TO "example-misskey-user";

--
-- Name: used_username; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.used_username (
    username character varying(128) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL
);


ALTER TABLE public.used_username OWNER TO "example-misskey-user";

--
-- Name: user; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public."user" (
    id character varying(32) NOT NULL,
    "updatedAt" timestamp with time zone,
    "lastFetchedAt" timestamp with time zone,
    username character varying(128) NOT NULL,
    "usernameLower" character varying(128) NOT NULL,
    name character varying(128),
    "followersCount" integer DEFAULT 0 NOT NULL,
    "followingCount" integer DEFAULT 0 NOT NULL,
    "notesCount" integer DEFAULT 0 NOT NULL,
    "avatarId" character varying(32),
    "bannerId" character varying(32),
    tags character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL,
    "isSuspended" boolean DEFAULT false NOT NULL,
    "isLocked" boolean DEFAULT false NOT NULL,
    "isBot" boolean DEFAULT false NOT NULL,
    "isCat" boolean DEFAULT false NOT NULL,
    "isRoot" boolean DEFAULT false NOT NULL,
    emojis character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL,
    host character varying(128),
    inbox character varying(512),
    "sharedInbox" character varying(512),
    featured character varying(512),
    uri character varying(512),
    token character(16),
    "isExplorable" boolean DEFAULT true NOT NULL,
    "followersUri" character varying(512),
    "lastActiveDate" timestamp with time zone,
    "hideOnlineStatus" boolean DEFAULT false NOT NULL,
    "isDeleted" boolean DEFAULT false NOT NULL,
    "avatarUrl" character varying(512),
    "bannerUrl" character varying(512),
    "avatarBlurhash" character varying(128),
    "bannerBlurhash" character varying(128),
    "movedToUri" character varying(512),
    "alsoKnownAs" text,
    "movedAt" timestamp with time zone,
    "isHibernated" boolean DEFAULT false NOT NULL,
    "avatarDecorations" jsonb DEFAULT '[]'::jsonb NOT NULL
);


ALTER TABLE public."user" OWNER TO "example-misskey-user";

--
-- Name: COLUMN "user"."updatedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."updatedAt" IS 'The updated date of the User.';


--
-- Name: COLUMN "user".username; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user".username IS 'The username of the User.';


--
-- Name: COLUMN "user"."usernameLower"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."usernameLower" IS 'The username (lowercased) of the User.';


--
-- Name: COLUMN "user".name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user".name IS 'The name of the User.';


--
-- Name: COLUMN "user"."followersCount"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."followersCount" IS 'The count of followers.';


--
-- Name: COLUMN "user"."followingCount"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."followingCount" IS 'The count of following.';


--
-- Name: COLUMN "user"."notesCount"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."notesCount" IS 'The count of notes.';


--
-- Name: COLUMN "user"."avatarId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."avatarId" IS 'The ID of avatar DriveFile.';


--
-- Name: COLUMN "user"."bannerId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."bannerId" IS 'The ID of banner DriveFile.';


--
-- Name: COLUMN "user"."isSuspended"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."isSuspended" IS 'Whether the User is suspended.';


--
-- Name: COLUMN "user"."isLocked"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."isLocked" IS 'Whether the User is locked.';


--
-- Name: COLUMN "user"."isBot"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."isBot" IS 'Whether the User is a bot.';


--
-- Name: COLUMN "user"."isCat"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."isCat" IS 'Whether the User is a cat.';


--
-- Name: COLUMN "user"."isRoot"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."isRoot" IS 'Whether the User is the root.';


--
-- Name: COLUMN "user".host; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user".host IS 'The host of the User. It will be null if the origin of the user is local.';


--
-- Name: COLUMN "user".inbox; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user".inbox IS 'The inbox URL of the User. It will be null if the origin of the user is local.';


--
-- Name: COLUMN "user"."sharedInbox"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."sharedInbox" IS 'The sharedInbox URL of the User. It will be null if the origin of the user is local.';


--
-- Name: COLUMN "user".featured; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user".featured IS 'The featured URL of the User. It will be null if the origin of the user is local.';


--
-- Name: COLUMN "user".uri; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user".uri IS 'The URI of the User. It will be null if the origin of the user is local.';


--
-- Name: COLUMN "user".token; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user".token IS 'The native access token of the User. It will be null if the origin of the user is local.';


--
-- Name: COLUMN "user"."isExplorable"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."isExplorable" IS 'Whether the User is explorable.';


--
-- Name: COLUMN "user"."followersUri"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."followersUri" IS 'The URI of the user Follower Collection. It will be null if the origin of the user is local.';


--
-- Name: COLUMN "user"."isDeleted"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."isDeleted" IS 'Whether the User is deleted.';


--
-- Name: COLUMN "user"."movedToUri"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."movedToUri" IS 'The URI of the new account of the User';


--
-- Name: COLUMN "user"."alsoKnownAs"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."alsoKnownAs" IS 'URIs the user is known as too';


--
-- Name: COLUMN "user"."movedAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public."user"."movedAt" IS 'When the user moved to another account';


--
-- Name: user_group; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_group (
    id character varying(32) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    name character varying(256) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "isPrivate" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.user_group OWNER TO "example-misskey-user";

--
-- Name: COLUMN user_group."createdAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_group."createdAt" IS 'The created date of the UserGroup.';


--
-- Name: COLUMN user_group."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_group."userId" IS 'The ID of owner.';


--
-- Name: user_group_invitation; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_group_invitation (
    id character varying(32) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "userId" character varying(32) NOT NULL,
    "userGroupId" character varying(32) NOT NULL
);


ALTER TABLE public.user_group_invitation OWNER TO "example-misskey-user";

--
-- Name: COLUMN user_group_invitation."createdAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_group_invitation."createdAt" IS 'The created date of the UserGroupInvitation.';


--
-- Name: COLUMN user_group_invitation."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_group_invitation."userId" IS 'The user ID.';


--
-- Name: COLUMN user_group_invitation."userGroupId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_group_invitation."userGroupId" IS 'The group ID.';


--
-- Name: user_group_invite; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_group_invite (
    id character varying(32) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "userId" character varying(32) NOT NULL,
    "userGroupId" character varying(32) NOT NULL
);


ALTER TABLE public.user_group_invite OWNER TO "example-misskey-user";

--
-- Name: user_group_joining; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_group_joining (
    id character varying(32) NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "userId" character varying(32) NOT NULL,
    "userGroupId" character varying(32) NOT NULL
);


ALTER TABLE public.user_group_joining OWNER TO "example-misskey-user";

--
-- Name: COLUMN user_group_joining."createdAt"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_group_joining."createdAt" IS 'The created date of the UserGroupJoining.';


--
-- Name: COLUMN user_group_joining."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_group_joining."userId" IS 'The user ID.';


--
-- Name: COLUMN user_group_joining."userGroupId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_group_joining."userGroupId" IS 'The group ID.';


--
-- Name: user_ip; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_ip (
    id integer NOT NULL,
    "createdAt" timestamp with time zone NOT NULL,
    "userId" character varying(32) NOT NULL,
    ip character varying(128) NOT NULL
);


ALTER TABLE public.user_ip OWNER TO "example-misskey-user";

--
-- Name: user_ip_id_seq; Type: SEQUENCE; Schema: public; Owner: example-misskey-user
--

CREATE SEQUENCE public.user_ip_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.user_ip_id_seq OWNER TO "example-misskey-user";

--
-- Name: user_ip_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: example-misskey-user
--

ALTER SEQUENCE public.user_ip_id_seq OWNED BY public.user_ip.id;


--
-- Name: user_keypair; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_keypair (
    "userId" character varying(32) NOT NULL,
    "publicKey" character varying(4096) NOT NULL,
    "privateKey" character varying(4096) NOT NULL
);


ALTER TABLE public.user_keypair OWNER TO "example-misskey-user";

--
-- Name: user_list; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_list (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    name character varying(128) NOT NULL,
    "isPublic" boolean DEFAULT false NOT NULL
);


ALTER TABLE public.user_list OWNER TO "example-misskey-user";

--
-- Name: COLUMN user_list."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_list."userId" IS 'The owner ID.';


--
-- Name: COLUMN user_list.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_list.name IS 'The name of the UserList.';


--
-- Name: user_list_favorite; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_list_favorite (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "userListId" character varying(32) NOT NULL
);


ALTER TABLE public.user_list_favorite OWNER TO "example-misskey-user";

--
-- Name: user_list_membership; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_list_membership (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "userListId" character varying(32) NOT NULL,
    "withReplies" boolean DEFAULT false NOT NULL,
    "userListUserId" character varying(32) NOT NULL
);


ALTER TABLE public.user_list_membership OWNER TO "example-misskey-user";

--
-- Name: COLUMN user_list_membership."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_list_membership."userId" IS 'The user ID.';


--
-- Name: COLUMN user_list_membership."userListId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_list_membership."userListId" IS 'The list ID.';


--
-- Name: user_memo; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_memo (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "targetUserId" character varying(32) NOT NULL,
    memo character varying(2048) NOT NULL
);


ALTER TABLE public.user_memo OWNER TO "example-misskey-user";

--
-- Name: COLUMN user_memo."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_memo."userId" IS 'The ID of author.';


--
-- Name: COLUMN user_memo."targetUserId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_memo."targetUserId" IS 'The ID of target user.';


--
-- Name: COLUMN user_memo.memo; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_memo.memo IS 'Memo.';


--
-- Name: user_note_pining; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_note_pining (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    "noteId" character varying(32) NOT NULL
);


ALTER TABLE public.user_note_pining OWNER TO "example-misskey-user";

--
-- Name: user_pending; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_pending (
    id character varying(32) NOT NULL,
    code character varying(128) NOT NULL,
    username character varying(128) NOT NULL,
    email character varying(128) NOT NULL,
    password character varying(128) NOT NULL
);


ALTER TABLE public.user_pending OWNER TO "example-misskey-user";

--
-- Name: user_profile; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_profile (
    "userId" character varying(32) NOT NULL,
    location character varying(128),
    birthday character(10),
    description character varying(2048),
    fields jsonb DEFAULT '[]'::jsonb NOT NULL,
    url character varying(512),
    email character varying(128),
    "emailVerifyCode" character varying(128),
    "emailVerified" boolean DEFAULT false NOT NULL,
    "twoFactorTempSecret" character varying(128),
    "twoFactorSecret" character varying(128),
    "twoFactorEnabled" boolean DEFAULT false NOT NULL,
    password character varying(128),
    "clientData" jsonb DEFAULT '{}'::jsonb NOT NULL,
    "autoAcceptFollowed" boolean DEFAULT false NOT NULL,
    "alwaysMarkNsfw" boolean DEFAULT false NOT NULL,
    "carefulBot" boolean DEFAULT false NOT NULL,
    "userHost" character varying(128),
    "securityKeysAvailable" boolean DEFAULT false NOT NULL,
    "usePasswordLessLogin" boolean DEFAULT false NOT NULL,
    "pinnedPageId" character varying(32),
    room jsonb DEFAULT '{}'::jsonb NOT NULL,
    "injectFeaturedNote" boolean DEFAULT true NOT NULL,
    "enableWordMute" boolean DEFAULT false NOT NULL,
    "mutedWords" jsonb DEFAULT '[]'::jsonb NOT NULL,
    "noCrawle" boolean DEFAULT false NOT NULL,
    "receiveAnnouncementEmail" boolean DEFAULT true NOT NULL,
    "emailNotificationTypes" jsonb DEFAULT '["follow", "receiveFollowRequest"]'::jsonb NOT NULL,
    lang character varying(32),
    "mutedInstances" jsonb DEFAULT '[]'::jsonb NOT NULL,
    "publicReactions" boolean DEFAULT true NOT NULL,
    "autoSensitive" boolean DEFAULT false NOT NULL,
    "moderationNote" character varying(8192) DEFAULT ''::character varying NOT NULL,
    achievements jsonb DEFAULT '[]'::jsonb NOT NULL,
    "loggedInDates" character varying(32)[] DEFAULT '{}'::character varying[] NOT NULL,
    "preventAiLearning" boolean DEFAULT true NOT NULL,
    "twoFactorBackupSecret" character varying[],
    "verifiedLinks" character varying[] DEFAULT '{}'::character varying[] NOT NULL,
    "notificationRecieveConfig" jsonb DEFAULT '{}'::jsonb NOT NULL,
    "hardMutedWords" jsonb DEFAULT '[]'::jsonb NOT NULL,
    "followingVisibility" public.user_profile_followingvisibility_enum DEFAULT 'public'::public.user_profile_followingvisibility_enum NOT NULL,
    "followersVisibility" public."user_profile_followersVisibility_enum" DEFAULT 'public'::public."user_profile_followersVisibility_enum" NOT NULL
);


ALTER TABLE public.user_profile OWNER TO "example-misskey-user";

--
-- Name: COLUMN user_profile.location; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile.location IS 'The location of the User.';


--
-- Name: COLUMN user_profile.birthday; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile.birthday IS 'The birthday (YYYY-MM-DD) of the User.';


--
-- Name: COLUMN user_profile.description; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile.description IS 'The description (bio) of the User.';


--
-- Name: COLUMN user_profile.url; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile.url IS 'Remote URL of the user.';


--
-- Name: COLUMN user_profile.email; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile.email IS 'The email address of the User.';


--
-- Name: COLUMN user_profile.password; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile.password IS 'The password hash of the User. It will be null if the origin of the user is local.';


--
-- Name: COLUMN user_profile."clientData"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile."clientData" IS 'The client-specific data of the User.';


--
-- Name: COLUMN user_profile."userHost"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile."userHost" IS '[Denormalized]';


--
-- Name: COLUMN user_profile.room; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile.room IS 'The room data of the User.';


--
-- Name: COLUMN user_profile."noCrawle"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile."noCrawle" IS 'Whether reject index by crawler.';


--
-- Name: COLUMN user_profile."mutedInstances"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_profile."mutedInstances" IS 'List of instances muted by the user.';


--
-- Name: user_publickey; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_publickey (
    "userId" character varying(32) NOT NULL,
    "keyId" character varying(256) NOT NULL,
    "keyPem" character varying(4096) NOT NULL
);


ALTER TABLE public.user_publickey OWNER TO "example-misskey-user";

--
-- Name: user_security_key; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.user_security_key (
    id character varying NOT NULL,
    "userId" character varying(32) NOT NULL,
    "publicKey" character varying NOT NULL,
    "lastUsed" timestamp with time zone DEFAULT now() NOT NULL,
    name character varying(30) NOT NULL,
    counter bigint DEFAULT '0'::bigint NOT NULL,
    "credentialDeviceType" character varying(32),
    "credentialBackedUp" boolean,
    transports character varying(32)[]
);


ALTER TABLE public.user_security_key OWNER TO "example-misskey-user";

--
-- Name: COLUMN user_security_key.id; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_security_key.id IS 'Variable-length id given to navigator.credentials.get()';


--
-- Name: COLUMN user_security_key."publicKey"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_security_key."publicKey" IS 'The public key of the UserSecurityKey, hex-encoded.';


--
-- Name: COLUMN user_security_key."lastUsed"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_security_key."lastUsed" IS 'Timestamp of the last time the UserSecurityKey was used.';


--
-- Name: COLUMN user_security_key.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_security_key.name IS 'User-defined name for this key';


--
-- Name: COLUMN user_security_key.counter; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_security_key.counter IS 'The number of times the UserSecurityKey was validated.';


--
-- Name: COLUMN user_security_key."credentialDeviceType"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_security_key."credentialDeviceType" IS 'The type of Backup Eligibility in authenticator data';


--
-- Name: COLUMN user_security_key."credentialBackedUp"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_security_key."credentialBackedUp" IS 'Whether or not the credential has been backed up';


--
-- Name: COLUMN user_security_key.transports; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.user_security_key.transports IS 'The type of the credential returned by the browser';


--
-- Name: webhook; Type: TABLE; Schema: public; Owner: example-misskey-user
--

CREATE TABLE public.webhook (
    id character varying(32) NOT NULL,
    "userId" character varying(32) NOT NULL,
    name character varying(128) NOT NULL,
    "on" character varying(128)[] DEFAULT '{}'::character varying[] NOT NULL,
    url character varying(1024) NOT NULL,
    secret character varying(1024) NOT NULL,
    active boolean DEFAULT true NOT NULL,
    "latestSentAt" timestamp with time zone,
    "latestStatus" integer
);


ALTER TABLE public.webhook OWNER TO "example-misskey-user";

--
-- Name: COLUMN webhook."userId"; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.webhook."userId" IS 'The owner ID.';


--
-- Name: COLUMN webhook.name; Type: COMMENT; Schema: public; Owner: example-misskey-user
--

COMMENT ON COLUMN public.webhook.name IS 'The name of the Antenna.';


--
-- Name: __chart__active_users id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__active_users ALTER COLUMN id SET DEFAULT nextval('public.__chart__active_users_id_seq'::regclass);


--
-- Name: __chart__ap_request id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__ap_request ALTER COLUMN id SET DEFAULT nextval('public.__chart__ap_request_id_seq'::regclass);


--
-- Name: __chart__drive id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__drive ALTER COLUMN id SET DEFAULT nextval('public.__chart__drive_id_seq'::regclass);


--
-- Name: __chart__federation id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__federation ALTER COLUMN id SET DEFAULT nextval('public.__chart__federation_id_seq'::regclass);


--
-- Name: __chart__hashtag id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__hashtag ALTER COLUMN id SET DEFAULT nextval('public.__chart__hashtag_id_seq'::regclass);


--
-- Name: __chart__instance id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__instance ALTER COLUMN id SET DEFAULT nextval('public.__chart__instance_id_seq'::regclass);


--
-- Name: __chart__network id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__network ALTER COLUMN id SET DEFAULT nextval('public.__chart__network_id_seq'::regclass);


--
-- Name: __chart__notes id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__notes ALTER COLUMN id SET DEFAULT nextval('public.__chart__notes_id_seq'::regclass);


--
-- Name: __chart__per_user_drive id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_drive ALTER COLUMN id SET DEFAULT nextval('public.__chart__per_user_drive_id_seq'::regclass);


--
-- Name: __chart__per_user_following id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_following ALTER COLUMN id SET DEFAULT nextval('public.__chart__per_user_following_id_seq'::regclass);


--
-- Name: __chart__per_user_notes id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_notes ALTER COLUMN id SET DEFAULT nextval('public.__chart__per_user_notes_id_seq'::regclass);


--
-- Name: __chart__per_user_pv id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_pv ALTER COLUMN id SET DEFAULT nextval('public.__chart__per_user_pv_id_seq'::regclass);


--
-- Name: __chart__per_user_reaction id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_reaction ALTER COLUMN id SET DEFAULT nextval('public.__chart__per_user_reaction_id_seq'::regclass);


--
-- Name: __chart__test id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__test ALTER COLUMN id SET DEFAULT nextval('public.__chart__test_id_seq'::regclass);


--
-- Name: __chart__test_grouped id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__test_grouped ALTER COLUMN id SET DEFAULT nextval('public.__chart__test_grouped_id_seq'::regclass);


--
-- Name: __chart__test_unique id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__test_unique ALTER COLUMN id SET DEFAULT nextval('public.__chart__test_unique_id_seq'::regclass);


--
-- Name: __chart__users id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__users ALTER COLUMN id SET DEFAULT nextval('public.__chart__users_id_seq'::regclass);


--
-- Name: __chart_day__active_users id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__active_users ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__active_users_id_seq'::regclass);


--
-- Name: __chart_day__ap_request id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__ap_request ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__ap_request_id_seq'::regclass);


--
-- Name: __chart_day__drive id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__drive ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__drive_id_seq'::regclass);


--
-- Name: __chart_day__federation id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__federation ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__federation_id_seq'::regclass);


--
-- Name: __chart_day__hashtag id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__hashtag ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__hashtag_id_seq'::regclass);


--
-- Name: __chart_day__instance id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__instance ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__instance_id_seq'::regclass);


--
-- Name: __chart_day__network id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__network ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__network_id_seq'::regclass);


--
-- Name: __chart_day__notes id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__notes ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__notes_id_seq'::regclass);


--
-- Name: __chart_day__per_user_drive id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_drive ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__per_user_drive_id_seq'::regclass);


--
-- Name: __chart_day__per_user_following id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_following ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__per_user_following_id_seq'::regclass);


--
-- Name: __chart_day__per_user_notes id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_notes ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__per_user_notes_id_seq'::regclass);


--
-- Name: __chart_day__per_user_pv id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_pv ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__per_user_pv_id_seq'::regclass);


--
-- Name: __chart_day__per_user_reaction id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_reaction ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__per_user_reaction_id_seq'::regclass);


--
-- Name: __chart_day__users id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__users ALTER COLUMN id SET DEFAULT nextval('public.__chart_day__users_id_seq'::regclass);


--
-- Name: migrations id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.migrations ALTER COLUMN id SET DEFAULT nextval('public.migrations_id_seq'::regclass);


--
-- Name: user_ip id; Type: DEFAULT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_ip ALTER COLUMN id SET DEFAULT nextval('public.user_ip_id_seq'::regclass);


--
-- Data for Name: __chart__active_users; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__active_users (id, date, "unique_temp___registeredWithinWeek", "___registeredWithinWeek", "unique_temp___registeredWithinMonth", "___registeredWithinMonth", "unique_temp___registeredWithinYear", "___registeredWithinYear", "unique_temp___registeredOutsideWeek", "___registeredOutsideWeek", "unique_temp___registeredOutsideMonth", "___registeredOutsideMonth", "unique_temp___registeredOutsideYear", "___registeredOutsideYear", "___readWrite", unique_temp___read, ___read, unique_temp___write, ___write) FROM stdin;
\.


--
-- Data for Name: __chart__ap_request; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__ap_request (id, date, "___deliverFailed", "___deliverSucceeded", "___inboxReceived") FROM stdin;
\.


--
-- Data for Name: __chart__drive; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__drive (id, date, "___local_incCount", "___local_incSize", "___local_decCount", "___local_decSize", "___remote_incCount", "___remote_incSize", "___remote_decCount", "___remote_decSize") FROM stdin;
\.


--
-- Data for Name: __chart__federation; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__federation (id, date, "unique_temp___deliveredInstances", "___deliveredInstances", "unique_temp___inboxInstances", "___inboxInstances", unique_temp___stalled, ___stalled, ___sub, ___pub, ___pubsub, "___subActive", "___pubActive") FROM stdin;
\.


--
-- Data for Name: __chart__hashtag; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__hashtag (id, date, "group", ___local_users, ___remote_users, unique_temp___local_users, unique_temp___remote_users) FROM stdin;
\.


--
-- Data for Name: __chart__instance; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__instance (id, date, "group", ___requests_failed, ___requests_succeeded, ___requests_received, ___notes_total, ___notes_inc, ___notes_dec, ___notes_diffs_normal, ___notes_diffs_reply, ___notes_diffs_renote, ___users_total, ___users_inc, ___users_dec, ___following_total, ___following_inc, ___following_dec, ___followers_total, ___followers_inc, ___followers_dec, "___drive_totalFiles", "___drive_incFiles", "___drive_incUsage", "___drive_decFiles", "___drive_decUsage", "___notes_diffs_withFile") FROM stdin;
\.


--
-- Data for Name: __chart__network; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__network (id, date, "___incomingRequests", "___outgoingRequests", "___totalTime", "___incomingBytes", "___outgoingBytes") FROM stdin;
\.


--
-- Data for Name: __chart__notes; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__notes (id, date, ___local_total, ___local_inc, ___local_dec, ___local_diffs_normal, ___local_diffs_reply, ___local_diffs_renote, ___remote_total, ___remote_inc, ___remote_dec, ___remote_diffs_normal, ___remote_diffs_reply, ___remote_diffs_renote, "___local_diffs_withFile", "___remote_diffs_withFile") FROM stdin;
\.


--
-- Data for Name: __chart__per_user_drive; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__per_user_drive (id, date, "group", "___totalCount", "___totalSize", "___incCount", "___incSize", "___decCount", "___decSize") FROM stdin;
\.


--
-- Data for Name: __chart__per_user_following; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__per_user_following (id, date, "group", ___local_followings_total, ___local_followings_inc, ___local_followings_dec, ___local_followers_total, ___local_followers_inc, ___local_followers_dec, ___remote_followings_total, ___remote_followings_inc, ___remote_followings_dec, ___remote_followers_total, ___remote_followers_inc, ___remote_followers_dec) FROM stdin;
\.


--
-- Data for Name: __chart__per_user_notes; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__per_user_notes (id, date, "group", ___total, ___inc, ___dec, ___diffs_normal, ___diffs_reply, ___diffs_renote, "___diffs_withFile") FROM stdin;
\.


--
-- Data for Name: __chart__per_user_pv; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__per_user_pv (id, date, "group", unique_temp___upv_user, ___upv_user, ___pv_user, unique_temp___upv_visitor, ___upv_visitor, ___pv_visitor) FROM stdin;
\.


--
-- Data for Name: __chart__per_user_reaction; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__per_user_reaction (id, date, "group", ___local_count, ___remote_count) FROM stdin;
\.


--
-- Data for Name: __chart__test; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__test (id, date, "group", ___foo_total, ___foo_inc, ___foo_dec) FROM stdin;
\.


--
-- Data for Name: __chart__test_grouped; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__test_grouped (id, date, "group", ___foo_total, ___foo_inc, ___foo_dec) FROM stdin;
\.


--
-- Data for Name: __chart__test_unique; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__test_unique (id, date, "group", ___foo) FROM stdin;
\.


--
-- Data for Name: __chart__users; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart__users (id, date, ___local_total, ___local_inc, ___local_dec, ___remote_total, ___remote_inc, ___remote_dec) FROM stdin;
\.


--
-- Data for Name: __chart_day__active_users; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__active_users (id, date, "unique_temp___registeredWithinWeek", "___registeredWithinWeek", "unique_temp___registeredWithinMonth", "___registeredWithinMonth", "unique_temp___registeredWithinYear", "___registeredWithinYear", "unique_temp___registeredOutsideWeek", "___registeredOutsideWeek", "unique_temp___registeredOutsideMonth", "___registeredOutsideMonth", "unique_temp___registeredOutsideYear", "___registeredOutsideYear", "___readWrite", unique_temp___read, ___read, unique_temp___write, ___write) FROM stdin;
\.


--
-- Data for Name: __chart_day__ap_request; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__ap_request (id, date, "___deliverFailed", "___deliverSucceeded", "___inboxReceived") FROM stdin;
\.


--
-- Data for Name: __chart_day__drive; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__drive (id, date, "___local_incCount", "___local_incSize", "___local_decCount", "___local_decSize", "___remote_incCount", "___remote_incSize", "___remote_decCount", "___remote_decSize") FROM stdin;
\.


--
-- Data for Name: __chart_day__federation; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__federation (id, date, "unique_temp___deliveredInstances", "___deliveredInstances", "unique_temp___inboxInstances", "___inboxInstances", unique_temp___stalled, ___stalled, ___sub, ___pub, ___pubsub, "___subActive", "___pubActive") FROM stdin;
\.


--
-- Data for Name: __chart_day__hashtag; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__hashtag (id, date, "group", ___local_users, ___remote_users, unique_temp___local_users, unique_temp___remote_users) FROM stdin;
\.


--
-- Data for Name: __chart_day__instance; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__instance (id, date, "group", ___requests_failed, ___requests_succeeded, ___requests_received, ___notes_total, ___notes_inc, ___notes_dec, ___notes_diffs_normal, ___notes_diffs_reply, ___notes_diffs_renote, ___users_total, ___users_inc, ___users_dec, ___following_total, ___following_inc, ___following_dec, ___followers_total, ___followers_inc, ___followers_dec, "___drive_totalFiles", "___drive_incFiles", "___drive_incUsage", "___drive_decFiles", "___drive_decUsage", "___notes_diffs_withFile") FROM stdin;
\.


--
-- Data for Name: __chart_day__network; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__network (id, date, "___incomingRequests", "___outgoingRequests", "___totalTime", "___incomingBytes", "___outgoingBytes") FROM stdin;
\.


--
-- Data for Name: __chart_day__notes; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__notes (id, date, ___local_total, ___local_inc, ___local_dec, ___local_diffs_normal, ___local_diffs_reply, ___local_diffs_renote, ___remote_total, ___remote_inc, ___remote_dec, ___remote_diffs_normal, ___remote_diffs_reply, ___remote_diffs_renote, "___local_diffs_withFile", "___remote_diffs_withFile") FROM stdin;
\.


--
-- Data for Name: __chart_day__per_user_drive; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__per_user_drive (id, date, "group", "___totalCount", "___totalSize", "___incCount", "___incSize", "___decCount", "___decSize") FROM stdin;
\.


--
-- Data for Name: __chart_day__per_user_following; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__per_user_following (id, date, "group", ___local_followings_total, ___local_followings_inc, ___local_followings_dec, ___local_followers_total, ___local_followers_inc, ___local_followers_dec, ___remote_followings_total, ___remote_followings_inc, ___remote_followings_dec, ___remote_followers_total, ___remote_followers_inc, ___remote_followers_dec) FROM stdin;
\.


--
-- Data for Name: __chart_day__per_user_notes; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__per_user_notes (id, date, "group", ___total, ___inc, ___dec, ___diffs_normal, ___diffs_reply, ___diffs_renote, "___diffs_withFile") FROM stdin;
\.


--
-- Data for Name: __chart_day__per_user_pv; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__per_user_pv (id, date, "group", unique_temp___upv_user, ___upv_user, ___pv_user, unique_temp___upv_visitor, ___upv_visitor, ___pv_visitor) FROM stdin;
\.


--
-- Data for Name: __chart_day__per_user_reaction; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__per_user_reaction (id, date, "group", ___local_count, ___remote_count) FROM stdin;
\.


--
-- Data for Name: __chart_day__users; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.__chart_day__users (id, date, ___local_total, ___local_inc, ___local_dec, ___remote_total, ___remote_inc, ___remote_dec) FROM stdin;
\.


--
-- Data for Name: abuse_user_report; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.abuse_user_report (id, "targetUserId", "reporterId", "assigneeId", resolved, comment, "targetUserHost", "reporterHost", forwarded) FROM stdin;
\.


--
-- Data for Name: access_token; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.access_token (id, token, hash, "userId", "appId", "lastUsedAt", session, name, description, "iconUrl", permission, fetched) FROM stdin;
\.


--
-- Data for Name: ad; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.ad (id, "expiresAt", place, priority, url, "imageUrl", memo, ratio, "startsAt", "dayOfWeek") FROM stdin;
\.


--
-- Data for Name: announcement; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.announcement (id, text, title, "imageUrl", "updatedAt", display, "needConfirmationToRead", "isActive", "forExistingUsers", "userId", icon, silence) FROM stdin;
\.


--
-- Data for Name: announcement_read; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.announcement_read (id, "userId", "announcementId") FROM stdin;
\.


--
-- Data for Name: antenna; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.antenna (id, "userId", name, src, "userListId", keywords, "withFile", expression, notify, "caseSensitive", "withReplies", users, "excludeKeywords", "lastUsedAt", "isActive", "localOnly") FROM stdin;
\.


--
-- Data for Name: app; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.app (id, "userId", secret, name, description, permission, "callbackUrl") FROM stdin;
\.


--
-- Data for Name: auth_session; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.auth_session (id, token, "userId", "appId") FROM stdin;
\.


--
-- Data for Name: avatar_decoration; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.avatar_decoration (id, "updatedAt", url, name, description, "roleIdsThatCanBeUsedThisDecoration") FROM stdin;
\.


--
-- Data for Name: blocking; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.blocking (id, "blockeeId", "blockerId") FROM stdin;
\.


--
-- Data for Name: bubble_game_record; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.bubble_game_record (id, "userId", "seededAt", seed, "gameVersion", "gameMode", score, logs, "isVerified") FROM stdin;
\.


--
-- Data for Name: channel; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.channel (id, "lastNotedAt", "userId", name, description, "bannerId", "notesCount", "usersCount", "pinnedNoteIds", color, "isArchived", "isSensitive", "allowRenoteToExternal") FROM stdin;
\.


--
-- Data for Name: channel_favorite; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.channel_favorite (id, "channelId", "userId") FROM stdin;
\.


--
-- Data for Name: channel_following; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.channel_following (id, "followeeId", "followerId") FROM stdin;
\.


--
-- Data for Name: channel_note_pining; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.channel_note_pining (id, "createdAt", "channelId", "noteId") FROM stdin;
\.


--
-- Data for Name: clip; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.clip (id, "userId", name, "isPublic", description, "lastClippedAt") FROM stdin;
\.


--
-- Data for Name: clip_favorite; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.clip_favorite (id, "userId", "clipId") FROM stdin;
\.


--
-- Data for Name: clip_note; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.clip_note (id, "noteId", "clipId") FROM stdin;
\.


--
-- Data for Name: drive_file; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.drive_file (id, "userId", "userHost", md5, name, type, size, comment, properties, "storedInternal", url, "thumbnailUrl", "webpublicUrl", "accessKey", "thumbnailAccessKey", "webpublicAccessKey", uri, src, "folderId", "isSensitive", "isLink", blurhash, "webpublicType", "requestHeaders", "requestIp", "maybeSensitive", "maybePorn") FROM stdin;
\.


--
-- Data for Name: drive_folder; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.drive_folder (id, name, "userId", "parentId") FROM stdin;
\.


--
-- Data for Name: emoji; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.emoji (id, "updatedAt", name, host, "originalUrl", uri, type, aliases, category, "publicUrl", license, "localOnly", "isSensitive", "roleIdsThatCanBeUsedThisEmojiAsReaction") FROM stdin;
\.


--
-- Data for Name: flash; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.flash (id, "updatedAt", title, summary, "userId", script, permissions, "likedCount", visibility) FROM stdin;
\.


--
-- Data for Name: flash_like; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.flash_like (id, "userId", "flashId") FROM stdin;
\.


--
-- Data for Name: follow_request; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.follow_request (id, "followeeId", "followerId", "requestId", "followerHost", "followerInbox", "followerSharedInbox", "followeeHost", "followeeInbox", "followeeSharedInbox", "withReplies") FROM stdin;
\.


--
-- Data for Name: following; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.following (id, "followeeId", "followerId", "followerHost", "followerInbox", "followerSharedInbox", "followeeHost", "followeeInbox", "followeeSharedInbox", notify, "withReplies", "isFollowerHibernated") FROM stdin;
\.


--
-- Data for Name: gallery_like; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.gallery_like (id, "userId", "postId") FROM stdin;
\.


--
-- Data for Name: gallery_post; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.gallery_post (id, "updatedAt", title, description, "userId", "fileIds", "isSensitive", "likedCount", tags) FROM stdin;
\.


--
-- Data for Name: hashtag; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.hashtag (id, name, "mentionedUserIds", "mentionedUsersCount", "mentionedLocalUserIds", "mentionedLocalUsersCount", "mentionedRemoteUserIds", "mentionedRemoteUsersCount", "attachedUserIds", "attachedUsersCount", "attachedLocalUserIds", "attachedLocalUsersCount", "attachedRemoteUserIds", "attachedRemoteUsersCount") FROM stdin;
\.


--
-- Data for Name: instance; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.instance (id, "firstRetrievedAt", host, "usersCount", "notesCount", "followingCount", "followersCount", "latestRequestReceivedAt", "isNotResponding", "softwareName", "softwareVersion", "openRegistrations", name, description, "maintainerName", "maintainerEmail", "infoUpdatedAt", "isSuspended", "iconUrl", "themeColor", "faviconUrl", "moderationNote") FROM stdin;
\.


--
-- Data for Name: messaging_message; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.messaging_message (id, "createdAt", "userId", "recipientId", text, "isRead", "fileId", "groupId", reads, uri) FROM stdin;
\.


--
-- Data for Name: meta; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.meta (id, name, description, "maintainerName", "maintainerEmail", "disableRegistration", langs, "hiddenTags", "blockedHosts", "mascotImageUrl", "bannerUrl", "iconUrl", "cacheRemoteFiles", "enableRecaptcha", "recaptchaSiteKey", "recaptchaSecretKey", "summalyProxy", "enableEmail", email, "smtpSecure", "smtpHost", "smtpPort", "smtpUser", "smtpPass", "enableServiceWorker", "swPublicKey", "swPrivateKey", "pinnedUsers", "termsOfServiceUrl", "repositoryUrl", "feedbackUrl", "useObjectStorage", "objectStorageBucket", "objectStoragePrefix", "objectStorageBaseUrl", "objectStorageEndpoint", "objectStorageRegion", "objectStorageAccessKey", "objectStorageSecretKey", "objectStoragePort", "objectStorageUseSSL", "proxyAccountId", "objectStorageUseProxy", "enableHcaptcha", "hcaptchaSiteKey", "hcaptchaSecretKey", "objectStorageSetPublicRead", "backgroundImageUrl", "logoImageUrl", "objectStorageS3ForcePathStyle", "deeplAuthKey", "deeplIsPro", "emailRequiredForSignup", "themeColor", "defaultLightTheme", "defaultDarkTheme", "sensitiveMediaDetection", "sensitiveMediaDetectionSensitivity", "setSensitiveFlagAutomatically", "enableIpLogging", "enableSensitiveMediaDetectionForVideos", "enableActiveEmailValidation", "enableTurnstile", "turnstileSiteKey", "turnstileSecretKey", policies, "sensitiveWords", "enableChartsForRemoteUser", "enableChartsForFederatedInstances", "serverRules", "preservedUsernames", "serverErrorImageUrl", "notFoundImageUrl", "infoImageUrl", "enableServerMachineStats", "enableIdenticonGeneration", "cacheRemoteSensitiveFiles", "app192IconUrl", "app512IconUrl", "manifestJsonOverride", "shortName", "impressumUrl", "privacyPolicyUrl", "perLocalUserUserTimelineCacheMax", "perRemoteUserUserTimelineCacheMax", "perUserHomeTimelineCacheMax", "perUserListTimelineCacheMax", "notesPerOneAd", "silencedHosts", "enableFanoutTimeline", "enableFanoutTimelineDbFallback", "verifymailAuthKey", "enableVerifymailApi", "bannedEmailDomains", "truemailInstance", "truemailAuthKey", "enableTruemailApi", "enableMcaptcha", "mcaptchaSitekey", "mcaptchaSecretKey", "mcaptchaInstanceUrl", "prohibitedWords") FROM stdin;
x	\N	\N	\N	\N	t	{}	{}	{}	\N	\N	\N	f	f	\N	\N	\N	f	\N	f	\N	\N	\N	\N	f	\N	\N	{}	\N	https://github.com/misskey-dev/misskey	https://github.com/misskey-dev/misskey/issues/new	f	\N	\N	\N	\N	\N	\N	\N	\N	t	\N	t	f	\N	\N	f	\N	\N	t	\N	f	f	\N	\N	\N	none	medium	f	f	f	t	f	\N	\N	{}	{}	t	t	{}	{admin,administrator,root,system,maintainer,host,mod,moderator,owner,superuser,staff,auth,i,me,everyone,all,mention,mentions,example,user,users,account,accounts,official,help,helps,support,supports,info,information,informations,announce,announces,announcement,announcements,notice,notification,notifications,dev,developer,developers,tech,misskey}	\N	\N	\N	f	t	t	\N	\N	{}	\N	\N	\N	300	100	300	300	0	{}	t	t	\N	f	{}	\N	\N	f	f	\N	\N	\N	{}
\.


--
-- Data for Name: migrations; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.migrations (id, "timestamp", name) FROM stdin;
1	1000000000000	Init1000000000000
2	1556348509290	Pages1556348509290
3	1556746559567	UserProfile1556746559567
4	1557476068003	PinnedUsers1557476068003
5	1557761316509	AddSomeUrls1557761316509
6	1557932705754	ObjectStorageSetting1557932705754
7	1558072954435	PageLike1558072954435
8	1558103093633	UserGroup1558103093633
9	1558257926829	UserGroupInvite1558257926829
10	1558266512381	UserListJoining1558266512381
11	1561706992953	webauthn1561706992953
12	1561873850023	ChartIndexes1561873850023
13	1562422242907	PasswordLessLogin1562422242907
14	1562444565093	PinnedPage1562444565093
15	1562448332510	PageTitleHideOption1562448332510
16	1562869971568	ModerationLog1562869971568
17	1563757595828	UsedUsername1563757595828
18	1565634203341	room1565634203341
19	1571220798684	CustomEmojiCategory1571220798684
20	1572760203493	nodeinfo1572760203493
21	1576269851876	TalkFederationId1576269851876
22	1576869585998	ProxyRemoteFiles1576869585998
23	1579267006611	v121579267006611
24	1579270193251	v1221579270193251
25	1579282808087	v1231579282808087
26	1579544426412	v1241579544426412
27	1579977526288	v1251579977526288
28	1579993013959	v1261579993013959
29	1580069531114	v1271580069531114
30	1580148575182	v1281580148575182
31	1580154400017	v1291580154400017
32	1580276619901	v12101580276619901
33	1580331224276	v12111580331224276
34	1580508795118	v12121580508795118
35	1580543501339	v12131580543501339
36	1580864313253	v12141580864313253
37	1581526429287	userGroupInvitation1581526429287
38	1581695816408	userGroupAntenna1581695816408
39	1581708415836	driveUserFolderIdIndex1581708415836
40	1581979837262	promo1581979837262
41	1582019042083	featuredInjecttion1582019042083
42	1582210532752	antennaExclude1582210532752
43	1582875306439	noteReactionLength1582875306439
44	1585361548360	miauth1585361548360
45	1585385921215	customNotification1585385921215
46	1585772678853	apUrl1585772678853
47	1586624197029	AddObjectStorageUseProxy1586624197029
48	1586641139527	remoteReaction1586641139527
49	1586708940386	pageAiScript1586708940386
50	1588044505511	hCaptcha1588044505511
51	1589023282116	pubRelay1589023282116
52	1595075960584	blurhash1595075960584
53	1595077605646	blurhashForAvatarBanner1595077605646
54	1595676934834	instanceIconUrl1595676934834
55	1595771249699	wordMute1595771249699
56	1595782306083	wordMute21595782306083
57	1596548170836	channel1596548170836
58	1596786425167	channel21596786425167
59	1597230137744	objectStorageSetPublicRead1597230137744
60	1597236229720	IncludingNotificationTypes1597236229720
61	1597385880794	addSensitiveIndex1597385880794
62	1597459042300	channelUnread1597459042300
63	1597893996136	ChannelNoteIdDescIndex1597893996136
64	1600353287890	mutingNotificationTypes1600353287890
65	1603094348345	refineAbuseUserReport1603094348345
66	1603095701770	refineAbuseUserReport21603095701770
67	1603776877564	instanceThemeColor1603776877564
68	1603781553011	instanceFavicon1603781553011
69	1604821689616	deleteAutoWatch1604821689616
70	1605408848373	clipDescription1605408848373
71	1605408971051	comments1605408971051
72	1605585339718	instancePinnedPages1605585339718
73	1605965516823	instanceImages1605965516823
74	1606191203881	noCrawle1606191203881
75	1607151207216	instancePinnedClip1607151207216
76	1607353487793	isExplorable1607353487793
77	1610277136869	registry1610277136869
78	1610277585759	registry21610277585759
79	1610283021566	registry31610283021566
80	1611354329133	followersUri1611354329133
81	1611397665007	gallery1611397665007
82	1611547387175	objectStorageS3ForcePathStyle1611547387175
83	1612619156584	announcementEmail1612619156584
84	1613155914446	emailNotificationTypes1613155914446
85	1613181457597	userLang1613181457597
86	1613503367223	useBigintForDriveUsage1613503367223
87	1615965918224	chartV21615965918224
88	1615966519402	chartV221615966519402
89	1618637372000	userLastActiveDate1618637372000
90	1618639857000	userHideOnlineStatus1618639857000
91	1619942102890	passwordReset1619942102890
92	1620019354680	ad1620019354680
93	1620364649428	ad21620364649428
94	1621479946000	addNoteIndexes1621479946000
95	1622679304522	userProfileDescriptionLength1622679304522
96	1622681548499	logMessageLength1622681548499
97	1626509500668	fixRemoteFileProxy1626509500668
98	1629004542760	chartReindex1629004542760
99	1629024377804	deeplIntegration1629024377804
100	1629288472000	fixChannelUserId1629288472000
101	1629512953000	isUserDeleted1629512953000
102	1629778475000	deeplIntegration21629778475000
103	1629833361000	addShowTLReplies1629833361000
104	1629968054000	userInstanceBlocks1629968054000
105	1633068642000	emailRequiredForSignup1633068642000
106	1633071909016	userPending1633071909016
107	1634486652000	userPublicReactions1634486652000
108	1634902659689	deleteLog1634902659689
109	1635500777168	noteThreadMute1635500777168
110	1636197624383	ffVisibility1636197624383
111	1636697408073	removeViaMobile1636697408073
112	1637320813000	forwardedReport1637320813000
113	1639325650583	chartV31639325650583
114	1642611822809	emojiUrl1642611822809
115	1642613870898	driveFileWebpublicType1642613870898
116	1643963705770	chartV41643963705770
117	1643966656277	chartV51643966656277
118	1643967331284	chartV61643967331284
119	1644010796173	convertHardMutes1644010796173
120	1644058404077	chartV71644058404077
121	1644059847460	chartV81644059847460
122	1644060125705	chartV91644060125705
123	1644073149413	chartV101644073149413
124	1644095659741	chartV111644095659741
125	1644328606241	chartV121644328606241
126	1644331238153	chartV131644331238153
127	1644344266289	chartV141644344266289
128	1644395759931	instanceThemeColor1644395759931
129	1644481657998	chartV151644481657998
130	1644551208096	followingIndexes1644551208096
131	1645340161439	removeMaxNoteTextLength1645340161439
132	1645599900873	federationChartPubsub1645599900873
133	1646143552768	instanceDefaultTheme1646143552768
134	1646387162108	muteExpiresAt1646387162108
135	1646549089451	pollEndedNotification1646549089451
136	1646633030285	chartFederationActive1646633030285
137	1646655454495	removeInstanceDriveColumns1646655454495
138	1646732390560	chartFederationActiveSubPub1646732390560
139	1648548247382	webhook1648548247382
140	1648816172177	webhook21648816172177
141	1651224615271	foreignKeyReports1651224615271
142	1652859567549	uniformThemecolor1652859567549
143	1655368940105	nsfwDetection1655368940105
144	1655371960534	nsfwDetection21655371960534
145	1655388169582	nsfwDetection31655388169582
146	1655393015659	nsfwDetection41655393015659
147	1655813815729	driveCapacityOverrideMb1655813815729
148	1655918165614	userIp1655918165614
149	1656122560740	fileIp1656122560740
150	1656251734807	nsfwDetection51656251734807
151	1656328812281	ip21656328812281
152	1656408772602	nsfwDetection61656408772602
153	1656772790599	userModerationNote1656772790599
154	1657346559800	activeEmailValidation1657346559800
155	1664694635394	turnstile1664694635394
156	1665091090561	addRenoteMuting1665091090561
157	1669138716634	whetherPushNotifyToSendReadMessage1669138716634
158	1671924750884	RetentionAggregation1671924750884
159	1671926422832	RetentionAggregation21671926422832
160	1672562400597	PerUserPvChart1672562400597
161	1672703171386	removeLatestRequestSentAt1672703171386
162	1672704017999	removeLastCommunicatedAt1672704017999
163	1672704136584	removeLatestStatus1672704136584
164	1672822262496	Flash1672822262496
165	1673336077243	PollChoiceLength1673336077243
166	1673500412259	Role1673500412259
167	1673515526953	RoleColor1673515526953
168	1673522856499	RoleIroiro1673522856499
169	1673524604156	RoleLastUsedAt1673524604156
170	1673570377815	RoleConditional1673570377815
171	1673575973645	MetaClean1673575973645
172	1673783015567	Policies1673783015567
173	1673812883772	firstRetrievedAt1673812883772
174	1674086433654	flashScriptLength1674086433654
175	1674118260469	achievement1674118260469
176	1674255666603	loggedInDates1674255666603
177	1675053125067	fixforeignkeyreports1675053125067
178	1675404035646	cleanup1675404035646
179	1675557528704	roleIconBadge1675557528704
180	1676434944993	dropGroup1676434944993
181	1676438468213	ad1676438468213
182	1677054292210	ad1677054292210
183	1677570181236	roleAssignmentExpiresAt1677570181236
184	1678164627293	perNoteReactionAcceptance1678164627293
185	1678426061773	tweakVarcharLength1678426061773
186	1678427401214	removeUnused1678427401214
187	1678602320354	roleDisplayOrder1678602320354
188	1678694614599	sensitiveWords1678694614599
189	1678869617549	retentionDateKey1678869617549
190	1678945242650	addPropsForCustomEmoji1678945242650
191	1678953978856	clipFavorite1678953978856
192	1679309757174	antennaActive1679309757174
193	1679639483253	enableChartsForRemoteUser1679639483253
194	1679651580149	cleanup1679651580149
195	1679652081809	enableChartsForFederatedInstances1679652081809
196	1680228513388	channelFavorite1680228513388
197	1680238118084	channelNotePining1680238118084
198	1680491187535	cleanup1680491187535
199	1680582195041	cleanup1680582195041
200	1680702787050	UserMemo1680702787050
201	1680775031481	AvatarUrlAndBannerUrl1680775031481
202	1680931179228	AccountMove1680931179228
203	1681400427971	ServerRules1681400427971
204	1681870960239	RoleTLSetting1681870960239
205	1682190963894	MovedAt1682190963894
206	1682754135458	PreservedUsernames1682754135458
207	1682985520254	ChannelColor1682985520254
208	1683328299359	ChannelArchive1683328299359
209	1683682889948	PreventAiLarning1683682889948
210	1683683083083	PublicReactionsDefaultTrue1683683083083
211	1683789676867	FixTypo1683789676867
212	1683847157541	UserList1683847157541
213	1683869758873	UserListFavorites1683869758873
214	1684206886988	RemoveShowTimelineReplies1684206886988
215	1684386446061	EmojiImprove1684386446061
216	1685973839966	ErrorImageUrl1685973839966
217	1688280713783	AddMetaOptions1688280713783
218	1688720440658	RefactorInviteSystem1688720440658
219	1688880985544	AddIndexToRelations1688880985544
220	1689102832143	NsfwCache1689102832143
221	1689325027964	UserBlacklistAnntena1689325027964
222	1690417561185	FixRenoteMuting1690417561185
223	1690417561186	ChangeCacheRemoteFilesDefault1690417561186
224	1690417561187	Fix1690417561187
225	1690569881926	User2faBackupCodes1690569881926
226	1690782653311	SensitiveChannel1690782653311
227	1690796169261	PlayVisibility1690796169261
228	1691649257651	RefineAnnouncement1691649257651
229	1691657412740	RefineAnnouncement21691657412740
230	1691959191872	PasskeySupport1691959191872
231	1694850832075	ServerIconsAndManifest1694850832075
232	1694915420864	ClippedCount1694915420864
233	1695260774117	VerifiedLinks1695260774117
234	1695288787870	FollowingNotify1695288787870
235	1695440131671	ShortName1695440131671
236	1695605508898	MutingNotificationTypes1695605508898
237	1695901659683	NoteUpdatedAt1695901659683
238	1695944637565	NotificationRecieveConfig1695944637565
239	1696003580220	AddSomeUrls1696003580220
240	1696222183852	WithReplies1696222183852
241	1696323464251	UserListMembership1696323464251
242	1696331570827	Hibernation1696331570827
243	1696332072038	Clean1696332072038
244	1696373953614	MetaCacheSettings1696373953614
245	1696388600237	RevertNoteEdit1696388600237
246	1696405744672	CleanUp1696405744672
247	1696569742153	CleanUp1696569742153
248	1696581429196	CleanUp1696581429196
249	1696743032098	AdsOnStream1696743032098
250	1696807733453	UserListUserId1696807733453
251	1696808725134	UserListUserId21696808725134
252	1697247230117	InstanceSilence1697247230117
253	1697420555911	DeleteCreatedAt1697420555911
254	1697436246389	AntennaLocalOnly1697436246389
255	1697441463087	FollowRequestWithReplies1697441463087
256	1697673894459	NoteReactionAndUserPairCache1697673894459
257	1697847397844	AvatarDecoration1697847397844
258	1697941908548	AvatarDecoration21697941908548
259	1698041201306	EnableFtt1698041201306
260	1698840138000	AddAllowRenoteToExternal1698840138000
261	1699141698112	AnnouncementSilence1699141698112
262	1700096812223	EnableFanoutTimelineDbFallback1700096812223
263	1700303245007	SupportVerifyMailApi1700303245007
264	1700383825690	HardMute1700383825690
265	1700902349231	AddBdayIndex1700902349231
266	1702718871541	ffVisibility1702718871541
267	1703209889304	bannedEmailDomains1703209889304
268	1703658526000	SupportTrueMailApi1703658526000
269	1704373210054	SupportMcaptcha1704373210054
270	1704959805077	BubbleGameRecord1704959805077
271	1705222772858	OptimizeNoteIndexForArrayColumns1705222772858
272	1705475608437	Reversi1705475608437
273	1705654039457	Reversi21705654039457
274	1705793785675	Reversi31705793785675
275	1705794768153	Reversi41705794768153
276	1705798904141	Reversi51705798904141
277	1706081514499	Reversi61706081514499
278	1706791962000	FixMetaDisableRegistration1706791962000
279	1707429690000	prohibitedWords1707429690000
280	1707808106310	MakeRepositoryUrlNullable1707808106310
281	1708266695091	RepositoryUrlFromSyuiloToMisskeyDev1708266695091
282	1708399372194	PerInstanceModNote1708399372194
\.


--
-- Data for Name: moderation_log; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.moderation_log (id, "userId", type, info) FROM stdin;
\.


--
-- Data for Name: muting; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.muting (id, "muteeId", "muterId", "expiresAt") FROM stdin;
\.


--
-- Data for Name: note; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.note (id, "replyId", "renoteId", text, name, cw, "userId", "localOnly", "renoteCount", "repliesCount", reactions, visibility, uri, "fileIds", "attachedFileTypes", "visibleUserIds", mentions, "mentionedRemoteUsers", emojis, tags, "hasPoll", "userHost", "replyUserId", "replyUserHost", "renoteUserId", "renoteUserHost", url, "channelId", "threadId", "reactionAcceptance", "clippedCount", "reactionAndUserPairCache") FROM stdin;
\.


--
-- Data for Name: note_favorite; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.note_favorite (id, "userId", "noteId") FROM stdin;
\.


--
-- Data for Name: note_reaction; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.note_reaction (id, "userId", "noteId", reaction) FROM stdin;
\.


--
-- Data for Name: note_thread_muting; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.note_thread_muting (id, "userId", "threadId") FROM stdin;
\.


--
-- Data for Name: note_unread; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.note_unread (id, "userId", "noteId", "noteUserId", "isSpecified", "isMentioned", "noteChannelId") FROM stdin;
\.


--
-- Data for Name: note_watching; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.note_watching (id, "createdAt", "userId", "noteId", "noteUserId") FROM stdin;
\.


--
-- Data for Name: page; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.page (id, "updatedAt", title, name, summary, "alignCenter", font, "userId", "eyeCatchingImageId", content, variables, visibility, "visibleUserIds", "likedCount", "hideTitleWhenPinned", script) FROM stdin;
\.


--
-- Data for Name: page_like; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.page_like (id, "userId", "pageId") FROM stdin;
\.


--
-- Data for Name: password_reset_request; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.password_reset_request (id, token, "userId") FROM stdin;
\.


--
-- Data for Name: poll; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.poll ("noteId", "expiresAt", multiple, choices, votes, "noteVisibility", "userId", "userHost") FROM stdin;
\.


--
-- Data for Name: poll_vote; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.poll_vote (id, "userId", "noteId", choice) FROM stdin;
\.


--
-- Data for Name: promo_note; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.promo_note ("noteId", "expiresAt", "userId") FROM stdin;
\.


--
-- Data for Name: promo_read; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.promo_read (id, "userId", "noteId") FROM stdin;
\.


--
-- Data for Name: registration_ticket; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.registration_ticket (id, code, "expiresAt", "usedAt", "pendingUserId", "createdById", "usedById") FROM stdin;
\.


--
-- Data for Name: registry_item; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.registry_item (id, "updatedAt", "userId", key, scope, domain, value) FROM stdin;
9r70xhzd0mav0002	2024-03-23 02:17:06.121+00	9r70xhde0mav0001	widgets	{client,base}	\N	[{"id": "a", "data": {}, "name": "calendar", "place": "right"}, {"id": "b", "data": {}, "name": "notifications", "place": "right"}, {"id": "c", "data": {}, "name": "trends", "place": "right"}]
9r70xitk0mav0003	2024-03-23 02:17:23.396+00	9r70xhde0mav0001	accountSetupWizard	{client,base}	\N	-1
\.


--
-- Data for Name: relay; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.relay (id, inbox, status) FROM stdin;
\.


--
-- Data for Name: renote_muting; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.renote_muting (id, "muteeId", "muterId") FROM stdin;
\.


--
-- Data for Name: retention_aggregation; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.retention_aggregation (id, "createdAt", "userIds", data, "updatedAt", "usersCount", "dateKey") FROM stdin;
\.


--
-- Data for Name: reversi_game; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.reversi_game (id, "startedAt", "user1Id", "user2Id", "user1Ready", "user2Ready", black, "isStarted", "isEnded", "winnerId", "surrenderedUserId", logs, map, bw, "isLlotheo", "canPutEverywhere", "loopedBoard", form1, form2, crc32, "timeoutUserId", "endedAt", "timeLimitForEachTurn", "noIrregularRules") FROM stdin;
\.


--
-- Data for Name: reversi_matching; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.reversi_matching (id, "parentId", "childId") FROM stdin;
\.


--
-- Data for Name: role; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.role (id, "updatedAt", name, description, "isPublic", "isModerator", "isAdministrator", policies, color, "canEditMembersByModerator", "lastUsedAt", target, "condFormula", "iconUrl", "asBadge", "displayOrder", "isExplorable") FROM stdin;
\.


--
-- Data for Name: role_assignment; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.role_assignment (id, "userId", "roleId", "expiresAt") FROM stdin;
\.


--
-- Data for Name: signin; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.signin (id, "userId", ip, headers, success) FROM stdin;
\.


--
-- Data for Name: sw_subscription; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.sw_subscription (id, "userId", endpoint, auth, publickey, "sendReadMessage") FROM stdin;
\.


--
-- Data for Name: used_username; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.used_username (username, "createdAt") FROM stdin;
missuser	2024-03-23 02:17:05.351+00
\.


--
-- Data for Name: user; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public."user" (id, "updatedAt", "lastFetchedAt", username, "usernameLower", name, "followersCount", "followingCount", "notesCount", "avatarId", "bannerId", tags, "isSuspended", "isLocked", "isBot", "isCat", "isRoot", emojis, host, inbox, "sharedInbox", featured, uri, token, "isExplorable", "followersUri", "lastActiveDate", "hideOnlineStatus", "isDeleted", "avatarUrl", "bannerUrl", "avatarBlurhash", "bannerBlurhash", "movedToUri", "alsoKnownAs", "movedAt", "isHibernated", "avatarDecorations") FROM stdin;
9r70xhde0mav0001	\N	\N	missuser	missuser	missuser dayo	0	0	0	\N	\N	{}	f	f	f	f	t	{}	\N	\N	\N	\N	\N	rnQFrV5unPGMBAvX	t	\N	2024-03-23 02:17:05.665+00	f	f	\N	\N	\N	\N	\N	\N	\N	f	[]
\.


--
-- Data for Name: user_group; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_group (id, "createdAt", name, "userId", "isPrivate") FROM stdin;
\.


--
-- Data for Name: user_group_invitation; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_group_invitation (id, "createdAt", "userId", "userGroupId") FROM stdin;
\.


--
-- Data for Name: user_group_invite; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_group_invite (id, "createdAt", "userId", "userGroupId") FROM stdin;
\.


--
-- Data for Name: user_group_joining; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_group_joining (id, "createdAt", "userId", "userGroupId") FROM stdin;
\.


--
-- Data for Name: user_ip; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_ip (id, "createdAt", "userId", ip) FROM stdin;
\.


--
-- Data for Name: user_keypair; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_keypair ("userId", "publicKey", "privateKey") FROM stdin;
9r70xhde0mav0001	-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA9RbL++NOcIcKPpB+0N9t\nkG7LQxDQzJOzlC9r8aEiWKGdh+DfnmYGuTP1yv8pV9eSYffKQLO73enzzrbGXVT7\ns9efaqmkdF0oaxQKm8wAW8HPqw518R/tuIflNOJ5l59Juju34MyvOqY2QdbOFZLG\n6E4xjAvmTQVn8TOXcehcIxVzc8jh7MmoAUWm2m/LYeBOlyo67bElD1JH4Iw1kuWg\n437CNKfaIDhW+W9H0veQ1l+Y4jnWqpvjlmJ33d6/MR/fPk7VQZlJR2K/8p9iFa21\nkWgjx9Hv481+poksQdUdbT0ZciDtbSx7p+PW2DuAU2Zsv+zrurcfLeDd16ml2vUB\nSQIDAQAB\n-----END PUBLIC KEY-----\n	-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQD1Fsv7405whwo+\nkH7Q322QbstDENDMk7OUL2vxoSJYoZ2H4N+eZga5M/XK/ylX15Jh98pAs7vd6fPO\ntsZdVPuz159qqaR0XShrFAqbzABbwc+rDnXxH+24h+U04nmXn0m6O7fgzK86pjZB\n1s4VksboTjGMC+ZNBWfxM5dx6FwjFXNzyOHsyagBRabab8th4E6XKjrtsSUPUkfg\njDWS5aDjfsI0p9ogOFb5b0fS95DWX5jiOdaqm+OWYnfd3r8xH98+TtVBmUlHYr/y\nn2IVrbWRaCPH0e/jzX6miSxB1R1tPRlyIO1tLHun49bYO4BTZmy/7Ou6tx8t4N3X\nqaXa9QFJAgMBAAECggEADmVf5vRDk72C3wjqwgcVrvGrE83ljdaxKiec7gz6cP1R\nPP169RlmFLPWIK3bNlMtwH5nDWThSJgo34AB59KJlFc6gG/lfoZITP3Y36zcaodY\nVOJdECRBeODWmEQjQ0IyePNwCg7LziScNELTSRYdg+Cv7Vt3lBeYsuTGZQTygIbx\nGsaj/kGUyuka55luI8NM5bwOCO7ZnK4Rg3KjWxNygXKoXjYKpFb15TA6KyzBNykc\nouq1ThGVOd5OQ1K18tkeMzE7IbnmTKqZoEX4URebADX3FI2bAJ/Dvlm3QJ5J2Mge\n6O4TbD1N2mP11Q0p/6Fu2V3DtTVYPcBhWd/coaWA/wKBgQD98HC9OVycslB1+57Z\n5w36JHPX4nn9J6nb/kQ7DL569YeGxs8NXIjfPpcTAKfsbRi2xuPiQ8FG39x6dX/L\nXyS4jr0gK7VZzP0nU8ocZplp2jRHcrfUnreeiTL9S9B/70qAofj3s0lFfUvd23Vf\ne8V25Dzyo0in4YkmZB1MU0do3wKBgQD3E/hcxOC4rhiMkDA5lqgfpKAfvpRSh+H/\n2aA07pMK3sJXiPPXvsHoCsWg5N7LnrCJDvoIsyOtrJzKl2xEnc5CS/yfCvtgMeIJ\nFX/xp4DSMffBsnVpB2MPW1HBE8Znnt5NEFTkUJl84ONwDwduGSNOUcEgKdDb+d81\nOSUr16vS1wKBgQDgiaNWXgs67xzgRh5e9MRSI7te7/4Hz/OM99ajFBC1rrcogFCC\nswi/xZtWDSVuk8TfkRvdbSXQoo9UpOLcFAPnQSeP87YGcpCCHr++vyX2CCBj8NcF\npVYdU5mHiWsSRKdu+Emp1Jj8Xd8gjDXLuSiQiR4vOhw7fdyE2s9hNt4UpwKBgGN2\nKI6/um7dtogvKxqjqT5DuSnOQEsQ5EtoQfPM7mh7z+QI/5Aj/E76tx/TwlRZp1sl\nKkYCRySMzflIB49/rx5FFIa5lwPcUM+zVfPjqBn1f3T78AO9s4TlD/4XhdEExRxk\nrKfUQlVg+m2Lv0P03p+SjZny+17yMYZtYKdLhKG1AoGAL4LePv+bu8O2kdKbTY6e\nmhIbSsDMY/rkfsk+jHQHEvXlDhHupuOE3XmOVzlELaAxVldDn1uWqr2bpc9BTgjy\nfzX34IEmFUaMj25Im2fc9HoZRDtxabJmmkuBvEP9LEgg9KBjPzgP7Ws3Pag21th5\nLAIb7loVV5ZNkZ69NQ1n5JU=\n-----END PRIVATE KEY-----\n
\.


--
-- Data for Name: user_list; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_list (id, "userId", name, "isPublic") FROM stdin;
\.


--
-- Data for Name: user_list_favorite; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_list_favorite (id, "userId", "userListId") FROM stdin;
\.


--
-- Data for Name: user_list_membership; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_list_membership (id, "userId", "userListId", "withReplies", "userListUserId") FROM stdin;
\.


--
-- Data for Name: user_memo; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_memo (id, "userId", "targetUserId", memo) FROM stdin;
\.


--
-- Data for Name: user_note_pining; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_note_pining (id, "userId", "noteId") FROM stdin;
\.


--
-- Data for Name: user_pending; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_pending (id, code, username, email, password) FROM stdin;
\.


--
-- Data for Name: user_profile; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_profile ("userId", location, birthday, description, fields, url, email, "emailVerifyCode", "emailVerified", "twoFactorTempSecret", "twoFactorSecret", "twoFactorEnabled", password, "clientData", "autoAcceptFollowed", "alwaysMarkNsfw", "carefulBot", "userHost", "securityKeysAvailable", "usePasswordLessLogin", "pinnedPageId", room, "injectFeaturedNote", "enableWordMute", "mutedWords", "noCrawle", "receiveAnnouncementEmail", "emailNotificationTypes", lang, "mutedInstances", "publicReactions", "autoSensitive", "moderationNote", achievements, "loggedInDates", "preventAiLearning", "twoFactorBackupSecret", "verifiedLinks", "notificationRecieveConfig", "hardMutedWords", "followingVisibility", "followersVisibility") FROM stdin;
9r70xhde0mav0001	\N	\N	misskey bio	[]	\N	\N	\N	f	\N	\N	f	$2a$08$1pkPUJi46LLAGciFiBdZUe6Wr/M4FaLU6W.f3WgYcbq5aWRRw.CtS	{}	t	f	f	\N	f	f	\N	{}	t	f	[]	f	t	["follow", "receiveFollowRequest"]	\N	[]	t	f		[]	{2024/3/23}	t	\N	{}	{}	[]	public	public
\.


--
-- Data for Name: user_publickey; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_publickey ("userId", "keyId", "keyPem") FROM stdin;
\.


--
-- Data for Name: user_security_key; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.user_security_key (id, "userId", "publicKey", "lastUsed", name, counter, "credentialDeviceType", "credentialBackedUp", transports) FROM stdin;
\.


--
-- Data for Name: webhook; Type: TABLE DATA; Schema: public; Owner: example-misskey-user
--

COPY public.webhook (id, "userId", name, "on", url, secret, active, "latestSentAt", "latestStatus") FROM stdin;
\.


--
-- Name: __chart__active_users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__active_users_id_seq', 1, false);


--
-- Name: __chart__ap_request_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__ap_request_id_seq', 1, false);


--
-- Name: __chart__drive_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__drive_id_seq', 1, false);


--
-- Name: __chart__federation_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__federation_id_seq', 1, false);


--
-- Name: __chart__hashtag_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__hashtag_id_seq', 1, false);


--
-- Name: __chart__instance_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__instance_id_seq', 1, false);


--
-- Name: __chart__network_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__network_id_seq', 1, false);


--
-- Name: __chart__notes_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__notes_id_seq', 1, false);


--
-- Name: __chart__per_user_drive_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__per_user_drive_id_seq', 1, false);


--
-- Name: __chart__per_user_following_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__per_user_following_id_seq', 1, false);


--
-- Name: __chart__per_user_notes_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__per_user_notes_id_seq', 1, false);


--
-- Name: __chart__per_user_pv_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__per_user_pv_id_seq', 1, false);


--
-- Name: __chart__per_user_reaction_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__per_user_reaction_id_seq', 1, false);


--
-- Name: __chart__test_grouped_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__test_grouped_id_seq', 1, false);


--
-- Name: __chart__test_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__test_id_seq', 1, false);


--
-- Name: __chart__test_unique_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__test_unique_id_seq', 1, false);


--
-- Name: __chart__users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart__users_id_seq', 1, false);


--
-- Name: __chart_day__active_users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__active_users_id_seq', 1, false);


--
-- Name: __chart_day__ap_request_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__ap_request_id_seq', 1, false);


--
-- Name: __chart_day__drive_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__drive_id_seq', 1, false);


--
-- Name: __chart_day__federation_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__federation_id_seq', 1, false);


--
-- Name: __chart_day__hashtag_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__hashtag_id_seq', 1, false);


--
-- Name: __chart_day__instance_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__instance_id_seq', 1, false);


--
-- Name: __chart_day__network_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__network_id_seq', 1, false);


--
-- Name: __chart_day__notes_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__notes_id_seq', 1, false);


--
-- Name: __chart_day__per_user_drive_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__per_user_drive_id_seq', 1, false);


--
-- Name: __chart_day__per_user_following_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__per_user_following_id_seq', 1, false);


--
-- Name: __chart_day__per_user_notes_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__per_user_notes_id_seq', 1, false);


--
-- Name: __chart_day__per_user_pv_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__per_user_pv_id_seq', 1, false);


--
-- Name: __chart_day__per_user_reaction_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__per_user_reaction_id_seq', 1, false);


--
-- Name: __chart_day__users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.__chart_day__users_id_seq', 1, false);


--
-- Name: migrations_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.migrations_id_seq', 282, true);


--
-- Name: user_ip_id_seq; Type: SEQUENCE SET; Schema: public; Owner: example-misskey-user
--

SELECT pg_catalog.setval('public.user_ip_id_seq', 1, false);


--
-- Name: __chart_day__per_user_pv PK_0085d7542f6772e99b9dcfb0a9c; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_pv
    ADD CONSTRAINT "PK_0085d7542f6772e99b9dcfb0a9c" PRIMARY KEY (id);


--
-- Name: ad PK_0193d5ef09746e88e9ea92c634d; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.ad
    ADD CONSTRAINT "PK_0193d5ef09746e88e9ea92c634d" PRIMARY KEY (id);


--
-- Name: __chart__notes PK_0aec823fa85c7f901bdb3863b14; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__notes
    ADD CONSTRAINT "PK_0aec823fa85c7f901bdb3863b14" PRIMARY KEY (id);


--
-- Name: flash PK_0c01a2c1c5f2266942dd1b3fdbc; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.flash
    ADD CONSTRAINT "PK_0c01a2c1c5f2266942dd1b3fdbc" PRIMARY KEY (id);


--
-- Name: user_publickey PK_10c146e4b39b443ede016f6736d; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_publickey
    ADD CONSTRAINT "PK_10c146e4b39b443ede016f6736d" PRIMARY KEY ("userId");


--
-- Name: user_list_membership PK_11abb3768da1c5f8de101c9df45; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_list_membership
    ADD CONSTRAINT "PK_11abb3768da1c5f8de101c9df45" PRIMARY KEY (id);


--
-- Name: __chart__instance PK_1267c67c7c2d47b4903975f2c00; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__instance
    ADD CONSTRAINT "PK_1267c67c7c2d47b4903975f2c00" PRIMARY KEY (id);


--
-- Name: __chart_day__hashtag PK_13d5a3b089344e5557f8e0980b4; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__hashtag
    ADD CONSTRAINT "PK_13d5a3b089344e5557f8e0980b4" PRIMARY KEY (id);


--
-- Name: user_group_joining PK_15f2425885253c5507e1599cfe7; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_joining
    ADD CONSTRAINT "PK_15f2425885253c5507e1599cfe7" PRIMARY KEY (id);


--
-- Name: user_group_invitation PK_160c63ec02bf23f6a5c5e8140d6; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_invitation
    ADD CONSTRAINT "PK_160c63ec02bf23f6a5c5e8140d6" PRIMARY KEY (id);


--
-- Name: note_unread PK_1904eda61a784f57e6e51fa9c1f; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_unread
    ADD CONSTRAINT "PK_1904eda61a784f57e6e51fa9c1f" PRIMARY KEY (id);


--
-- Name: auth_session PK_19354ed146424a728c1112a8cbf; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.auth_session
    ADD CONSTRAINT "PK_19354ed146424a728c1112a8cbf" PRIMARY KEY (id);


--
-- Name: __chart_day__per_user_drive PK_1ae135254c137011645da7f4045; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_drive
    ADD CONSTRAINT "PK_1ae135254c137011645da7f4045" PRIMARY KEY (id);


--
-- Name: clip_favorite PK_1b539f43906f05ebcabe752a977; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.clip_favorite
    ADD CONSTRAINT "PK_1b539f43906f05ebcabe752a977" PRIMARY KEY (id);


--
-- Name: __chart_day__notes PK_1fa4139e1f338272b758d05e090; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__notes
    ADD CONSTRAINT "PK_1fa4139e1f338272b758d05e090" PRIMARY KEY (id);


--
-- Name: retention_aggregation PK_22aad3e8640b15fb3b90ee02d18; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.retention_aggregation
    ADD CONSTRAINT "PK_22aad3e8640b15fb3b90ee02d18" PRIMARY KEY (id);


--
-- Name: user_ip PK_2c44ddfbf7c0464d028dcef325e; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_ip
    ADD CONSTRAINT "PK_2c44ddfbf7c0464d028dcef325e" PRIMARY KEY (id);


--
-- Name: muting PK_2e92d06c8b5c602eeb27ca9ba48; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.muting
    ADD CONSTRAINT "PK_2e92d06c8b5c602eeb27ca9ba48" PRIMARY KEY (id);


--
-- Name: __chart__active_users PK_317237a9f733b970604a11e314f; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__active_users
    ADD CONSTRAINT "PK_317237a9f733b970604a11e314f" PRIMARY KEY (id);


--
-- Name: __chart__per_user_notes PK_334acf6e915af2f29edc11b8e50; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_notes
    ADD CONSTRAINT "PK_334acf6e915af2f29edc11b8e50" PRIMARY KEY (id);


--
-- Name: user_group_invite PK_3893884af0d3a5f4d01e7921a97; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_invite
    ADD CONSTRAINT "PK_3893884af0d3a5f4d01e7921a97" PRIMARY KEY (id);


--
-- Name: user_group PK_3c29fba6fe013ec8724378ce7c9; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group
    ADD CONSTRAINT "PK_3c29fba6fe013ec8724378ce7c9" PRIMARY KEY (id);


--
-- Name: __chart__per_user_pv PK_3c938a24f0203b5bd13fab51059; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_pv
    ADD CONSTRAINT "PK_3c938a24f0203b5bd13fab51059" PRIMARY KEY (id);


--
-- Name: user_security_key PK_3e508571121ab39c5f85d10c166; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_security_key
    ADD CONSTRAINT "PK_3e508571121ab39c5f85d10c166" PRIMARY KEY (id);


--
-- Name: __chart__test_unique PK_409bac9c97cc612d8500012319d; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__test_unique
    ADD CONSTRAINT "PK_409bac9c97cc612d8500012319d" PRIMARY KEY (id);


--
-- Name: drive_file PK_43ddaaaf18c9e68029b7cbb032e; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.drive_file
    ADD CONSTRAINT "PK_43ddaaaf18c9e68029b7cbb032e" PRIMARY KEY (id);


--
-- Name: channel_note_pining PK_44f7474496bcf2e4b741681146d; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_note_pining
    ADD CONSTRAINT "PK_44f7474496bcf2e4b741681146d" PRIMARY KEY (id);


--
-- Name: __chart_day__instance PK_479a8ff9d959274981087043023; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__instance
    ADD CONSTRAINT "PK_479a8ff9d959274981087043023" PRIMARY KEY (id);


--
-- Name: note_watching PK_49286fdb23725945a74aa27d757; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_watching
    ADD CONSTRAINT "PK_49286fdb23725945a74aa27d757" PRIMARY KEY (id);


--
-- Name: announcement_read PK_4b90ad1f42681d97b2683890c5e; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.announcement_read
    ADD CONSTRAINT "PK_4b90ad1f42681d97b2683890c5e" PRIMARY KEY (id);


--
-- Name: __chart__users PK_4dfcf2c78d03524b9eb2c99d328; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__users
    ADD CONSTRAINT "PK_4dfcf2c78d03524b9eb2c99d328" PRIMARY KEY (id);


--
-- Name: user_profile PK_51cb79b5555effaf7d69ba1cff9; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_profile
    ADD CONSTRAINT "PK_51cb79b5555effaf7d69ba1cff9" PRIMARY KEY ("userId");


--
-- Name: follow_request PK_53a9aa3725f7a3deb150b39dbfc; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.follow_request
    ADD CONSTRAINT "PK_53a9aa3725f7a3deb150b39dbfc" PRIMARY KEY (id);


--
-- Name: __chart__ap_request PK_56a25cd447c7ee08876b3baf8d8; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__ap_request
    ADD CONSTRAINT "PK_56a25cd447c7ee08876b3baf8d8" PRIMARY KEY (id);


--
-- Name: __chart_day__per_user_notes PK_58bab6b6d3ad9310cbc7460fd28; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_notes
    ADD CONSTRAINT "PK_58bab6b6d3ad9310cbc7460fd28" PRIMARY KEY (id);


--
-- Name: channel PK_590f33ee6ee7d76437acf362e39; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel
    ADD CONSTRAINT "PK_590f33ee6ee7d76437acf362e39" PRIMARY KEY (id);


--
-- Name: channel_favorite PK_59bddfd54d48689a298d41af00c; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_favorite
    ADD CONSTRAINT "PK_59bddfd54d48689a298d41af00c" PRIMARY KEY (id);


--
-- Name: promo_read PK_61917c1541002422b703318b7c9; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.promo_read
    ADD CONSTRAINT "PK_61917c1541002422b703318b7c9" PRIMARY KEY (id);


--
-- Name: registry_item PK_64b3f7e6008b4d89b826cd3af95; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.registry_item
    ADD CONSTRAINT "PK_64b3f7e6008b4d89b826cd3af95" PRIMARY KEY (id);


--
-- Name: __chart_day__per_user_following PK_68ce6b67da57166da66fc8fb27e; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_following
    ADD CONSTRAINT "PK_68ce6b67da57166da66fc8fb27e" PRIMARY KEY (id);


--
-- Name: page PK_742f4117e065c5b6ad21b37ba1f; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.page
    ADD CONSTRAINT "PK_742f4117e065c5b6ad21b37ba1f" PRIMARY KEY (id);


--
-- Name: note_reaction PK_767ec729b108799b587a3fcc9cf; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_reaction
    ADD CONSTRAINT "PK_767ec729b108799b587a3fcc9cf" PRIMARY KEY (id);


--
-- Name: reversi_game PK_76b30eeba71b1193ad7c5311c3f; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.reversi_game
    ADD CONSTRAINT "PK_76b30eeba71b1193ad7c5311c3f" PRIMARY KEY (id);


--
-- Name: relay PK_78ebc9cfddf4292633b7ba57aee; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.relay
    ADD CONSTRAINT "PK_78ebc9cfddf4292633b7ba57aee" PRIMARY KEY (id);


--
-- Name: used_username PK_78fd79d2d24c6ac2f4cc9a31a5d; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.used_username
    ADD CONSTRAINT "PK_78fd79d2d24c6ac2f4cc9a31a5d" PRIMARY KEY (username);


--
-- Name: drive_folder PK_7a0c089191f5ebdc214e0af808a; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.drive_folder
    ADD CONSTRAINT "PK_7a0c089191f5ebdc214e0af808a" PRIMARY KEY (id);


--
-- Name: __chart_day__federation PK_7ca721c769f31698e0e1331e8e6; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__federation
    ADD CONSTRAINT "PK_7ca721c769f31698e0e1331e8e6" PRIMARY KEY (id);


--
-- Name: role_assignment PK_7e79671a8a5db18936173148cb4; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.role_assignment
    ADD CONSTRAINT "PK_7e79671a8a5db18936173148cb4" PRIMARY KEY (id);


--
-- Name: page_like PK_813f034843af992d3ae0f43c64c; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.page_like
    ADD CONSTRAINT "PK_813f034843af992d3ae0f43c64c" PRIMARY KEY (id);


--
-- Name: gallery_like PK_853ab02be39b8de45cd720cc15f; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.gallery_like
    ADD CONSTRAINT "PK_853ab02be39b8de45cd720cc15f" PRIMARY KEY (id);


--
-- Name: __chart__per_user_following PK_85bb1b540363a29c2fec83bd907; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_following
    ADD CONSTRAINT "PK_85bb1b540363a29c2fec83bd907" PRIMARY KEY (id);


--
-- Name: abuse_user_report PK_87873f5f5cc5c321a1306b2d18c; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.abuse_user_report
    ADD CONSTRAINT "PK_87873f5f5cc5c321a1306b2d18c" PRIMARY KEY (id);


--
-- Name: user_list PK_87bab75775fd9b1ff822b656402; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_list
    ADD CONSTRAINT "PK_87bab75775fd9b1ff822b656402" PRIMARY KEY (id);


--
-- Name: reversi_matching PK_880bd0afbab232f21c8b9d146cf; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.reversi_matching
    ADD CONSTRAINT "PK_880bd0afbab232f21c8b9d146cf" PRIMARY KEY (id);


--
-- Name: __chart_day__per_user_reaction PK_8af24e2d51ff781a354fe595eda; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_reaction
    ADD CONSTRAINT "PK_8af24e2d51ff781a354fe595eda" PRIMARY KEY (id);


--
-- Name: channel_following PK_8b104be7f7415113f2a02cd5bdd; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_following
    ADD CONSTRAINT "PK_8b104be7f7415113f2a02cd5bdd" PRIMARY KEY (id);


--
-- Name: migrations PK_8c82d7f526340ab734260ea46be; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.migrations
    ADD CONSTRAINT "PK_8c82d7f526340ab734260ea46be" PRIMARY KEY (id);


--
-- Name: gallery_post PK_8e90d7b6015f2c4518881b14753; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.gallery_post
    ADD CONSTRAINT "PK_8e90d7b6015f2c4518881b14753" PRIMARY KEY (id);


--
-- Name: __chart_day__ap_request PK_9318b49daee320194e23f712e69; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__ap_request
    ADD CONSTRAINT "PK_9318b49daee320194e23f712e69" PRIMARY KEY (id);


--
-- Name: app PK_9478629fc093d229df09e560aea; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.app
    ADD CONSTRAINT "PK_9478629fc093d229df09e560aea" PRIMARY KEY (id);


--
-- Name: note PK_96d0c172a4fba276b1bbed43058; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT "PK_96d0c172a4fba276b1bbed43058" PRIMARY KEY (id);


--
-- Name: __chart__per_user_reaction PK_984f54dae441e65b633e8d27a7f; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_reaction
    ADD CONSTRAINT "PK_984f54dae441e65b633e8d27a7f" PRIMARY KEY (id);


--
-- Name: signin PK_9e96ddc025712616fc492b3b588; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.signin
    ADD CONSTRAINT "PK_9e96ddc025712616fc492b3b588" PRIMARY KEY (id);


--
-- Name: user_note_pining PK_a6a2dad4ae000abce2ea9d9b103; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_note_pining
    ADD CONSTRAINT "PK_a6a2dad4ae000abce2ea9d9b103" PRIMARY KEY (id);


--
-- Name: bubble_game_record PK_a75395fe404b392e2893b50d7ea; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.bubble_game_record
    ADD CONSTRAINT "PK_a75395fe404b392e2893b50d7ea" PRIMARY KEY (id);


--
-- Name: note_favorite PK_af0da35a60b9fa4463a62082b36; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_favorite
    ADD CONSTRAINT "PK_af0da35a60b9fa4463a62082b36" PRIMARY KEY (id);


--
-- Name: __chart_day__active_users PK_b1790489b14f005ae8f404f5795; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__active_users
    ADD CONSTRAINT "PK_b1790489b14f005ae8f404f5795" PRIMARY KEY (id);


--
-- Name: role PK_b36bcfe02fc8de3c57a8b2391c2; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.role
    ADD CONSTRAINT "PK_b36bcfe02fc8de3c57a8b2391c2" PRIMARY KEY (id);


--
-- Name: __chart__federation PK_b39dcd31a0fe1a7757e348e85fd; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__federation
    ADD CONSTRAINT "PK_b39dcd31a0fe1a7757e348e85fd" PRIMARY KEY (id);


--
-- Name: __chart__test PK_b4bc31dffbd1b785276a3ecfc1e; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__test
    ADD CONSTRAINT "PK_b4bc31dffbd1b785276a3ecfc1e" PRIMARY KEY (id);


--
-- Name: avatar_decoration PK_b6de9296f6097078e1dc53f7603; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.avatar_decoration
    ADD CONSTRAINT "PK_b6de9296f6097078e1dc53f7603" PRIMARY KEY (id);


--
-- Name: __chart__network PK_bc4290c2e27fad14ef0c1ca93f3; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__network
    ADD CONSTRAINT "PK_bc4290c2e27fad14ef0c1ca93f3" PRIMARY KEY (id);


--
-- Name: user_list_favorite PK_c0974b21e18502a4c8178e09fe6; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_list_favorite
    ADD CONSTRAINT "PK_c0974b21e18502a4c8178e09fe6" PRIMARY KEY (id);


--
-- Name: antenna PK_c170b99775e1dccca947c9f2d5f; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.antenna
    ADD CONSTRAINT "PK_c170b99775e1dccca947c9f2d5f" PRIMARY KEY (id);


--
-- Name: __chart__hashtag PK_c32f1ea2b44a5d2f7881e37f8f9; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__hashtag
    ADD CONSTRAINT "PK_c32f1ea2b44a5d2f7881e37f8f9" PRIMARY KEY (id);


--
-- Name: meta PK_c4c17a6c2bd7651338b60fc590b; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.meta
    ADD CONSTRAINT "PK_c4c17a6c2bd7651338b60fc590b" PRIMARY KEY (id);


--
-- Name: following PK_c76c6e044bdf76ecf8bfb82a645; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.following
    ADD CONSTRAINT "PK_c76c6e044bdf76ecf8bfb82a645" PRIMARY KEY (id);


--
-- Name: __chart_day__network PK_cac499d6f471042dfed1e7e0132; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__network
    ADD CONSTRAINT "PK_cac499d6f471042dfed1e7e0132" PRIMARY KEY (id);


--
-- Name: user PK_cace4a159ff9f2512dd42373760; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT "PK_cace4a159ff9f2512dd42373760" PRIMARY KEY (id);


--
-- Name: hashtag PK_cb36eb8af8412bfa978f1165d78; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.hashtag
    ADD CONSTRAINT "PK_cb36eb8af8412bfa978f1165d78" PRIMARY KEY (id);


--
-- Name: moderation_log PK_d0adca6ecfd068db83e4526cc26; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.moderation_log
    ADD CONSTRAINT "PK_d0adca6ecfd068db83e4526cc26" PRIMARY KEY (id);


--
-- Name: __chart__per_user_drive PK_d0ef23d24d666e1a44a0cd3d208; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_drive
    ADD CONSTRAINT "PK_d0ef23d24d666e1a44a0cd3d208" PRIMARY KEY (id);


--
-- Name: flash_like PK_d110109ee310588d63d6183b233; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.flash_like
    ADD CONSTRAINT "PK_d110109ee310588d63d6183b233" PRIMARY KEY (id);


--
-- Name: user_pending PK_d4c84e013c98ec02d19b8fbbafa; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_pending
    ADD CONSTRAINT "PK_d4c84e013c98ec02d19b8fbbafa" PRIMARY KEY (id);


--
-- Name: __chart_day__users PK_d7f7185abb9851f70c4726c54bd; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__users
    ADD CONSTRAINT "PK_d7f7185abb9851f70c4726c54bd" PRIMARY KEY (id);


--
-- Name: poll PK_da851e06d0dfe2ef397d8b1bf1b; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.poll
    ADD CONSTRAINT "PK_da851e06d0dfe2ef397d8b1bf1b" PRIMARY KEY ("noteId");


--
-- Name: messaging_message PK_db398fd79dc95d0eb8c30456eaa; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.messaging_message
    ADD CONSTRAINT "PK_db398fd79dc95d0eb8c30456eaa" PRIMARY KEY (id);


--
-- Name: emoji PK_df74ce05e24999ee01ea0bc50a3; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.emoji
    ADD CONSTRAINT "PK_df74ce05e24999ee01ea0bc50a3" PRIMARY KEY (id);


--
-- Name: announcement PK_e0ef0550174fd1099a308fd18a0; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.announcement
    ADD CONSTRAINT "PK_e0ef0550174fd1099a308fd18a0" PRIMARY KEY (id);


--
-- Name: promo_note PK_e263909ca4fe5d57f8d4230dd5c; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.promo_note
    ADD CONSTRAINT "PK_e263909ca4fe5d57f8d4230dd5c" PRIMARY KEY ("noteId");


--
-- Name: blocking PK_e5d9a541cc1965ee7e048ea09dd; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.blocking
    ADD CONSTRAINT "PK_e5d9a541cc1965ee7e048ea09dd" PRIMARY KEY (id);


--
-- Name: webhook PK_e6765510c2d078db49632b59020; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.webhook
    ADD CONSTRAINT "PK_e6765510c2d078db49632b59020" PRIMARY KEY (id);


--
-- Name: __chart_day__drive PK_e7ec0de057c77c40fc8d8b62151; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__drive
    ADD CONSTRAINT "PK_e7ec0de057c77c40fc8d8b62151" PRIMARY KEY (id);


--
-- Name: sw_subscription PK_e8f763631530051b95eb6279b91; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.sw_subscription
    ADD CONSTRAINT "PK_e8f763631530051b95eb6279b91" PRIMARY KEY (id);


--
-- Name: clip_note PK_e94cda2f40a99b57e032a1a738b; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.clip_note
    ADD CONSTRAINT "PK_e94cda2f40a99b57e032a1a738b" PRIMARY KEY (id);


--
-- Name: user_memo PK_e9aaa58f7d3699a84d79078f4d9; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_memo
    ADD CONSTRAINT "PK_e9aaa58f7d3699a84d79078f4d9" PRIMARY KEY (id);


--
-- Name: instance PK_eaf60e4a0c399c9935413e06474; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.instance
    ADD CONSTRAINT "PK_eaf60e4a0c399c9935413e06474" PRIMARY KEY (id);


--
-- Name: note_thread_muting PK_ec5936d94d1a0369646d12a3a47; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_thread_muting
    ADD CONSTRAINT "PK_ec5936d94d1a0369646d12a3a47" PRIMARY KEY (id);


--
-- Name: clip PK_f0685dac8d4dd056d7255670b75; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.clip
    ADD CONSTRAINT "PK_f0685dac8d4dd056d7255670b75" PRIMARY KEY (id);


--
-- Name: registration_ticket PK_f11696b6fafcf3662d4292734f8; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.registration_ticket
    ADD CONSTRAINT "PK_f11696b6fafcf3662d4292734f8" PRIMARY KEY (id);


--
-- Name: access_token PK_f20f028607b2603deabd8182d12; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.access_token
    ADD CONSTRAINT "PK_f20f028607b2603deabd8182d12" PRIMARY KEY (id);


--
-- Name: user_keypair PK_f4853eb41ab722fe05f81cedeb6; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_keypair
    ADD CONSTRAINT "PK_f4853eb41ab722fe05f81cedeb6" PRIMARY KEY ("userId");


--
-- Name: __chart__test_grouped PK_f4a2b175d308695af30d4293272; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__test_grouped
    ADD CONSTRAINT "PK_f4a2b175d308695af30d4293272" PRIMARY KEY (id);


--
-- Name: __chart__drive PK_f96bc548a765cd4b3b354221ce7; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__drive
    ADD CONSTRAINT "PK_f96bc548a765cd4b3b354221ce7" PRIMARY KEY (id);


--
-- Name: password_reset_request PK_fcf4b02eae1403a2edaf87fd074; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.password_reset_request
    ADD CONSTRAINT "PK_fcf4b02eae1403a2edaf87fd074" PRIMARY KEY (id);


--
-- Name: poll_vote PK_fd002d371201c472490ba89c6a0; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.poll_vote
    ADD CONSTRAINT "PK_fd002d371201c472490ba89c6a0" PRIMARY KEY (id);


--
-- Name: renote_muting PK_renoteMuting_id; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.renote_muting
    ADD CONSTRAINT "PK_renoteMuting_id" PRIMARY KEY (id);


--
-- Name: user REL_58f5c71eaab331645112cf8cfa; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT "REL_58f5c71eaab331645112cf8cfa" UNIQUE ("avatarId");


--
-- Name: user REL_afc64b53f8db3707ceb34eb28e; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT "REL_afc64b53f8db3707ceb34eb28e" UNIQUE ("bannerId");


--
-- Name: __chart__active_users UQ_0ad37b7ef50f4ddc84363d7ccca; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__active_users
    ADD CONSTRAINT "UQ_0ad37b7ef50f4ddc84363d7ccca" UNIQUE (date);


--
-- Name: __chart_day__drive UQ_0b60ebb3aa0065f10b0616c1171; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__drive
    ADD CONSTRAINT "UQ_0b60ebb3aa0065f10b0616c1171" UNIQUE (date);


--
-- Name: __chart__drive UQ_13565815f618a1ff53886c5b28a; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__drive
    ADD CONSTRAINT "UQ_13565815f618a1ff53886c5b28a" UNIQUE (date);


--
-- Name: __chart_day__notes UQ_1a527b423ad0858a1af5a056d43; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__notes
    ADD CONSTRAINT "UQ_1a527b423ad0858a1af5a056d43" UNIQUE (date);


--
-- Name: __chart__per_user_reaction UQ_229a41ad465f9205f1f57032910; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_reaction
    ADD CONSTRAINT "UQ_229a41ad465f9205f1f57032910" UNIQUE (date, "group");


--
-- Name: __chart__hashtag UQ_25a97c02003338124b2b75fdbc8; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__hashtag
    ADD CONSTRAINT "UQ_25a97c02003338124b2b75fdbc8" UNIQUE (date, "group");


--
-- Name: __chart__per_user_drive UQ_30bf67687f483ace115c5ca6429; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_drive
    ADD CONSTRAINT "UQ_30bf67687f483ace115c5ca6429" UNIQUE (date, "group");


--
-- Name: __chart__federation UQ_36cb699c49580d4e6c2e6159f97; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__federation
    ADD CONSTRAINT "UQ_36cb699c49580d4e6c2e6159f97" UNIQUE (date);


--
-- Name: __chart__instance UQ_39ee857ab2f23493037c6b66311; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__instance
    ADD CONSTRAINT "UQ_39ee857ab2f23493037c6b66311" UNIQUE (date, "group");


--
-- Name: __chart__notes UQ_42eb716a37d381cdf566192b2be; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__notes
    ADD CONSTRAINT "UQ_42eb716a37d381cdf566192b2be" UNIQUE (date);


--
-- Name: __chart__per_user_notes UQ_5048e9daccbbbc6d567bb142d34; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_notes
    ADD CONSTRAINT "UQ_5048e9daccbbbc6d567bb142d34" UNIQUE (date, "group");


--
-- Name: __chart_day__federation UQ_617a8fe225a6e701d89e02d2c74; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__federation
    ADD CONSTRAINT "UQ_617a8fe225a6e701d89e02d2c74" UNIQUE (date);


--
-- Name: __chart_day__per_user_drive UQ_62aa5047b5aec92524f24c701d7; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_drive
    ADD CONSTRAINT "UQ_62aa5047b5aec92524f24c701d7" UNIQUE (date, "group");


--
-- Name: user_profile UQ_6dc44f1ceb65b1e72bacef2ca27; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_profile
    ADD CONSTRAINT "UQ_6dc44f1ceb65b1e72bacef2ca27" UNIQUE ("pinnedPageId");


--
-- Name: __chart__users UQ_845254b3eaf708ae8a6cac30265; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__users
    ADD CONSTRAINT "UQ_845254b3eaf708ae8a6cac30265" UNIQUE (date);


--
-- Name: __chart_day__network UQ_8bfa548c2b31f9e07db113773ee; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__network
    ADD CONSTRAINT "UQ_8bfa548c2b31f9e07db113773ee" UNIQUE (date);


--
-- Name: __chart_day__hashtag UQ_8f589cf056ff51f09d6096f6450; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__hashtag
    ADD CONSTRAINT "UQ_8f589cf056ff51f09d6096f6450" UNIQUE (date, "group");


--
-- Name: __chart__network UQ_a1efd3e0048a5f2793a47360dc6; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__network
    ADD CONSTRAINT "UQ_a1efd3e0048a5f2793a47360dc6" UNIQUE (date);


--
-- Name: __chart_day__ap_request UQ_a848f66d6cec11980a5dd595822; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__ap_request
    ADD CONSTRAINT "UQ_a848f66d6cec11980a5dd595822" UNIQUE (date);


--
-- Name: user UQ_a854e557b1b14814750c7c7b0c9; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT "UQ_a854e557b1b14814750c7c7b0c9" UNIQUE (token);


--
-- Name: registration_ticket UQ_b6f93f2f30bdbb9a5ebdc7c7189; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.registration_ticket
    ADD CONSTRAINT "UQ_b6f93f2f30bdbb9a5ebdc7c7189" UNIQUE ("usedById");


--
-- Name: __chart__per_user_following UQ_b77d4dd9562c3a899d9a286fcd7; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_following
    ADD CONSTRAINT "UQ_b77d4dd9562c3a899d9a286fcd7" UNIQUE (date, "group");


--
-- Name: __chart_day__per_user_notes UQ_c5545d4b31cdc684034e33b81c3; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_notes
    ADD CONSTRAINT "UQ_c5545d4b31cdc684034e33b81c3" UNIQUE (date, "group");


--
-- Name: __chart_day__users UQ_cad6e07c20037f31cdba8a350c3; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__users
    ADD CONSTRAINT "UQ_cad6e07c20037f31cdba8a350c3" UNIQUE (date);


--
-- Name: __chart_day__per_user_reaction UQ_d54b653660d808b118e36c184c0; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_reaction
    ADD CONSTRAINT "UQ_d54b653660d808b118e36c184c0" UNIQUE (date, "group");


--
-- Name: __chart_day__active_users UQ_d5954f3df5e5e3bdfc3c03f3906; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__active_users
    ADD CONSTRAINT "UQ_d5954f3df5e5e3bdfc3c03f3906" UNIQUE (date);


--
-- Name: __chart_day__per_user_following UQ_e4849a3231f38281280ea4c0eee; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_following
    ADD CONSTRAINT "UQ_e4849a3231f38281280ea4c0eee" UNIQUE (date, "group");


--
-- Name: __chart__ap_request UQ_e56f4beac5746d44bc3e19c80d0; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__ap_request
    ADD CONSTRAINT "UQ_e56f4beac5746d44bc3e19c80d0" UNIQUE (date);


--
-- Name: __chart_day__per_user_pv UQ_f221e45cfac5bea0ce0f3149fbb; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__per_user_pv
    ADD CONSTRAINT "UQ_f221e45cfac5bea0ce0f3149fbb" UNIQUE (date, "group");


--
-- Name: __chart__per_user_pv UQ_f2a56da57921ca8439f45c1d95f; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart__per_user_pv
    ADD CONSTRAINT "UQ_f2a56da57921ca8439f45c1d95f" UNIQUE (date, "group");


--
-- Name: __chart_day__instance UQ_fea7c0278325a1a2492f2d6acbf; Type: CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.__chart_day__instance
    ADD CONSTRAINT "UQ_fea7c0278325a1a2492f2d6acbf" UNIQUE (date, "group");


--
-- Name: IDX_00ceffb0cdc238b3233294f08f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_00ceffb0cdc238b3233294f08f" ON public.drive_folder USING btree ("parentId");


--
-- Name: IDX_016f613dc4feb807e03e3e7da9; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_016f613dc4feb807e03e3e7da9" ON public.user_list_favorite USING btree ("userId");


--
-- Name: IDX_021015e6683570ae9f6b0c62be; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_021015e6683570ae9f6b0c62be" ON public.user_list_membership USING btree ("userId");


--
-- Name: IDX_03e7028ab8388a3f5e3ce2a861; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_03e7028ab8388a3f5e3ce2a861" ON public.note_watching USING btree ("noteId");


--
-- Name: IDX_04cc96756f89d0b7f9473e8cdf; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_04cc96756f89d0b7f9473e8cdf" ON public.abuse_user_report USING btree ("reporterId");


--
-- Name: IDX_05cca34b985d1b8edc1d1e28df; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_05cca34b985d1b8edc1d1e28df" ON public.gallery_post USING btree (tags);


--
-- Name: IDX_0610ebcfcfb4a18441a9bcdab2; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0610ebcfcfb4a18441a9bcdab2" ON public.poll USING btree ("userId");


--
-- Name: IDX_0627125f1a8a42c9a1929edb55; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0627125f1a8a42c9a1929edb55" ON public.blocking USING btree ("blockerId");


--
-- Name: IDX_084c2abb8948ef59a37dce6ac1; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_084c2abb8948ef59a37dce6ac1" ON public.antenna USING btree ("lastUsedAt");


--
-- Name: IDX_094b86cd36bb805d1aa1e8cc9a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_094b86cd36bb805d1aa1e8cc9a" ON public.channel USING btree ("usersCount");


--
-- Name: IDX_0953deda7ce6e1448e935859e5; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_0953deda7ce6e1448e935859e5" ON public.role_assignment USING btree ("userId", "roleId");


--
-- Name: IDX_09f4e5b9e4a2f268d3e284e4b3; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_09f4e5b9e4a2f268d3e284e4b3" ON public.retention_aggregation USING btree ("createdAt");


--
-- Name: IDX_0a72bdfcdb97c0eca11fe7ecad; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0a72bdfcdb97c0eca11fe7ecad" ON public.registry_item USING btree (domain);


--
-- Name: IDX_0ad37b7ef50f4ddc84363d7ccc; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_0ad37b7ef50f4ddc84363d7ccc" ON public.__chart__active_users USING btree (date);


--
-- Name: IDX_0b03cbcd7e6a7ce068efa8ecc2; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0b03cbcd7e6a7ce068efa8ecc2" ON public.hashtag USING btree ("attachedRemoteUsersCount");


--
-- Name: IDX_0b575fa9a4cfe638a925949285; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_0b575fa9a4cfe638a925949285" ON public.password_reset_request USING btree (token);


--
-- Name: IDX_0b60ebb3aa0065f10b0616c117; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_0b60ebb3aa0065f10b0616c117" ON public.__chart_day__drive USING btree (date);


--
-- Name: IDX_0c44bf4f680964145f2a68a341; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0c44bf4f680964145f2a68a341" ON public.hashtag USING btree ("attachedLocalUsersCount");


--
-- Name: IDX_0d7718e562dcedd0aa5cf2c9f7; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0d7718e562dcedd0aa5cf2c9f7" ON public.user_security_key USING btree ("publicKey");


--
-- Name: IDX_0d801c609cec4e9eb4b6b4490c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_0d801c609cec4e9eb4b6b4490c" ON public.renote_muting USING btree ("muterId", "muteeId");


--
-- Name: IDX_0d9a1738f2cf7f3b1c3334dfab; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_0d9a1738f2cf7f3b1c3334dfab" ON public.relay USING btree (inbox);


--
-- Name: IDX_0e206cec573f1edff4a3062923; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0e206cec573f1edff4a3062923" ON public.hashtag USING btree ("mentionedLocalUsersCount");


--
-- Name: IDX_0e43068c3f92cab197c3d3cd86; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0e43068c3f92cab197c3d3cd86" ON public.channel_following USING btree ("followeeId");


--
-- Name: IDX_0e61efab7f88dbb79c9166dbb4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0e61efab7f88dbb79c9166dbb4" ON public.page_like USING btree ("userId");


--
-- Name: IDX_0f4fb9ad355f3effff221ef245; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_0f4fb9ad355f3effff221ef245" ON public.note_favorite USING btree ("userId", "noteId");


--
-- Name: IDX_0f58c11241e649d2a638a8de94; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_0f58c11241e649d2a638a8de94" ON public.channel USING btree ("notesCount");


--
-- Name: IDX_0ff69e8dfa9fe31bb4a4660f59; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_0ff69e8dfa9fe31bb4a4660f59" ON public.registration_ticket USING btree (code);


--
-- Name: IDX_1039988afa3bf991185b277fe0; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_1039988afa3bf991185b277fe0" ON public.user_group_invite USING btree ("userId");


--
-- Name: IDX_12c01c0d1a79f77d9f6c15fadd; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_12c01c0d1a79f77d9f6c15fadd" ON public.follow_request USING btree ("followeeId");


--
-- Name: IDX_13565815f618a1ff53886c5b28; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_13565815f618a1ff53886c5b28" ON public.__chart__drive USING btree (date);


--
-- Name: IDX_13761f64257f40c5636d0ff95e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_13761f64257f40c5636d0ff95e" ON public.note_reaction USING btree ("userId");


--
-- Name: IDX_153536c67d05e9adb24e99fc2b; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_153536c67d05e9adb24e99fc2b" ON public.note USING btree (uri);


--
-- Name: IDX_16effb2e888f6763673b579f80; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_16effb2e888f6763673b579f80" ON public.__chart__test_unique USING btree (date) WHERE ("group" IS NULL);


--
-- Name: IDX_171e64971c780ebd23fae140bb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_171e64971c780ebd23fae140bb" ON public.user_publickey USING btree ("keyId");


--
-- Name: IDX_17cb3553c700a4985dff5a30ff; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_17cb3553c700a4985dff5a30ff" ON public.note USING btree ("replyId");


--
-- Name: IDX_1a165c68a49d08f11caffbd206; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_1a165c68a49d08f11caffbd206" ON public.gallery_post USING btree ("likedCount");


--
-- Name: IDX_1a527b423ad0858a1af5a056d4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_1a527b423ad0858a1af5a056d4" ON public.__chart_day__notes USING btree (date);


--
-- Name: IDX_1eb9d9824a630321a29fd3b290; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_1eb9d9824a630321a29fd3b290" ON public.muting USING btree ("muterId", "muteeId");


--
-- Name: IDX_20e30aa35180e317e133d75316; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_20e30aa35180e317e133d75316" ON public.user_group USING btree ("createdAt");


--
-- Name: IDX_2133ef8317e4bdb839c0dcbf13; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_2133ef8317e4bdb839c0dcbf13" ON public.page USING btree ("userId", name);


--
-- Name: IDX_229a41ad465f9205f1f5703291; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_229a41ad465f9205f1f5703291" ON public.__chart__per_user_reaction USING btree (date, "group");


--
-- Name: IDX_22baca135bb8a3ea1a83d13df3; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_22baca135bb8a3ea1a83d13df3" ON public.registry_item USING btree (scope);


--
-- Name: IDX_24e0042143a18157b234df186c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_24e0042143a18157b234df186c" ON public.following USING btree ("followeeId");


--
-- Name: IDX_25a31662b0b0cc9af6549a9d71; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_25a31662b0b0cc9af6549a9d71" ON public.clip_favorite USING btree ("userId");


--
-- Name: IDX_25a97c02003338124b2b75fdbc; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_25a97c02003338124b2b75fdbc" ON public.__chart__hashtag USING btree (date, "group");


--
-- Name: IDX_25b1dd384bec391b07b74b861c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_25b1dd384bec391b07b74b861c" ON public.note_unread USING btree ("isMentioned");


--
-- Name: IDX_26d4ee490b5a487142d35466ee; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_26d4ee490b5a487142d35466ee" ON public.bubble_game_record USING btree (score);


--
-- Name: IDX_2710a55f826ee236ea1a62698f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_2710a55f826ee236ea1a62698f" ON public.hashtag USING btree ("mentionedUsersCount");


--
-- Name: IDX_2882b8a1a07c7d281a98b6db16; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_2882b8a1a07c7d281a98b6db16" ON public.promo_read USING btree ("userId", "noteId");


--
-- Name: IDX_29c11c7deb06615076f8c95b80; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_29c11c7deb06615076f8c95b80" ON public.note_thread_muting USING btree ("userId");


--
-- Name: IDX_29e8c1d579af54d4232939f994; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_29e8c1d579af54d4232939f994" ON public.note_unread USING btree ("noteUserId");


--
-- Name: IDX_29ef80c6f13bcea998447fce43; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_29ef80c6f13bcea998447fce43" ON public.channel USING btree ("lastNotedAt");


--
-- Name: IDX_2b15aaf4a0dc5be3499af7ab6a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_2b15aaf4a0dc5be3499af7ab6a" ON public.abuse_user_report USING btree (resolved);


--
-- Name: IDX_2b5ec6c574d6802c94c80313fb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_2b5ec6c574d6802c94c80313fb" ON public.clip USING btree ("userId");


--
-- Name: IDX_2c308dbdc50d94dc625670055f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_2c308dbdc50d94dc625670055f" ON public.signin USING btree ("userId");


--
-- Name: IDX_2c4be03b446884f9e9c502135b; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_2c4be03b446884f9e9c502135b" ON public.messaging_message USING btree ("groupId");


--
-- Name: IDX_2cd4a2743a99671308f5417759; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_2cd4a2743a99671308f5417759" ON public.blocking USING btree ("blockeeId");


--
-- Name: IDX_2da24ce20ad209f1d9dc032457; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_2da24ce20ad209f1d9dc032457" ON public.ad USING btree ("expiresAt");


--
-- Name: IDX_2e230dd45a10e671d781d99f3e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_2e230dd45a10e671d781d99f3e" ON public.channel_following USING btree ("followerId", "followeeId");


--
-- Name: IDX_307be5f1d1252e0388662acb96; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_307be5f1d1252e0388662acb96" ON public.following USING btree ("followerId", "followeeId");


--
-- Name: IDX_30bf67687f483ace115c5ca642; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_30bf67687f483ace115c5ca642" ON public.__chart__per_user_drive USING btree (date, "group");


--
-- Name: IDX_315c779174fe8247ab324f036e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_315c779174fe8247ab324f036e" ON public.drive_file USING btree ("isLink");


--
-- Name: IDX_318cdf42a9cfc11f479bd802bb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_318cdf42a9cfc11f479bd802bb" ON public.note_watching USING btree ("createdAt");


--
-- Name: IDX_3252a5df8d5bbd16b281f7799e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3252a5df8d5bbd16b281f7799e" ON public."user" USING btree (host);


--
-- Name: IDX_34500da2e38ac393f7bb6b299c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_34500da2e38ac393f7bb6b299c" ON public.instance USING btree ("isSuspended");


--
-- Name: IDX_347fec870eafea7b26c8a73bac; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_347fec870eafea7b26c8a73bac" ON public.hashtag USING btree (name);


--
-- Name: IDX_361b500e06721013c124b7b6c5; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_361b500e06721013c124b7b6c5" ON public.user_ip USING btree ("userId", ip);


--
-- Name: IDX_36cb699c49580d4e6c2e6159f9; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_36cb699c49580d4e6c2e6159f9" ON public.__chart__federation USING btree (date);


--
-- Name: IDX_36ef5192a1ce55ed0e40aa4db5; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_36ef5192a1ce55ed0e40aa4db5" ON public.antenna USING btree ("isActive");


--
-- Name: IDX_37bb9a1b4585f8a3beb24c62d6; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_37bb9a1b4585f8a3beb24c62d6" ON public.drive_file USING btree (md5);


--
-- Name: IDX_39ee857ab2f23493037c6b6631; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_39ee857ab2f23493037c6b6631" ON public.__chart__instance USING btree (date, "group");


--
-- Name: IDX_3aa8ea9a8f15214ad91638c0a7; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3aa8ea9a8f15214ad91638c0a7" ON public.flash USING btree ("updatedAt");


--
-- Name: IDX_3b25402709dd9882048c2bbade; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3b25402709dd9882048c2bbade" ON public.reversi_matching USING btree ("parentId");


--
-- Name: IDX_3b33dff77bb64b23c88151d23e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3b33dff77bb64b23c88151d23e" ON public.drive_file USING btree ("maybeSensitive");


--
-- Name: IDX_3befe6f999c86aff06eb0257b4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3befe6f999c86aff06eb0257b4" ON public.user_profile USING btree ("enableWordMute");


--
-- Name: IDX_3ca50563facd913c425e7a89ee; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3ca50563facd913c425e7a89ee" ON public.gallery_post USING btree ("fileIds");


--
-- Name: IDX_3d6b372788ab01be58853003c9; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3d6b372788ab01be58853003c9" ON public.user_group USING btree ("userId");


--
-- Name: IDX_3f5b0899ef90527a3462d7c2cb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3f5b0899ef90527a3462d7c2cb" ON public.app USING btree ("userId");


--
-- Name: IDX_3fcc2c589eaefc205e0714b99c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_3fcc2c589eaefc205e0714b99c" ON public.ad USING btree ("startsAt");


--
-- Name: IDX_410cd649884b501c02d6e72738; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_410cd649884b501c02d6e72738" ON public.user_note_pining USING btree ("userId", "noteId");


--
-- Name: IDX_42eb716a37d381cdf566192b2b; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_42eb716a37d381cdf566192b2b" ON public.__chart__notes USING btree (date);


--
-- Name: IDX_44499765eec6b5489d72c4253b; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_44499765eec6b5489d72c4253b" ON public.note_watching USING btree ("noteUserId");


--
-- Name: IDX_45145e4953780f3cd5656f0ea6; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_45145e4953780f3cd5656f0ea6" ON public.note_reaction USING btree ("noteId");


--
-- Name: IDX_47f4b1892f5d6ba8efb3057d81; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_47f4b1892f5d6ba8efb3057d81" ON public.note_favorite USING btree ("userId");


--
-- Name: IDX_48a00f08598662b9ca540521eb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_48a00f08598662b9ca540521eb" ON public.user_list USING btree ("isPublic");


--
-- Name: IDX_4ae7053179014915d1432d3f40; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_4ae7053179014915d1432d3f40" ON public.bubble_game_record USING btree ("seededAt");


--
-- Name: IDX_4bb7fd4a34492ae0e6cc8d30ac; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_4bb7fd4a34492ae0e6cc8d30ac" ON public.password_reset_request USING btree ("userId");


--
-- Name: IDX_4c02d38a976c3ae132228c6fce; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_4c02d38a976c3ae132228c6fce" ON public.hashtag USING btree ("mentionedRemoteUsersCount");


--
-- Name: IDX_4ccd2239268ebbd1b35e318754; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_4ccd2239268ebbd1b35e318754" ON public.following USING btree ("followerHost");


--
-- Name: IDX_4ce6fb9c70529b4c8ac46c9bfa; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_4ce6fb9c70529b4c8ac46c9bfa" ON public.page_like USING btree ("userId", "pageId");


--
-- Name: IDX_4e5c4c99175638ec0761714ab0; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_4e5c4c99175638ec0761714ab0" ON public.user_pending USING btree (code);


--
-- Name: IDX_4ebbf7f93cdc10e8d1ef2fc6cd; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_4ebbf7f93cdc10e8d1ef2fc6cd" ON public.abuse_user_report USING btree ("targetUserHost");


--
-- Name: IDX_4f4d35e1256c84ae3d1f0eab10; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_4f4d35e1256c84ae3d1f0eab10" ON public.emoji USING btree (name, host);


--
-- Name: IDX_5048e9daccbbbc6d567bb142d3; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_5048e9daccbbbc6d567bb142d3" ON public.__chart__per_user_notes USING btree (date, "group");


--
-- Name: IDX_50bd7164c5b78f1f4a42c4d21f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_50bd7164c5b78f1f4a42c4d21f" ON public.poll_vote USING btree ("userId", "noteId", choice);


--
-- Name: IDX_5108098457488634a4768e1d12; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_5108098457488634a4768e1d12" ON public.following USING btree (notify);


--
-- Name: IDX_52ccc804d7c69037d558bac4c9; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_52ccc804d7c69037d558bac4c9" ON public.note USING btree ("renoteId");


--
-- Name: IDX_5377c307783fce2b6d352e1203; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_5377c307783fce2b6d352e1203" ON public.messaging_message USING btree ("userId");


--
-- Name: IDX_539b6c08c05067599743bb6389; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_539b6c08c05067599743bb6389" ON public.role_assignment USING btree ("expiresAt");


--
-- Name: IDX_55720b33a61a7c806a8215b825; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_55720b33a61a7c806a8215b825" ON public.drive_file USING btree ("userId", "folderId", id);


--
-- Name: IDX_56b0166d34ddae49d8ef7610bb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_56b0166d34ddae49d8ef7610bb" ON public.note_unread USING btree ("userId");


--
-- Name: IDX_5900e907bb46516ddf2871327c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_5900e907bb46516ddf2871327c" ON public.emoji USING btree (host);


--
-- Name: IDX_5a056076f76b2efe08216ba655; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_5a056076f76b2efe08216ba655" ON public.webhook USING btree (active);


--
-- Name: IDX_5b87d9d19127bd5d92026017a7; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_5b87d9d19127bd5d92026017a7" ON public.note USING btree ("userId");


--
-- Name: IDX_5cc8c468090e129857e9fecce5; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_5cc8c468090e129857e9fecce5" ON public.user_group_invitation USING btree ("userGroupId");


--
-- Name: IDX_5deb01ae162d1d70b80d064c27; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_5deb01ae162d1d70b80d064c27" ON public."user" USING btree ("usernameLower", host);


--
-- Name: IDX_603a7b1e7aa0533c6c88e9bfaf; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_603a7b1e7aa0533c6c88e9bfaf" ON public.announcement_read USING btree ("announcementId");


--
-- Name: IDX_60c4af1c19a7a75f1592f93b28; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_60c4af1c19a7a75f1592f93b28" ON public.flash_like USING btree ("userId");


--
-- Name: IDX_617a8fe225a6e701d89e02d2c7; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_617a8fe225a6e701d89e02d2c7" ON public.__chart_day__federation USING btree (date);


--
-- Name: IDX_62aa5047b5aec92524f24c701d; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_62aa5047b5aec92524f24c701d" ON public.__chart_day__per_user_drive USING btree (date, "group");


--
-- Name: IDX_62cb09e1129f6ec024ef66e183; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_62cb09e1129f6ec024ef66e183" ON public.auth_session USING btree (token);


--
-- Name: IDX_6446c571a0e8d0f05f01c78909; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_6446c571a0e8d0f05f01c78909" ON public.antenna USING btree ("userId");


--
-- Name: IDX_64c327441248bae40f7d92f34f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_64c327441248bae40f7d92f34f" ON public.access_token USING btree (hash);


--
-- Name: IDX_650b49c5639b5840ee6a2b8f83; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_650b49c5639b5840ee6a2b8f83" ON public.user_memo USING btree ("userId");


--
-- Name: IDX_6516c5a6f3c015b4eed39978be; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_6516c5a6f3c015b4eed39978be" ON public.following USING btree ("followerId");


--
-- Name: IDX_66ac4a82894297fd09ba61f3d3; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_66ac4a82894297fd09ba61f3d3" ON public.user_memo USING btree ("targetUserId");


--
-- Name: IDX_66d2bd2ee31d14bcc23069a89f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_66d2bd2ee31d14bcc23069a89f" ON public.poll_vote USING btree ("userId");


--
-- Name: IDX_67dc758bc0566985d1b3d39986; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_67dc758bc0566985d1b3d39986" ON public.user_group_joining USING btree ("userGroupId");


--
-- Name: IDX_6a57f051d82c6d4036c141e107; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_6a57f051d82c6d4036c141e107" ON public.note_unread USING btree ("noteChannelId");


--
-- Name: IDX_6d8084ec9496e7334a4602707e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_6d8084ec9496e7334a4602707e" ON public.channel_following USING btree ("followerId");


--
-- Name: IDX_6fc0ec357d55a18646262fdfff; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_6fc0ec357d55a18646262fdfff" ON public.clip_note USING btree ("noteId", "clipId");


--
-- Name: IDX_70ba8f6af34bc924fc9e12adb8; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_70ba8f6af34bc924fc9e12adb8" ON public.access_token USING btree (token);


--
-- Name: IDX_7125a826ab192eb27e11d358a5; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_7125a826ab192eb27e11d358a5" ON public.note USING btree ("userHost");


--
-- Name: IDX_75276757070d21fdfaf4c05290; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_75276757070d21fdfaf4c05290" ON public.bubble_game_record USING btree ("userId");


--
-- Name: IDX_78787741f9010886796f2320a4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_78787741f9010886796f2320a4" ON public.user_group_invite USING btree ("userId", "userGroupId");


--
-- Name: IDX_7aa72a5fe76019bfe8e5e0e8b7; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_7aa72a5fe76019bfe8e5e0e8b7" ON public.renote_muting USING btree ("muterId");


--
-- Name: IDX_7b8d9225168e962f94ea517e00; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_7b8d9225168e962f94ea517e00" ON public.announcement USING btree (silence);


--
-- Name: IDX_7eac97594bcac5ffcf2068089b; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_7eac97594bcac5ffcf2068089b" ON public.renote_muting USING btree ("muteeId");


--
-- Name: IDX_7f7f1c66f48e9a8e18a33bc515; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_7f7f1c66f48e9a8e18a33bc515" ON public.user_ip USING btree ("userId");


--
-- Name: IDX_7fa20a12319c7f6dc3aed98c0a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_7fa20a12319c7f6dc3aed98c0a" ON public.poll USING btree ("userHost");


--
-- Name: IDX_8063a0586ed1dfbe86e982d961; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_8063a0586ed1dfbe86e982d961" ON public.webhook USING btree ("on");


--
-- Name: IDX_80ca6e6ef65fb9ef34ea8c90f4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_80ca6e6ef65fb9ef34ea8c90f4" ON public."user" USING btree ("updatedAt");


--
-- Name: IDX_8125f950afd3093acb10d2db8a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_8125f950afd3093acb10d2db8a" ON public.channel_note_pining USING btree ("channelId");


--
-- Name: IDX_823bae55bd81b3be6e05cff438; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_823bae55bd81b3be6e05cff438" ON public.channel USING btree ("userId");


--
-- Name: IDX_8288151386172b8109f7239ab2; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_8288151386172b8109f7239ab2" ON public.announcement_read USING btree ("userId");


--
-- Name: IDX_8302bd27226605ece14842fb25; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_8302bd27226605ece14842fb25" ON public.channel_favorite USING btree ("userId");


--
-- Name: IDX_83f0862e9bae44af52ced7099e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_83f0862e9bae44af52ced7099e" ON public.promo_note USING btree ("userId");


--
-- Name: IDX_845254b3eaf708ae8a6cac3026; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_845254b3eaf708ae8a6cac3026" ON public.__chart__users USING btree (date);


--
-- Name: IDX_860fa6f6c7df5bb887249fba22; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_860fa6f6c7df5bb887249fba22" ON public.drive_file USING btree ("userId");


--
-- Name: IDX_89a29c9237b8c3b6b3cbb4cb30; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_89a29c9237b8c3b6b3cbb4cb30" ON public.note_unread USING btree ("isSpecified");


--
-- Name: IDX_8bdcd3dd2bddb78014999a16ce; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_8bdcd3dd2bddb78014999a16ce" ON public.drive_file USING btree ("maybePorn");


--
-- Name: IDX_8bfa548c2b31f9e07db113773e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_8bfa548c2b31f9e07db113773e" ON public.__chart_day__network USING btree (date);


--
-- Name: IDX_8d5afc98982185799b160e10eb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_8d5afc98982185799b160e10eb" ON public.instance USING btree (host);


--
-- Name: IDX_8f589cf056ff51f09d6096f645; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_8f589cf056ff51f09d6096f645" ON public.__chart_day__hashtag USING btree (date, "group");


--
-- Name: IDX_8fd5215095473061855ceb948c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_8fd5215095473061855ceb948c" ON public.gallery_like USING btree ("userId");


--
-- Name: IDX_90148bbc2bf0854428786bfc15; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_90148bbc2bf0854428786bfc15" ON public.page USING btree ("visibleUserIds");


--
-- Name: IDX_924fa71815cfa3941d003702a0; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_924fa71815cfa3941d003702a0" ON public.announcement_read USING btree ("userId", "announcementId");


--
-- Name: IDX_92779627994ac79277f070c91e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_92779627994ac79277f070c91e" ON public.drive_file USING btree ("userHost");


--
-- Name: IDX_93060675b4a79a577f31d260c6; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_93060675b4a79a577f31d260c6" ON public.muting USING btree ("muterId");


--
-- Name: IDX_9657d55550c3d37bfafaf7d4b0; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_9657d55550c3d37bfafaf7d4b0" ON public.promo_read USING btree ("userId");


--
-- Name: IDX_97754ca6f2baff9b4abb7f853d; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_97754ca6f2baff9b4abb7f853d" ON public.sw_subscription USING btree ("userId");


--
-- Name: IDX_985b836dddd8615e432d7043dd; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_985b836dddd8615e432d7043dd" ON public.gallery_post USING btree ("userId");


--
-- Name: IDX_98a1bc5cb30dfd159de056549f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_98a1bc5cb30dfd159de056549f" ON public.blocking USING btree ("blockerId", "blockeeId");


--
-- Name: IDX_9949557d0e1b2c19e5344c171e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_9949557d0e1b2c19e5344c171e" ON public.access_token USING btree ("userId");


--
-- Name: IDX_9b88250fc2fd009b8f1b5623ed; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_9b88250fc2fd009b8f1b5623ed" ON public.flash USING btree ("userId");


--
-- Name: IDX_NOTE_FILE_IDS; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_NOTE_FILE_IDS" ON public.note USING gin ("fileIds");


--
-- Name: IDX_NOTE_MENTIONS; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_NOTE_MENTIONS" ON public.note USING gin (mentions);


--
-- Name: IDX_NOTE_TAGS; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_NOTE_TAGS" ON public.note USING gin (tags);


--
-- Name: IDX_NOTE_VISIBLE_USER_IDS; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_NOTE_VISIBLE_USER_IDS" ON public.note USING gin ("visibleUserIds");


--
-- Name: IDX_a012eaf5c87c65da1deb5fdbfa; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_a012eaf5c87c65da1deb5fdbfa" ON public.clip_note USING btree ("noteId");


--
-- Name: IDX_a08ad074601d204e0f69da9a95; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_a08ad074601d204e0f69da9a95" ON public.moderation_log USING btree ("userId");


--
-- Name: IDX_a0cd75442dd10d0643a17c4a49; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_a0cd75442dd10d0643a17c4a49" ON public.__chart__test_unique USING btree (date, "group");


--
-- Name: IDX_a1efd3e0048a5f2793a47360dc; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_a1efd3e0048a5f2793a47360dc" ON public.__chart__network USING btree (date);


--
-- Name: IDX_a27b942a0d6dcff90e3ee9b5e8; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_a27b942a0d6dcff90e3ee9b5e8" ON public."user" USING btree ("usernameLower");


--
-- Name: IDX_a319e5dbf47e8a17497623beae; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_a319e5dbf47e8a17497623beae" ON public.__chart__test USING btree (date, "group");


--
-- Name: IDX_a3eac04ae2aa9e221e7596114a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_a3eac04ae2aa9e221e7596114a" ON public.clip USING btree ("lastClippedAt");


--
-- Name: IDX_a40b8df8c989d7db937ea27cf6; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_a40b8df8c989d7db937ea27cf6" ON public.drive_file USING btree (type);


--
-- Name: IDX_a42c93c69989ce1d09959df4cf; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_a42c93c69989ce1d09959df4cf" ON public.note_watching USING btree ("userId", "noteId");


--
-- Name: IDX_a7eba67f8b3fa27271e85d2e26; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_a7eba67f8b3fa27271e85d2e26" ON public.drive_file USING btree ("isSensitive");


--
-- Name: IDX_a7fd92dd6dc519e6fb435dd108; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_a7fd92dd6dc519e6fb435dd108" ON public.follow_request USING btree ("followerId");


--
-- Name: IDX_a848f66d6cec11980a5dd59582; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_a848f66d6cec11980a5dd59582" ON public.__chart_day__ap_request USING btree (date);


--
-- Name: IDX_a854e557b1b14814750c7c7b0c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_a854e557b1b14814750c7c7b0c" ON public."user" USING btree (token);


--
-- Name: IDX_a9021cc2e1feb5f72d3db6e9f5; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_a9021cc2e1feb5f72d3db6e9f5" ON public.abuse_user_report USING btree ("targetUserId");


--
-- Name: IDX_ad0c221b25672daf2df320a817; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_ad0c221b25672daf2df320a817" ON public.note_reaction USING btree ("userId", "noteId");


--
-- Name: IDX_ae1d917992dd0c9d9bbdad06c4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_ae1d917992dd0c9d9bbdad06c4" ON public.page USING btree ("userId");


--
-- Name: IDX_ae7aab18a2641d3e5f25e0c4ea; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_ae7aab18a2641d3e5f25e0c4ea" ON public.note_thread_muting USING btree ("userId", "threadId");


--
-- Name: IDX_aecfbd5ef60374918e63ee95fa; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_aecfbd5ef60374918e63ee95fa" ON public.poll_vote USING btree ("noteId");


--
-- Name: IDX_af639b066dfbca78b01a920f8a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_af639b066dfbca78b01a920f8a" ON public.page USING btree ("updatedAt");


--
-- Name: IDX_b0134ec406e8d09a540f818288; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_b0134ec406e8d09a540f818288" ON public.note_watching USING btree ("userId");


--
-- Name: IDX_b14489029e4b3aaf4bba5fb524; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_b14489029e4b3aaf4bba5fb524" ON public.__chart__test_grouped USING btree (date, "group");


--
-- Name: IDX_b1754a39d0b281e07ed7c078ec; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_b1754a39d0b281e07ed7c078ec" ON public.clip_favorite USING btree ("userId", "clipId");


--
-- Name: IDX_b37dafc86e9af007e3295c2781; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_b37dafc86e9af007e3295c2781" ON public.emoji USING btree (name);


--
-- Name: IDX_b6f93f2f30bdbb9a5ebdc7c718; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_b6f93f2f30bdbb9a5ebdc7c718" ON public.registration_ticket USING btree ("usedById");


--
-- Name: IDX_b77d4dd9562c3a899d9a286fcd; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_b77d4dd9562c3a899d9a286fcd" ON public.__chart__per_user_following USING btree (date, "group");


--
-- Name: IDX_b7fcefbdd1c18dce86687531f9; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_b7fcefbdd1c18dce86687531f9" ON public.user_list USING btree ("userId");


--
-- Name: IDX_b82c19c08afb292de4600d99e4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_b82c19c08afb292de4600d99e4" ON public.page USING btree (name);


--
-- Name: IDX_bb90d1956dafc4068c28aa7560; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_bb90d1956dafc4068c28aa7560" ON public.drive_file USING btree ("folderId");


--
-- Name: IDX_bc1afcc8ef7e9400cdc3c0a87e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_bc1afcc8ef7e9400cdc3c0a87e" ON public.announcement USING btree ("isActive");


--
-- Name: IDX_be623adaa4c566baf5d29ce0c8; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_be623adaa4c566baf5d29ce0c8" ON public."user" USING btree (uri);


--
-- Name: IDX_beba993576db0261a15364ea96; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_beba993576db0261a15364ea96" ON public.registration_ticket USING btree ("createdById");


--
-- Name: IDX_bf3a053c07d9fb5d87317c56ee; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_bf3a053c07d9fb5d87317c56ee" ON public.access_token USING btree (session);


--
-- Name: IDX_bfbc6305547539369fe73eb144; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_bfbc6305547539369fe73eb144" ON public.user_group_invitation USING btree ("userId");


--
-- Name: IDX_bfbc6f79ba4007b4ce5097f08d; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_bfbc6f79ba4007b4ce5097f08d" ON public.user_note_pining USING btree ("userId");


--
-- Name: IDX_c1fd1c3dfb0627aa36c253fd14; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_c1fd1c3dfb0627aa36c253fd14" ON public.muting USING btree ("expiresAt");


--
-- Name: IDX_c426394644267453e76f036926; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_c426394644267453e76f036926" ON public.note_thread_muting USING btree ("threadId");


--
-- Name: IDX_c5545d4b31cdc684034e33b81c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_c5545d4b31cdc684034e33b81c" ON public.__chart_day__per_user_notes USING btree (date, "group");


--
-- Name: IDX_c55b2b7c284d9fef98026fc88e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_c55b2b7c284d9fef98026fc88e" ON public.drive_file USING btree ("webpublicAccessKey");


--
-- Name: IDX_c71faf11f0a28a5c0bb506203c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_c71faf11f0a28a5c0bb506203c" ON public.channel_favorite USING btree ("userId", "channelId");


--
-- Name: IDX_c8cc87bd0f2f4487d17c651fbf; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_c8cc87bd0f2f4487d17c651fbf" ON public."user" USING btree ("lastActiveDate");


--
-- Name: IDX_cac14a4e3944454a5ce7daa514; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_cac14a4e3944454a5ce7daa514" ON public.messaging_message USING btree ("recipientId");


--
-- Name: IDX_cad6e07c20037f31cdba8a350c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_cad6e07c20037f31cdba8a350c" ON public.__chart_day__users USING btree (date);


--
-- Name: IDX_cc7c72974f1b2f385a8921f094; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_cc7c72974f1b2f385a8921f094" ON public.channel USING btree ("isArchived");


--
-- Name: IDX_cddcaf418dc4d392ecfcca842a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_cddcaf418dc4d392ecfcca842a" ON public.user_list_membership USING btree ("userListId");


--
-- Name: IDX_ce62b50d882d4e9dee10ad0d2f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_ce62b50d882d4e9dee10ad0d2f" ON public.following USING btree ("followeeId", "followerHost", "isFollowerHibernated");


--
-- Name: IDX_cfbfeeccb0cbedcd660b17eb07; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_cfbfeeccb0cbedcd660b17eb07" ON public.flash_like USING btree ("userId", "flashId");


--
-- Name: IDX_d3ca0db011b75ac2a940a2337d; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_d3ca0db011b75ac2a940a2337d" ON public.channel_favorite USING btree ("channelId");


--
-- Name: IDX_d4ebdef929896d6dc4a3c5bb48; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_d4ebdef929896d6dc4a3c5bb48" ON public.note USING btree ("threadId");


--
-- Name: IDX_d54a512b822fac7ed52800f6b4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_d54a512b822fac7ed52800f6b4" ON public.follow_request USING btree ("followerId", "followeeId");


--
-- Name: IDX_d54b653660d808b118e36c184c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_d54b653660d808b118e36c184c" ON public.__chart_day__per_user_reaction USING btree (date, "group");


--
-- Name: IDX_d57f9030cd3af7f63ffb1c267c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_d57f9030cd3af7f63ffb1c267c" ON public.hashtag USING btree ("attachedUsersCount");


--
-- Name: IDX_d5954f3df5e5e3bdfc3c03f390; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_d5954f3df5e5e3bdfc3c03f390" ON public.__chart_day__active_users USING btree (date);


--
-- Name: IDX_d5a1b83c7cab66f167e6888188; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_d5a1b83c7cab66f167e6888188" ON public."user" USING btree ("isExplorable");


--
-- Name: IDX_d6765a8c2a4c17c33f9d7f948b; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_d6765a8c2a4c17c33f9d7f948b" ON public.user_list_favorite USING btree ("userId", "userListId");


--
-- Name: IDX_d85a184c2540d2deba33daf642; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_d85a184c2540d2deba33daf642" ON public.drive_file USING btree ("accessKey");


--
-- Name: IDX_d908433a4953cc13216cd9c274; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_d908433a4953cc13216cd9c274" ON public.note_unread USING btree ("userId", "noteId");


--
-- Name: IDX_d9ecaed8c6dc43f3592c229282; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_d9ecaed8c6dc43f3592c229282" ON public.user_group_joining USING btree ("userId", "userGroupId");


--
-- Name: IDX_da522b4008a9f5d7743b87ad55; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_da522b4008a9f5d7743b87ad55" ON public.__chart__test_grouped USING btree (date) WHERE ("group" IS NULL);


--
-- Name: IDX_da795d3a83187e8832005ba19d; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_da795d3a83187e8832005ba19d" ON public.announcement USING btree ("forExistingUsers");


--
-- Name: IDX_dab383a36f3c9db4a0c9b02cf3; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_dab383a36f3c9db4a0c9b02cf3" ON public.__chart__test USING btree (date) WHERE ("group" IS NULL);


--
-- Name: IDX_db5b72c16227c97ca88734d5c2; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_db5b72c16227c97ca88734d5c2" ON public.role_assignment USING btree ("userId");


--
-- Name: IDX_dce530b98e454793dac5ec2f5a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_dce530b98e454793dac5ec2f5a" ON public.user_profile USING btree ("userHost");


--
-- Name: IDX_de22cd2b445eee31ae51cdbe99; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_de22cd2b445eee31ae51cdbe99" ON public.user_profile USING btree (substr((birthday)::text, 6, 5));


--
-- Name: IDX_df1b5f4099e99fb0bc5eae53b6; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_df1b5f4099e99fb0bc5eae53b6" ON public.gallery_like USING btree ("userId", "postId");


--
-- Name: IDX_e10924607d058004304611a436; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_e10924607d058004304611a436" ON public.user_group_invite USING btree ("userGroupId");


--
-- Name: IDX_e21cd3646e52ef9c94aaf17c2e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_e21cd3646e52ef9c94aaf17c2e" ON public.messaging_message USING btree ("createdAt");


--
-- Name: IDX_e247b23a3c9b45f89ec1299d06; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_e247b23a3c9b45f89ec1299d06" ON public.reversi_matching USING btree ("childId");


--
-- Name: IDX_e4849a3231f38281280ea4c0ee; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_e4849a3231f38281280ea4c0ee" ON public.__chart_day__per_user_following USING btree (date, "group");


--
-- Name: IDX_e4f3094c43f2d665e6030b0337; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_e4f3094c43f2d665e6030b0337" ON public.user_list_membership USING btree ("userId", "userListId");


--
-- Name: IDX_e56f4beac5746d44bc3e19c80d; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_e56f4beac5746d44bc3e19c80d" ON public.__chart__ap_request USING btree (date);


--
-- Name: IDX_e5848eac4940934e23dbc17581; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_e5848eac4940934e23dbc17581" ON public.drive_file USING btree (uri);


--
-- Name: IDX_e637cba4dc4410218c4251260e; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_e637cba4dc4410218c4251260e" ON public.note_unread USING btree ("noteId");


--
-- Name: IDX_e74022ce9a074b3866f70e0d27; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_e74022ce9a074b3866f70e0d27" ON public.drive_file USING btree ("thumbnailAccessKey");


--
-- Name: IDX_e9793f65f504e5a31fbaedbf2f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_e9793f65f504e5a31fbaedbf2f" ON public.user_group_invitation USING btree ("userId", "userGroupId");


--
-- Name: IDX_ebe99317bbbe9968a0c6f579ad; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_ebe99317bbbe9968a0c6f579ad" ON public.clip_note USING btree ("clipId");


--
-- Name: IDX_ec96b4fed9dae517e0dbbe0675; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_ec96b4fed9dae517e0dbbe0675" ON public.muting USING btree ("muteeId");


--
-- Name: IDX_f0de67fd09cd3cd0aabca79994; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f0de67fd09cd3cd0aabca79994" ON public.role_assignment USING btree ("roleId");


--
-- Name: IDX_f22169eb10657bded6d875ac8f; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f22169eb10657bded6d875ac8f" ON public.note USING btree ("channelId");


--
-- Name: IDX_f221e45cfac5bea0ce0f3149fb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_f221e45cfac5bea0ce0f3149fb" ON public.__chart_day__per_user_pv USING btree (date, "group");


--
-- Name: IDX_f272c8c8805969e6a6449c77b3; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f272c8c8805969e6a6449c77b3" ON public.webhook USING btree ("userId");


--
-- Name: IDX_f2a56da57921ca8439f45c1d95; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_f2a56da57921ca8439f45c1d95" ON public.__chart__per_user_pv USING btree (date, "group");


--
-- Name: IDX_f2d744d9a14d0dfb8b96cb7fc5; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f2d744d9a14d0dfb8b96cb7fc5" ON public.gallery_post USING btree ("isSensitive");


--
-- Name: IDX_f36fed37d6d4cdcc68c803cd9c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_f36fed37d6d4cdcc68c803cd9c" ON public.channel_note_pining USING btree ("channelId", "noteId");


--
-- Name: IDX_f3a1b4bd0c7cabba958a0c0b23; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f3a1b4bd0c7cabba958a0c0b23" ON public.user_group_joining USING btree ("userId");


--
-- Name: IDX_f49922d511d666848f250663c4; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f49922d511d666848f250663c4" ON public.app USING btree (secret);


--
-- Name: IDX_f4fc06e49c0171c85f1c48060d; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f4fc06e49c0171c85f1c48060d" ON public.drive_folder USING btree ("userId");


--
-- Name: IDX_f631d37835adb04792e361807c; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f631d37835adb04792e361807c" ON public.gallery_post USING btree ("updatedAt");


--
-- Name: IDX_f7b9d338207e40e768e4a5265a; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f7b9d338207e40e768e4a5265a" ON public.instance USING btree ("firstRetrievedAt");


--
-- Name: IDX_f7c3576b37bd2eec966ae24477; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_f7c3576b37bd2eec966ae24477" ON public.retention_aggregation USING btree ("dateKey");


--
-- Name: IDX_f8d8b93740ad12c4ce8213a199; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_f8d8b93740ad12c4ce8213a199" ON public.abuse_user_report USING btree ("reporterHost");


--
-- Name: IDX_fa99d777623947a5b05f394cae; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_fa99d777623947a5b05f394cae" ON public."user" USING btree (tags);


--
-- Name: IDX_faef300913c738265638ba3ebc; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_faef300913c738265638ba3ebc" ON public.user_memo USING btree ("userId", "targetUserId");


--
-- Name: IDX_fb9d21ba0abb83223263df6bcb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_fb9d21ba0abb83223263df6bcb" ON public.registry_item USING btree ("userId");


--
-- Name: IDX_fcdafee716dfe9c3b5fde90f30; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_fcdafee716dfe9c3b5fde90f30" ON public.following USING btree ("followeeHost");


--
-- Name: IDX_fd25dfe3da37df1715f11ba6ec; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_fd25dfe3da37df1715f11ba6ec" ON public.announcement USING btree ("userId");


--
-- Name: IDX_fea7c0278325a1a2492f2d6acb; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE UNIQUE INDEX "IDX_fea7c0278325a1a2492f2d6acb" ON public.__chart_day__instance USING btree (date, "group");


--
-- Name: IDX_ff9ca3b5f3ee3d0681367a9b44; Type: INDEX; Schema: public; Owner: example-misskey-user
--

CREATE INDEX "IDX_ff9ca3b5f3ee3d0681367a9b44" ON public.user_security_key USING btree ("userId");


--
-- Name: drive_folder FK_00ceffb0cdc238b3233294f08f2; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.drive_folder
    ADD CONSTRAINT "FK_00ceffb0cdc238b3233294f08f2" FOREIGN KEY ("parentId") REFERENCES public.drive_folder(id) ON DELETE SET NULL;


--
-- Name: user_list_favorite FK_016f613dc4feb807e03e3e7da92; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_list_favorite
    ADD CONSTRAINT "FK_016f613dc4feb807e03e3e7da92" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_list_membership FK_021015e6683570ae9f6b0c62bee; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_list_membership
    ADD CONSTRAINT "FK_021015e6683570ae9f6b0c62bee" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: note_watching FK_03e7028ab8388a3f5e3ce2a8619; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_watching
    ADD CONSTRAINT "FK_03e7028ab8388a3f5e3ce2a8619" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: abuse_user_report FK_04cc96756f89d0b7f9473e8cdf3; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.abuse_user_report
    ADD CONSTRAINT "FK_04cc96756f89d0b7f9473e8cdf3" FOREIGN KEY ("reporterId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: blocking FK_0627125f1a8a42c9a1929edb552; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.blocking
    ADD CONSTRAINT "FK_0627125f1a8a42c9a1929edb552" FOREIGN KEY ("blockerId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: abuse_user_report FK_08b883dd5fdd6f9c4c1572b36de; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.abuse_user_report
    ADD CONSTRAINT "FK_08b883dd5fdd6f9c4c1572b36de" FOREIGN KEY ("assigneeId") REFERENCES public."user"(id) ON DELETE SET NULL;


--
-- Name: note_favorite FK_0e00498f180193423c992bc4370; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_favorite
    ADD CONSTRAINT "FK_0e00498f180193423c992bc4370" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: channel_following FK_0e43068c3f92cab197c3d3cd86e; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_following
    ADD CONSTRAINT "FK_0e43068c3f92cab197c3d3cd86e" FOREIGN KEY ("followeeId") REFERENCES public.channel(id) ON DELETE CASCADE;


--
-- Name: page_like FK_0e61efab7f88dbb79c9166dbb48; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.page_like
    ADD CONSTRAINT "FK_0e61efab7f88dbb79c9166dbb48" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_group_invite FK_1039988afa3bf991185b277fe03; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_invite
    ADD CONSTRAINT "FK_1039988afa3bf991185b277fe03" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: channel_note_pining FK_10b19ef67d297ea9de325cd4502; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_note_pining
    ADD CONSTRAINT "FK_10b19ef67d297ea9de325cd4502" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: user_publickey FK_10c146e4b39b443ede016f6736d; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_publickey
    ADD CONSTRAINT "FK_10c146e4b39b443ede016f6736d" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: follow_request FK_12c01c0d1a79f77d9f6c15fadd2; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.follow_request
    ADD CONSTRAINT "FK_12c01c0d1a79f77d9f6c15fadd2" FOREIGN KEY ("followeeId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: note_reaction FK_13761f64257f40c5636d0ff95ee; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_reaction
    ADD CONSTRAINT "FK_13761f64257f40c5636d0ff95ee" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: note FK_17cb3553c700a4985dff5a30ff5; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT "FK_17cb3553c700a4985dff5a30ff5" FOREIGN KEY ("replyId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: following FK_24e0042143a18157b234df186c3; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.following
    ADD CONSTRAINT "FK_24e0042143a18157b234df186c3" FOREIGN KEY ("followeeId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: clip_favorite FK_25a31662b0b0cc9af6549a9d711; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.clip_favorite
    ADD CONSTRAINT "FK_25a31662b0b0cc9af6549a9d711" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: note_thread_muting FK_29c11c7deb06615076f8c95b80a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_thread_muting
    ADD CONSTRAINT "FK_29c11c7deb06615076f8c95b80a" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: clip FK_2b5ec6c574d6802c94c80313fb2; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.clip
    ADD CONSTRAINT "FK_2b5ec6c574d6802c94c80313fb2" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: signin FK_2c308dbdc50d94dc625670055f7; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.signin
    ADD CONSTRAINT "FK_2c308dbdc50d94dc625670055f7" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: messaging_message FK_2c4be03b446884f9e9c502135be; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.messaging_message
    ADD CONSTRAINT "FK_2c4be03b446884f9e9c502135be" FOREIGN KEY ("groupId") REFERENCES public.user_group(id) ON DELETE CASCADE;


--
-- Name: blocking FK_2cd4a2743a99671308f5417759e; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.blocking
    ADD CONSTRAINT "FK_2cd4a2743a99671308f5417759e" FOREIGN KEY ("blockeeId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: reversi_matching FK_3b25402709dd9882048c2bbade0; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.reversi_matching
    ADD CONSTRAINT "FK_3b25402709dd9882048c2bbade0" FOREIGN KEY ("parentId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_group FK_3d6b372788ab01be58853003c93; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group
    ADD CONSTRAINT "FK_3d6b372788ab01be58853003c93" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: app FK_3f5b0899ef90527a3462d7c2cb3; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.app
    ADD CONSTRAINT "FK_3f5b0899ef90527a3462d7c2cb3" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE SET NULL;


--
-- Name: note_reaction FK_45145e4953780f3cd5656f0ea6a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_reaction
    ADD CONSTRAINT "FK_45145e4953780f3cd5656f0ea6a" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: note_favorite FK_47f4b1892f5d6ba8efb3057d81a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_favorite
    ADD CONSTRAINT "FK_47f4b1892f5d6ba8efb3057d81a" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: password_reset_request FK_4bb7fd4a34492ae0e6cc8d30ac8; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.password_reset_request
    ADD CONSTRAINT "FK_4bb7fd4a34492ae0e6cc8d30ac8" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_list_favorite FK_4d52b20bfe32c8552e7a61e80d2; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_list_favorite
    ADD CONSTRAINT "FK_4d52b20bfe32c8552e7a61e80d2" FOREIGN KEY ("userListId") REFERENCES public.user_list(id) ON DELETE CASCADE;


--
-- Name: user_profile FK_51cb79b5555effaf7d69ba1cff9; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_profile
    ADD CONSTRAINT "FK_51cb79b5555effaf7d69ba1cff9" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: note FK_52ccc804d7c69037d558bac4c96; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT "FK_52ccc804d7c69037d558bac4c96" FOREIGN KEY ("renoteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: messaging_message FK_535def119223ac05ad3fa9ef64b; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.messaging_message
    ADD CONSTRAINT "FK_535def119223ac05ad3fa9ef64b" FOREIGN KEY ("fileId") REFERENCES public.drive_file(id) ON DELETE CASCADE;


--
-- Name: messaging_message FK_5377c307783fce2b6d352e1203b; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.messaging_message
    ADD CONSTRAINT "FK_5377c307783fce2b6d352e1203b" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: note_unread FK_56b0166d34ddae49d8ef7610bb9; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_unread
    ADD CONSTRAINT "FK_56b0166d34ddae49d8ef7610bb9" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user FK_58f5c71eaab331645112cf8cfa5; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT "FK_58f5c71eaab331645112cf8cfa5" FOREIGN KEY ("avatarId") REFERENCES public.drive_file(id) ON DELETE SET NULL;


--
-- Name: note FK_5b87d9d19127bd5d92026017a7b; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT "FK_5b87d9d19127bd5d92026017a7b" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_group_invitation FK_5cc8c468090e129857e9fecce5a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_invitation
    ADD CONSTRAINT "FK_5cc8c468090e129857e9fecce5a" FOREIGN KEY ("userGroupId") REFERENCES public.user_group(id) ON DELETE CASCADE;


--
-- Name: announcement_read FK_603a7b1e7aa0533c6c88e9bfafe; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.announcement_read
    ADD CONSTRAINT "FK_603a7b1e7aa0533c6c88e9bfafe" FOREIGN KEY ("announcementId") REFERENCES public.announcement(id) ON DELETE CASCADE;


--
-- Name: flash_like FK_60c4af1c19a7a75f1592f93b287; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.flash_like
    ADD CONSTRAINT "FK_60c4af1c19a7a75f1592f93b287" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: antenna FK_6446c571a0e8d0f05f01c789096; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.antenna
    ADD CONSTRAINT "FK_6446c571a0e8d0f05f01c789096" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_memo FK_650b49c5639b5840ee6a2b8f83e; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_memo
    ADD CONSTRAINT "FK_650b49c5639b5840ee6a2b8f83e" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: following FK_6516c5a6f3c015b4eed39978be5; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.following
    ADD CONSTRAINT "FK_6516c5a6f3c015b4eed39978be5" FOREIGN KEY ("followerId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: reversi_game FK_6649a4e8c5d5cf32fb03b5da9f6; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.reversi_game
    ADD CONSTRAINT "FK_6649a4e8c5d5cf32fb03b5da9f6" FOREIGN KEY ("user2Id") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_memo FK_66ac4a82894297fd09ba61f3d35; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_memo
    ADD CONSTRAINT "FK_66ac4a82894297fd09ba61f3d35" FOREIGN KEY ("targetUserId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: poll_vote FK_66d2bd2ee31d14bcc23069a89f8; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.poll_vote
    ADD CONSTRAINT "FK_66d2bd2ee31d14bcc23069a89f8" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_group_joining FK_67dc758bc0566985d1b3d399865; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_joining
    ADD CONSTRAINT "FK_67dc758bc0566985d1b3d399865" FOREIGN KEY ("userGroupId") REFERENCES public.user_group(id) ON DELETE CASCADE;


--
-- Name: user_note_pining FK_68881008f7c3588ad7ecae471cf; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_note_pining
    ADD CONSTRAINT "FK_68881008f7c3588ad7ecae471cf" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: flash_like FK_6c16fe0e93b7a1951eca624b76a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.flash_like
    ADD CONSTRAINT "FK_6c16fe0e93b7a1951eca624b76a" FOREIGN KEY ("flashId") REFERENCES public.flash(id) ON DELETE CASCADE;


--
-- Name: channel_following FK_6d8084ec9496e7334a4602707e1; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_following
    ADD CONSTRAINT "FK_6d8084ec9496e7334a4602707e1" FOREIGN KEY ("followerId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_profile FK_6dc44f1ceb65b1e72bacef2ca27; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_profile
    ADD CONSTRAINT "FK_6dc44f1ceb65b1e72bacef2ca27" FOREIGN KEY ("pinnedPageId") REFERENCES public.page(id) ON DELETE SET NULL;


--
-- Name: antenna FK_709d7d32053d0dd7620f678eeb9; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.antenna
    ADD CONSTRAINT "FK_709d7d32053d0dd7620f678eeb9" FOREIGN KEY ("userListId") REFERENCES public.user_list(id) ON DELETE CASCADE;


--
-- Name: bubble_game_record FK_75276757070d21fdfaf4c052909; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.bubble_game_record
    ADD CONSTRAINT "FK_75276757070d21fdfaf4c052909" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: renote_muting FK_7aa72a5fe76019bfe8e5e0e8b7d; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.renote_muting
    ADD CONSTRAINT "FK_7aa72a5fe76019bfe8e5e0e8b7d" FOREIGN KEY ("muterId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: renote_muting FK_7eac97594bcac5ffcf2068089b6; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.renote_muting
    ADD CONSTRAINT "FK_7eac97594bcac5ffcf2068089b6" FOREIGN KEY ("muteeId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: channel_note_pining FK_8125f950afd3093acb10d2db8a8; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_note_pining
    ADD CONSTRAINT "FK_8125f950afd3093acb10d2db8a8" FOREIGN KEY ("channelId") REFERENCES public.channel(id) ON DELETE CASCADE;


--
-- Name: channel FK_823bae55bd81b3be6e05cff4383; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel
    ADD CONSTRAINT "FK_823bae55bd81b3be6e05cff4383" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE SET NULL;


--
-- Name: announcement_read FK_8288151386172b8109f7239ab28; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.announcement_read
    ADD CONSTRAINT "FK_8288151386172b8109f7239ab28" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: channel_favorite FK_8302bd27226605ece14842fb25a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_favorite
    ADD CONSTRAINT "FK_8302bd27226605ece14842fb25a" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: drive_file FK_860fa6f6c7df5bb887249fba22e; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.drive_file
    ADD CONSTRAINT "FK_860fa6f6c7df5bb887249fba22e" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE SET NULL;


--
-- Name: gallery_like FK_8fd5215095473061855ceb948cf; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.gallery_like
    ADD CONSTRAINT "FK_8fd5215095473061855ceb948cf" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: muting FK_93060675b4a79a577f31d260c67; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.muting
    ADD CONSTRAINT "FK_93060675b4a79a577f31d260c67" FOREIGN KEY ("muterId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: promo_read FK_9657d55550c3d37bfafaf7d4b05; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.promo_read
    ADD CONSTRAINT "FK_9657d55550c3d37bfafaf7d4b05" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: sw_subscription FK_97754ca6f2baff9b4abb7f853dd; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.sw_subscription
    ADD CONSTRAINT "FK_97754ca6f2baff9b4abb7f853dd" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: gallery_post FK_985b836dddd8615e432d7043ddb; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.gallery_post
    ADD CONSTRAINT "FK_985b836dddd8615e432d7043ddb" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: access_token FK_9949557d0e1b2c19e5344c171e9; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.access_token
    ADD CONSTRAINT "FK_9949557d0e1b2c19e5344c171e9" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: channel FK_999da2bcc7efadbfe0e92d3bc19; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel
    ADD CONSTRAINT "FK_999da2bcc7efadbfe0e92d3bc19" FOREIGN KEY ("bannerId") REFERENCES public.drive_file(id) ON DELETE SET NULL;


--
-- Name: flash FK_9b88250fc2fd009b8f1b5623ed5; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.flash
    ADD CONSTRAINT "FK_9b88250fc2fd009b8f1b5623ed5" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: clip_note FK_a012eaf5c87c65da1deb5fdbfa3; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.clip_note
    ADD CONSTRAINT "FK_a012eaf5c87c65da1deb5fdbfa3" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: moderation_log FK_a08ad074601d204e0f69da9a954; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.moderation_log
    ADD CONSTRAINT "FK_a08ad074601d204e0f69da9a954" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: access_token FK_a3ff16c90cc87a82a0b5959e560; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.access_token
    ADD CONSTRAINT "FK_a3ff16c90cc87a82a0b5959e560" FOREIGN KEY ("appId") REFERENCES public.app(id) ON DELETE CASCADE;


--
-- Name: promo_read FK_a46a1a603ecee695d7db26da5f4; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.promo_read
    ADD CONSTRAINT "FK_a46a1a603ecee695d7db26da5f4" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: follow_request FK_a7fd92dd6dc519e6fb435dd108f; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.follow_request
    ADD CONSTRAINT "FK_a7fd92dd6dc519e6fb435dd108f" FOREIGN KEY ("followerId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: abuse_user_report FK_a9021cc2e1feb5f72d3db6e9f5f; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.abuse_user_report
    ADD CONSTRAINT "FK_a9021cc2e1feb5f72d3db6e9f5f" FOREIGN KEY ("targetUserId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: page FK_a9ca79ad939bf06066b81c9d3aa; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.page
    ADD CONSTRAINT "FK_a9ca79ad939bf06066b81c9d3aa" FOREIGN KEY ("eyeCatchingImageId") REFERENCES public.drive_file(id) ON DELETE CASCADE;


--
-- Name: meta FK_ab1bc0c1e209daa77b8e8d212ad; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.meta
    ADD CONSTRAINT "FK_ab1bc0c1e209daa77b8e8d212ad" FOREIGN KEY ("proxyAccountId") REFERENCES public."user"(id) ON DELETE SET NULL;


--
-- Name: page FK_ae1d917992dd0c9d9bbdad06c4a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.page
    ADD CONSTRAINT "FK_ae1d917992dd0c9d9bbdad06c4a" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: poll_vote FK_aecfbd5ef60374918e63ee95fa7; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.poll_vote
    ADD CONSTRAINT "FK_aecfbd5ef60374918e63ee95fa7" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: user FK_afc64b53f8db3707ceb34eb28e2; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT "FK_afc64b53f8db3707ceb34eb28e2" FOREIGN KEY ("bannerId") REFERENCES public.drive_file(id) ON DELETE SET NULL;


--
-- Name: note_watching FK_b0134ec406e8d09a540f8182888; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_watching
    ADD CONSTRAINT "FK_b0134ec406e8d09a540f8182888" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: gallery_like FK_b1cb568bfe569e47b7051699fc8; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.gallery_like
    ADD CONSTRAINT "FK_b1cb568bfe569e47b7051699fc8" FOREIGN KEY ("postId") REFERENCES public.gallery_post(id) ON DELETE CASCADE;


--
-- Name: registration_ticket FK_b6f93f2f30bdbb9a5ebdc7c7189; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.registration_ticket
    ADD CONSTRAINT "FK_b6f93f2f30bdbb9a5ebdc7c7189" FOREIGN KEY ("usedById") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_list FK_b7fcefbdd1c18dce86687531f99; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_list
    ADD CONSTRAINT "FK_b7fcefbdd1c18dce86687531f99" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: drive_file FK_bb90d1956dafc4068c28aa7560a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.drive_file
    ADD CONSTRAINT "FK_bb90d1956dafc4068c28aa7560a" FOREIGN KEY ("folderId") REFERENCES public.drive_folder(id) ON DELETE SET NULL;


--
-- Name: registration_ticket FK_beba993576db0261a15364ea96e; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.registration_ticket
    ADD CONSTRAINT "FK_beba993576db0261a15364ea96e" FOREIGN KEY ("createdById") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_group_invitation FK_bfbc6305547539369fe73eb144a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_invitation
    ADD CONSTRAINT "FK_bfbc6305547539369fe73eb144a" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_note_pining FK_bfbc6f79ba4007b4ce5097f08d6; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_note_pining
    ADD CONSTRAINT "FK_bfbc6f79ba4007b4ce5097f08d6" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: auth_session FK_c072b729d71697f959bde66ade0; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.auth_session
    ADD CONSTRAINT "FK_c072b729d71697f959bde66ade0" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: messaging_message FK_cac14a4e3944454a5ce7daa5142; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.messaging_message
    ADD CONSTRAINT "FK_cac14a4e3944454a5ce7daa5142" FOREIGN KEY ("recipientId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_list_membership FK_cddcaf418dc4d392ecfcca842a7; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_list_membership
    ADD CONSTRAINT "FK_cddcaf418dc4d392ecfcca842a7" FOREIGN KEY ("userListId") REFERENCES public.user_list(id) ON DELETE CASCADE;


--
-- Name: page_like FK_cf8782626dced3176038176a847; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.page_like
    ADD CONSTRAINT "FK_cf8782626dced3176038176a847" FOREIGN KEY ("pageId") REFERENCES public.page(id) ON DELETE CASCADE;


--
-- Name: channel_favorite FK_d3ca0db011b75ac2a940a2337d2; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.channel_favorite
    ADD CONSTRAINT "FK_d3ca0db011b75ac2a940a2337d2" FOREIGN KEY ("channelId") REFERENCES public.channel(id) ON DELETE CASCADE;


--
-- Name: poll FK_da851e06d0dfe2ef397d8b1bf1b; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.poll
    ADD CONSTRAINT "FK_da851e06d0dfe2ef397d8b1bf1b" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: role_assignment FK_db5b72c16227c97ca88734d5c2b; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.role_assignment
    ADD CONSTRAINT "FK_db5b72c16227c97ca88734d5c2b" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: auth_session FK_dbe037d4bddd17b03a1dc778dee; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.auth_session
    ADD CONSTRAINT "FK_dbe037d4bddd17b03a1dc778dee" FOREIGN KEY ("appId") REFERENCES public.app(id) ON DELETE CASCADE;


--
-- Name: user_group_invite FK_e10924607d058004304611a436a; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_invite
    ADD CONSTRAINT "FK_e10924607d058004304611a436a" FOREIGN KEY ("userGroupId") REFERENCES public.user_group(id) ON DELETE CASCADE;


--
-- Name: reversi_matching FK_e247b23a3c9b45f89ec1299d066; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.reversi_matching
    ADD CONSTRAINT "FK_e247b23a3c9b45f89ec1299d066" FOREIGN KEY ("childId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: promo_note FK_e263909ca4fe5d57f8d4230dd5c; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.promo_note
    ADD CONSTRAINT "FK_e263909ca4fe5d57f8d4230dd5c" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: note_unread FK_e637cba4dc4410218c4251260e4; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note_unread
    ADD CONSTRAINT "FK_e637cba4dc4410218c4251260e4" FOREIGN KEY ("noteId") REFERENCES public.note(id) ON DELETE CASCADE;


--
-- Name: clip_note FK_ebe99317bbbe9968a0c6f579adf; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.clip_note
    ADD CONSTRAINT "FK_ebe99317bbbe9968a0c6f579adf" FOREIGN KEY ("clipId") REFERENCES public.clip(id) ON DELETE CASCADE;


--
-- Name: muting FK_ec96b4fed9dae517e0dbbe0675c; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.muting
    ADD CONSTRAINT "FK_ec96b4fed9dae517e0dbbe0675c" FOREIGN KEY ("muteeId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: role_assignment FK_f0de67fd09cd3cd0aabca79994d; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.role_assignment
    ADD CONSTRAINT "FK_f0de67fd09cd3cd0aabca79994d" FOREIGN KEY ("roleId") REFERENCES public.role(id) ON DELETE CASCADE;


--
-- Name: note FK_f22169eb10657bded6d875ac8f9; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT "FK_f22169eb10657bded6d875ac8f9" FOREIGN KEY ("channelId") REFERENCES public.channel(id) ON DELETE CASCADE;


--
-- Name: webhook FK_f272c8c8805969e6a6449c77b3c; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.webhook
    ADD CONSTRAINT "FK_f272c8c8805969e6a6449c77b3c" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_group_joining FK_f3a1b4bd0c7cabba958a0c0b231; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_group_joining
    ADD CONSTRAINT "FK_f3a1b4bd0c7cabba958a0c0b231" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_keypair FK_f4853eb41ab722fe05f81cedeb6; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_keypair
    ADD CONSTRAINT "FK_f4853eb41ab722fe05f81cedeb6" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: drive_folder FK_f4fc06e49c0171c85f1c48060d2; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.drive_folder
    ADD CONSTRAINT "FK_f4fc06e49c0171c85f1c48060d2" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: reversi_game FK_f7467510c60a45ce5aca6292743; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.reversi_game
    ADD CONSTRAINT "FK_f7467510c60a45ce5aca6292743" FOREIGN KEY ("user1Id") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: registry_item FK_fb9d21ba0abb83223263df6bcb3; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.registry_item
    ADD CONSTRAINT "FK_fb9d21ba0abb83223263df6bcb3" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: clip_favorite FK_fce61c7986cee54393e79f1d849; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.clip_favorite
    ADD CONSTRAINT "FK_fce61c7986cee54393e79f1d849" FOREIGN KEY ("clipId") REFERENCES public.clip(id) ON DELETE CASCADE;


--
-- Name: announcement FK_fd25dfe3da37df1715f11ba6ec8; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.announcement
    ADD CONSTRAINT "FK_fd25dfe3da37df1715f11ba6ec8" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- Name: user_security_key FK_ff9ca3b5f3ee3d0681367a9b447; Type: FK CONSTRAINT; Schema: public; Owner: example-misskey-user
--

ALTER TABLE ONLY public.user_security_key
    ADD CONSTRAINT "FK_ff9ca3b5f3ee3d0681367a9b447" FOREIGN KEY ("userId") REFERENCES public."user"(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

