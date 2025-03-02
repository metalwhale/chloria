## Manual guide
### Add a static client credential
Generate a hashed secret using Argon2:
```bash
apt update -y
apt install -y argon2
echo -n "${SECRET}" | argon2 ${SALT} -id
```
> Type:	    Argon2id\
> ...\
> Hash:	    ...\
> Encoded:  ${HASHED_SECRET}\
> ...
- `${SECRET}`: Your private secret
- `${SALT}`: A random salt value

Insert a new credential into the database:
```sql
INSERT INTO clients DEFAULT VALUES;

-- Suppose the id of the new client is `1` but it can be any value
INSERT INTO client_authentications(id, method, identifier) VALUES (1, 'static', '${IDENTIFIER}');
INSERT INTO client_credentials(id, api_key, api_secret) VALUES (1, '${API_KEY}', '${HASHED_SECRET}');
```
- `${IDENTIFIER}`: A unique value to distinguish it from other static clients (e.g., `power-user`, `first-subscriber`,...)
- `${API_KEY}`: Randomly generated and unique
- `${HASHED_SECRET}`: The hashed secret generated in the previous step
