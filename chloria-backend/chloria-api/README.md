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
- `${HASHED_SECRET}`: Has the format `$argon2id...`

Insert a new credential into the database:
```sql
INSERT INTO clients(authentication_method, authentication_registry) VALUES ('static', '${AUTHENTICATION_REGISTRY}');

-- Suppose the id of the new client is `1` but it can be any value
INSERT INTO client_credentials(id, api_key, api_secret) VALUES (1, '${API_KEY}', '${HASHED_SECRET}');
```
- `${AUTHENTICATION_REGISTRY}`: A unique value to distinguish it from other static clients (e.g., `power-user`, `first-subscriber`,...)
- `${API_KEY}`: Randomly generated and unique
- `${HASHED_SECRET}`: The hashed secret generated in the previous step
