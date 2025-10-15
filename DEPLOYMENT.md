# WOPlanner Deployment Guide

## Build & Distribution Optionen

WOPlanner kann über Nix als natives Package oder als OCI-Container (Docker/Podman kompatibel) verteilt werden.

## Voraussetzungen

### SQLx Query Cache
Da SQLx compile-time checked queries verwendet, muss vor dem Build ein Query-Cache generiert werden:

```bash
# SQLx CLI installieren (einmalig)
cargo install sqlx-cli --no-default-features --features sqlite

# Query Cache generieren
cargo sqlx prepare --workspace

# Cache zu git hinzufügen
git add .sqlx
```

**Wichtig:** Der `.sqlx` Ordner muss im Git Repository sein, damit Nix Builds funktionieren!

## 1. Nix Package Build

### Bauen
```bash
# Standard Build
nix build

# Binary testen
./result/bin/woplanner

# Direkt ausführen ohne Build
nix run
```

### Installation
```bash
# In lokales Profil installieren
nix profile install .

# Von GitHub installieren (nach Push)
nix profile install github:username/woplanner
```

## 2. OCI Container Image (mit Nix)

### Bauen
```bash
# Container Image bauen (kein Dockerfile nötig!)
nix build .#docker

# Ergebnis: result -> .../docker-image-woplanner.tar.gz
# Größe: ~25MB (komprimiert)
```

### In Docker/Podman laden
```bash
# Docker
docker load < result

# Podman
podman load < result

# Image Liste zeigen
docker images | grep woplanner
```

### Container starten
```bash
# Einfacher Start (ohne Persistenz)
docker run -p 3000:3000 woplanner:latest

# Mit Volume für Datenbank
docker run -p 3000:3000 \
  -v woplanner-data:/data \
  -e SEED_DATABASE=false \
  woplanner:latest

# Mit Custom Port
docker run -p 8080:8080 \
  -v woplanner-data:/data \
  -e PORT=8080 \
  woplanner:latest
```

### Docker Compose
```yaml
version: '3.8'
services:
  woplanner:
    image: woplanner:latest
    ports:
      - "3000:3000"
    volumes:
      - woplanner-data:/data
    environment:
      - DATABASE_URL=sqlite:/data/woplanner.db
      - SEED_DATABASE=false
      - PORT=3000
    restart: unless-stopped

volumes:
  woplanner-data:
```

## 3. NixOS Module (für NixOS Systeme)

Das Projekt ist vorbereitet für ein NixOS Module. Beispiel-Konfiguration:

```nix
# In deiner configuration.nix oder als Modul
{ inputs, ... }: {
  imports = [ inputs.woplanner.nixosModules.default ];

  services.woplanner = {
    enable = true;
    port = 3000;
    database = "/var/lib/woplanner/woplanner.db";
  };
}
```

**TODO:** NixOS Module noch nicht implementiert, aber flake.nix ist vorbereitet.

## Umgebungsvariablen

| Variable | Default | Beschreibung |
|----------|---------|--------------|
| `DATABASE_URL` | `sqlite:/data/woplanner.db` | SQLite Datenbank Pfad |
| `SEED_DATABASE` | `false` (prod), `true` (dev) | Sample-Daten laden |
| `PORT` | `3000` | HTTP Port |

## Nix Flake Struktur

```bash
# Verfügbare Outputs
nix flake show

# packages.default  - Produktions-Binary
# packages.docker   - OCI Container Image
# apps.default      - Direkt ausführen
# devShells.default - Entwicklungsumgebung
```

## Image Publishing

### GitHub Container Registry (GHCR)
```bash
# 1. Image bauen
nix build .#docker

# 2. In Docker laden
docker load < result

# 3. Tag für GHCR
docker tag woplanner:latest ghcr.io/username/woplanner:latest
docker tag woplanner:latest ghcr.io/username/woplanner:$(git describe --tags)

# 4. Push
docker push ghcr.io/username/woplanner:latest
docker push ghcr.io/username/woplanner:$(git describe --tags)
```

### Nix Binary Cache (Optional)
Für schnellere Builds können Binary Caches verwendet werden (z.B. Cachix).

## Vorteile dieser Architektur

### Nix-basierter Workflow
- **Reproduzierbare Builds:** Identisches Binary auf allen Systemen
- **Kein Dockerfile:** Weniger Maintenance, ein Build-System für alles
- **Minimale Images:** Nur echte Dependencies (~25MB compressed)
- **Atomic Rollbacks:** Einfaches Zurückrollen zu alten Versionen

### Container vs. Native
- **Container:** Plattformübergreifend, isoliert, gewohnte Deployment-Tools
- **Native Nix:** Direktes Binary, perfekt für NixOS, weniger Overhead

### Best Practices
- `Cargo.lock` und `.sqlx/` im Git Repository
- Seeds nur in Development (`SEED_DATABASE=true`)
- Volume für `/data` in Production
- Health Check Endpoint: `GET /health`

## Troubleshooting

### Build-Fehler: "Cargo.lock does not exist"
```bash
git add Cargo.lock
```

### Build-Fehler: "set DATABASE_URL to use query macros"
```bash
cargo sqlx prepare --workspace
git add .sqlx
```

### Container startet nicht
```bash
# Logs checken
docker logs <container-id>

# Volume Permissions prüfen
docker run -it woplanner:latest bash
ls -la /data
```

## Weitere Ressourcen

- [Nix Flakes](https://nixos.wiki/wiki/Flakes)
- [nixpkgs Rust Support](https://nixos.org/manual/nixpkgs/stable/#rust)
- [dockerTools.buildImage](https://nixos.org/manual/nixpkgs/stable/#sec-pkgs-dockerTools-buildImage)
- [SQLx Offline Mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)
