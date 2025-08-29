# PhlopChain Docker Setup

## Quick Start

Build and run with Docker Compose:

```bash
docker-compose up --build
```

The application will be available at: http://localhost:3030

## Manual Docker Commands

Build the image:
```bash
docker build -t phlopchain .
```

Run the container:
```bash
docker run -p 3030:3030 phlopchain
```

## Development

To rebuild after code changes:
```bash
docker-compose down
docker-compose up --build
```
