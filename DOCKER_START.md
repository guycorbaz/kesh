# Démarrage de Kesh avec Docker Compose

## Prérequis

- Docker & Docker Compose installés
- Port 80 et 3306 disponibles sur la machine hôte

## Démarrage rapide

### 1. Lancer les containers

```bash
docker-compose up --build
```

**Options utiles:**
- `-d` : Lancer en arrière-plan
- `--pull always` : Tirer les images à jour
- `-v` pour plus de logs

Exemple :
```bash
docker-compose up -d --build
```

### 2. Attendre que MariaDB soit prêt

Les logs vous indiqueront quand la base est prête :
```
kesh-mariadb | ... ready for connections
kesh-api    | 2026-04-24T12:00:00 INFO kesh_api: listening on 0.0.0.0:80
```

### 3. Accéder à l'application

- **API:** http://localhost
- **Frontend (si implémenté):** http://localhost
- **Admin initial:**
  - Nom: `admin`
  - Mot de passe: `admin`

## Gestion des containers

### Afficher les logs
```bash
docker-compose logs -f kesh-api
docker-compose logs -f mariadb
```

### Arrêter
```bash
docker-compose down
```

### Arrêter et supprimer les données
```bash
docker-compose down -v
```

### Redémarrer
```bash
docker-compose restart
```

## Développement

### Recompiler après modification du code Rust

```bash
docker-compose up -d --build kesh-api
```

### Accéder à la base de données

```bash
docker-compose exec mariadb mysql -u kesh -pkesh_dev -D kesh
```

### Voir les volumes créés

```bash
docker volume ls | grep kesh
```

## Troubleshooting

### Container ne démarre pas
```bash
docker-compose logs kesh-api
```

### Erreur "port 80 already in use"
Le port 80 par défaut est occupé sur l'hôte (souvent Synology DSM Web Station,
nginx local, IIS…). Garder Kesh sur le port 80 **côté container** et remapper
**côté host** est la solution la plus simple : éditer `docker-compose.yml` et
changer le mapping `"80:80"` vers `"8080:80"` (ou tout autre HOST_PORT libre) :
```yaml
ports:
  - "8080:80"
```
Pas de modification de `KESH_PORT` nécessaire. URL d'accès : `http://localhost:8080`.

Cf. `.env.example` section "Conflit port 80" et le manuel admin section
"Changer le port d'écoute" pour les autres options d'override
(macvlan IP dédiée, dev `cargo run` natif sur Linux non-root, etc.).
Et redémarrez.

### Base de données ne s'initialise pas
```bash
docker-compose down -v
docker-compose up --build
```

## Notes

- Base de données: MariaDB 10.11 (parité prod NAS Synology, cf. Story 10-1 D3)
- Runtime: Debian Bookworm Slim
- Rust: 1.85 (build stage uniquement)
- Node.js: 22 (build stage uniquement)
- Les données sont persistées dans le volume `mariadb_data`
