SELECT *
FROM testing.users
WHERE email = $1
AND password = $2;

