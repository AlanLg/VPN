INSERT INTO testing.ips(ip, user_id)
VALUES ($1, $2)
RETURNING *;