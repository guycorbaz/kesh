# Démarrage de Kesh avec Docker Compose

## Prérequis

- Docker & Docker Compose installés
- Port 3000 et 3306 disponibles sur la machine hôte

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
kesh-api    | 2026-04-24T12:00:00 INFO kesh_api: listening on 0.0.0.0:3000
```

### 3. Accéder à l'application

- **API:** http://localhost:3000
- **Frontend (si implémenté):** http://localhost:3000
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

### Erreur "port 3000 already in use"
Changez le port dans `.env` :
```env
KESH_PORT=3001
```
Et redémarrez.

### Base de données ne s'initialise pas
```bash
docker-compose down -v
docker-compose up --build
```

## Notes

- Base de données: MariaDB 11 (jammy)
- Runtime: Debian Bookworm Slim
- Rust: 1.85 (build stage uniquement)
- Node.js: 22 (build stage uniquement)
- Les données sont persistées dans le volume `mariadb_data`
