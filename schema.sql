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

-- ----------------------------
-- Table structure for takedown_batches
-- ----------------------------
CREATE TABLE "public"."takedown_batches" (
  "batch_id" SERIAL PRIMARY KEY,                          -- 批量操作的唯一标识
  "reason" varchar(1024) COLLATE "pg_catalog"."default" NOT NULL,  -- 下架原因
  "takedown_time" timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP, -- 下架时间
  "takedown_user_id" int4 NOT NULL                        -- 执行下架操作的用户 ID
);

-- ----------------------------
-- Table structure for feed_takedown
-- ----------------------------
CREATE TABLE "public"."feed_takedown" (
  "id" SERIAL PRIMARY KEY,                               -- 唯一标识
  "feed_id" int4 NOT NULL UNIQUE,                               -- 对应被下架的 feed 的 ID
  "batch_id" int4 NOT NULL,                              -- 批量操作的 ID
  CONSTRAINT "fk_feed_id" FOREIGN KEY ("feed_id") REFERENCES "public"."feed" ("id") ON DELETE CASCADE,
  CONSTRAINT "fk_batch_id" FOREIGN KEY ("batch_id") REFERENCES "public"."takedown_batches" ("batch_id") ON DELETE CASCADE
);

CREATE TABLE "public"."circle_mention" (
  "sender" int4 NOT NULL,
  "mentioned" int4 NOT NULL,
  "feed_id" int4 NOT NULL,
  "time" timestamp NOT NULL,
  PRIMARY KEY ("sender", "mentioned", "feed_id"),
  FOREIGN KEY ("feed_id") REFERENCES "public"."feed" ("id") ON DELETE CASCADE ON UPDATE CASCADE
)
;

CREATE INDEX ON "public"."circle_mention" (
  "feed_id"
);

CREATE INDEX ON "public"."circle_mention" (
  "sender",
  "time" DESC
);

CREATE INDEX ON "public"."circle_mention" (
  "mentioned",
  "time" DESC
);
