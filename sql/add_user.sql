INSERT INTO testing.users(email, username, password, public_key, private_key)
VALUES ($1, $2, $3, $4, $5)
RETURNING users.email, users.username, users.public_key, users.private_key;
