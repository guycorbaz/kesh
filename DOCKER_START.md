# Démarrage de Kesh avec Docker Compose

## Prérequis

- Docker & Docker Compose installés
- Port 80 disponible sur la machine hôte (MariaDB n'est pas publiée : Kesh
  l'atteint par le réseau interne de Docker)

## Démarrage rapide

### 1. Préparer `.env` (obligatoire)

Sans `.env`, c'est d'abord **Compose** qui refuse, au terminal, en nommant
`MARIADB_ROOT_PASSWORD` ou `MARIADB_PASSWORD` (une seule à la fois, pas toujours
la même) : depuis la 0.13.0, `docker-compose.yml` n'a plus de mots de passe
MariaDB par défaut. Ensuite seulement, Kesh refuse le secret JWT par défaut de
`docker-compose.yml` (`change-me…`), à dessein.

```bash
cp .env.example .env
# Dans .env, remplacer la valeur de KESH_JWT_SECRET, puis décommenter
# MARIADB_ROOT_PASSWORD et MARIADB_PASSWORD et leur donner une valeur —
# chacune la sortie d'une commande :
openssl rand -hex 32
```

Le placeholder `<GENERATE_ME: …>` recopié tel quel est refusé au démarrage
(depuis la 0.13.0). Les deux mots de passe MariaDB ne sont lus qu'à la
**création** de la base : les changer ensuite dans `.env` ne change pas le mot
de passe enregistré (manuel d'administration, § *Changer un mot de passe
MariaDB*). Hexadécimal : une sortie base64 peut contenir `/`, qui casse une
`DATABASE_URL`.

### 2. Lancer les containers

```bash
docker compose up --build
```

**Options utiles:**
- `-d` : Lancer en arrière-plan
- `--pull always` : Tirer les images à jour
- `-v` pour plus de logs

Exemple :
```bash
docker compose up -d --build
```

⚠️ `docker-compose.yml` porte `image: gcorbaz/kesh:latest` **et** `build:` :
`docker compose up -d` **tire l'image publiée** ; `--build` construit depuis les
sources. Effet inverse : tout `up --build` **étiquette le build local
`gcorbaz/kesh:latest`**, que les `up -d` suivants emploient sans tirer — pour
revenir à l'image publiée : `docker compose pull kesh-api`.

### 3. Attendre que MariaDB soit prêt

Les logs vous indiqueront quand la base est prête :
```
kesh-mariadb | ... ready for connections
kesh-api    | 2026-04-24T12:00:00 INFO kesh_api: listening on 0.0.0.0:80
```

### 4. Accéder à l'application

- **API:** http://localhost
- **Frontend (si implémenté):** http://localhost
- **Admin initial :** aucun compte n'est créé d'office. Au premier accès,
  l'écran `/setup` crée l'administrateur ; ou, avant le premier démarrage,
  poser `KESH_ADMIN_USERNAME` / `KESH_ADMIN_PASSWORD` (≥ 12 caractères) dans
  `.env`.

## Gestion des containers

### Afficher les logs
```bash
docker compose logs -f kesh-api
docker compose logs -f mariadb
```

### Arrêter
```bash
docker compose down
```

### Arrêter et supprimer les données
```bash
docker compose down -v
```

### Redémarrer
```bash
docker compose restart
```

Après une modification de `.env` ou du compose : `docker compose up -d` — un
`restart` redémarre le conteneur avec son environnement d'origine et ne relit
pas `.env`. Seules les variables listées sous `environment:` du compose
atteignent Kesh.

## Développement

### Recompiler après modification du code Rust

```bash
docker compose up -d --build kesh-api
```

### Accéder à la base de données

```bash
docker compose exec mariadb mariadb -u kesh -p -D kesh
```

Le mot de passe (`MARIADB_PASSWORD` de `.env`) est demandé. Le port 3306 n'est
pas publié sur l'hôte : un port publié par Docker contourne le pare-feu (UFW).

### Voir les volumes créés

```bash
docker volume ls | grep kesh
```

## Troubleshooting

### Container ne démarre pas
```bash
docker compose logs kesh-api
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
Puis appliquez : `docker compose up -d`.

### Base de données ne démarre pas, ou Kesh ne s'y connecte pas

Diagnostic d'abord :
```bash
docker compose logs kesh-api
docker compose logs mariadb
```

- **Erreur 1045** (« Access denied ») ou l'**indice** de Kesh sur
  `MARIADB_PASSWORD` : le mot de passe de `.env` n'est pas celui avec lequel la
  base a été créée — MariaDB ne relit pas `.env`. Revenir à la valeur
  d'origine (manuel d'administration, § *Passer à la 0.13.0*), puis changer le
  mot de passe par la procédure du manuel (`ALTER USER` d'abord).
- **Compose refuse en nommant une variable** (`required variable … is missing
  a value`) : la poser dans `.env`.

⛔ `docker compose down -v` **supprime le volume de la base, définitivement** —
les livres avec. Ne l'employer que sur une installation **sans données à
conserver**, pour repartir d'une base vide :
```bash
docker compose down -v
docker compose up -d
```

## Notes

- Base de données: MariaDB 10.11 (parité prod NAS Synology, cf. Story 10-1 D3)
- Runtime: Debian Bookworm Slim
- Rust: 1.85 (build stage uniquement)
- Node.js: 22 (build stage uniquement)
- Les données sont persistées dans le volume `kesh-mariadb-data` du compose,
  que `docker volume ls` affiche `<projet>_kesh-mariadb-data` (préfixe du
  projet Compose, par défaut le nom du répertoire)
