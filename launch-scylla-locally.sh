#!/usr/bin/env bash
#
# This is a shell script used to run a Online Division
# style scylla keyspace on your device.
#
# Run it as follows:
#  ./launch-scylla-locally.sh
#
# And then connect to localhost:9042 to interact
# with your scylla cluster.


CREATE_USERS="
CREATE KEYSPACE division_online WITH replication = {'class': 'org.apache.cassandra.locator.SimpleStrategy', 'replication_factor': '1'} AND durable_writes = true AND tablets = {'enabled': false};

CREATE TABLE division_online.users (
    username text,
    bio text,
    email text,
    key text,
    password_hash text,
    password_salt text,
    pfp text,
    user_salt text,
    PRIMARY KEY (username)
) WITH bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_user_tokens (
    username text,
    datetime timestamp,
    key text,
    PRIMARY KEY (username, datetime, key)
) WITH CLUSTERING ORDER BY (datetime ASC, key ASC)
    AND bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_user_server_roles (
    server_id text,
    username text,
    role_name text,
    PRIMARY KEY (server_id, username, role_name)
) WITH CLUSTERING ORDER BY (username ASC, role_name ASC)
    AND bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_user_friends (
    username text,
    friend text,
    created_at timestamp,
    PRIMARY KEY (username, friend)
) WITH CLUSTERING ORDER BY (friend ASC)
    AND bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_servers (
    sid text,
    desc text,
    img_url text,
    name text,
    owner text,
    settings text,
    PRIMARY KEY (sid)
) WITH bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_server_users (
    sid text,
    username text,
    PRIMARY KEY (sid, username)
) WITH CLUSTERING ORDER BY (username ASC)
    AND bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_server_roles (
    server_id text,
    role_name text,
    color text,
    permissions set<text>,
    PRIMARY KEY (server_id, role_name)
) WITH CLUSTERING ORDER BY (role_name ASC)
    AND bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_server_messages_migration (
    sid text,
    channel_name text,
    datetime timestamp,
    m_content text,
    mid text,
    username text,
    PRIMARY KEY (sid, channel_name, datetime)
) WITH CLUSTERING ORDER BY (channel_name ASC, datetime DESC)
    AND bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_server_messages (
    mid text,
    channel_name text,
    datetime timestamp,
    m_content text,
    sid text,
    username text,
    PRIMARY KEY (mid)
) WITH bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_server_channels (
    sid text,
    channel_name text,
    PRIMARY KEY (sid, channel_name)
) WITH CLUSTERING ORDER BY (channel_name ASC)
    AND bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_dm_invites (
    u1 text,
    u2 text,
    invite_id text,
    sender text,
    PRIMARY KEY ((u1, u2))
) WITH bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};

CREATE TABLE division_online.o_spell_caster_secrets (
    key text,
    spell text,
    username text,
    PRIMARY KEY (key, spell)
) WITH CLUSTERING ORDER BY (spell ASC)
    AND bloom_filter_fp_chance = 0.01
    AND caching = {'keys': 'ALL', 'rows_per_partition': 'ALL'}
    AND comment = ''
    AND compaction = {'class': 'SizeTieredCompactionStrategy'}
    AND compression = {'sstable_compression': 'org.apache.cassandra.io.compress.LZ4Compressor'}
    AND crc_check_chance = 1
    AND default_time_to_live = 0
    AND gc_grace_seconds = 864000
    AND max_index_interval = 2048
    AND memtable_flush_period_in_ms = 0
    AND min_index_interval = 128
    AND speculative_retry = '99.0PERCENTILE'
    AND tombstone_gc = {'mode': 'timeout', 'propagation_delay_in_seconds': '3600'};
"

echo " * Killing running scylla containers."
docker kill scylla-division-online

echo " * Removing old scylla containers."
docker rm scylla-division-online

echo " * Running scylladb/scylla image."
docker run --name scylla-division-online -d scylladb/scylla --reactor-backend=epoll 

echo " * Waiting for docker container to settle."
sleep 5

echo " * Creating scylla keyspaces and tables."
echo "${CREATE_USERS}" | docker exec -i scylla-division-online cqlsh
