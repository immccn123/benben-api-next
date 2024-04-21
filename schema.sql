/*
 Navicat Premium Data Transfer

 Source Server         : lgfeed
 Source Server Type    : PostgreSQL
 Source Server Version : 120015
 Source Host           : localhost:5432
 Source Catalog        : lgfeed
 Source Schema         : public

 Target Server Type    : PostgreSQL
 Target Server Version : 120015
 File Encoding         : 65001

 Date: 13/04/2024 14:49:43
*/


-- ----------------------------
-- Sequence structure for feed_id_seq
-- ----------------------------
DROP SEQUENCE IF EXISTS "public"."feed_id_seq";
CREATE SEQUENCE "public"."feed_id_seq" 
INCREMENT 1
MINVALUE  1
MAXVALUE 2147483647
START 1
CACHE 1;

-- ----------------------------
-- Table structure for feed
-- ----------------------------
DROP TABLE IF EXISTS "public"."feed";
CREATE TABLE "public"."feed" (
  "id" int4 NOT NULL DEFAULT nextval('feed_id_seq'::regclass),
  "user_id" int4 NOT NULL,
  "user_color" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "username" varchar(255) COLLATE "pg_catalog"."default" NOT NULL,
  "time" timestamp(6) NOT NULL,
  "content" varchar(8192) COLLATE "pg_catalog"."default" NOT NULL,
  "hash" varchar(512) COLLATE "pg_catalog"."default" NOT NULL,
  "grub_time" timestamp(6) NOT NULL
)
;

-- ----------------------------
-- Alter sequences owned by
-- ----------------------------
ALTER SEQUENCE "public"."feed_id_seq"
OWNED BY "public"."feed"."id";
SELECT setval('"public"."feed_id_seq"', 3267645, true);

-- ----------------------------
-- Indexes structure for table feed
-- ----------------------------
CREATE UNIQUE INDEX "feed_hash" ON "public"."feed" USING btree (
  "hash" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST
);
CREATE INDEX "idx_user_id_time" ON "public"."feed" USING btree (
  "user_id" "pg_catalog"."int4_ops" ASC NULLS LAST,
  "time" "pg_catalog"."timestamp_ops" DESC NULLS FIRST
);

-- ----------------------------
-- Primary Key structure for table feed
-- ----------------------------
ALTER TABLE "public"."feed" ADD CONSTRAINT "feed_pkey" PRIMARY KEY ("id");
