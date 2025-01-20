# chloria
Chloria is a cute chloroplast. She doesn't perform regular "photosynthesis" but trains OCR models with synthetic photos.

## Local development
Change to [`deployment-local`](./deployment-local/) directory:
```bash
cd ./deployment-local/
```

Create a Docker Compose `.env` file and populate its environment variables with the appropriate values:
```bash
cp local.env .env
vi .env
```

Start and get inside the container:
```bash
docker compose up --build --remove-orphans -d
docker compose exec chloria-backend bash
```
