CREATE SCHEMA rsvp;

-- if uuid-ossp extension is not installed, install it
-- for uuid_generate_v4()
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- if btree_gist extension is not installed, install it
-- for gist index
CREATE EXTENSION IF NOT EXISTS "btree_gist";
