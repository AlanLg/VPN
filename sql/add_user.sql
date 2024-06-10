INSERT INTO testing.users(email, username, password)
VALUES ($1, $2, $3)
RETURNING users.id, users.email, users.username;
