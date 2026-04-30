# Serve JS browser demos (bench scenarios, workers, etc.) with COOP/COEP headers.
#
# Build from repo root:
#   docker build -f js/Dockerfile -t nxs-js js
# Run:
#   docker run --rm -p 8000:8000 nxs-js
#
# Open http://localhost:8000/

FROM python:3.12-slim-bookworm

WORKDIR /app

COPY . .

EXPOSE 8000

CMD ["python3", "server.py", "8000"]
