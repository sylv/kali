CREATE TABLE user_profiles (
    user_id INTEGER PRIMARY KEY,
    bio TEXT,
    colour INTEGER DEFAULT 0,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) STRICT;