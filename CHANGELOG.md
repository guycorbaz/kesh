# Changelog

Toutes les évolutions notables de **Kesh** sont consignées dans ce fichier.

Le format suit la convention [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et la numérotation respecte [Semantic Versioning](https://semver.org/lang/fr/).

Le contenu est rédigé en français à destination des **fiduciaires, PME, indépendants et associations** suisses qui utilisent Kesh. Pour le détail technique commit-par-commit, consulter [l'historique Git](https://github.com/guycorbaz/kesh/commits/main) du projet.

---

## [0.1.4] — Non publié

Hotfix UX consolidé suite à dogfooding live sur prod NAS Synology v0.1.3 : CRUD complet des comptes bancaires post-onboarding (le seul endpoint existant `POST /api/v1/onboarding/bank-account` refusait les appels post-onboarding), restructuration de la sidebar avec groupes collapsibles, ajout des 4 pages orphelines précédemment accessibles uniquement via URL directe, widget homepage avec soldes calculés.

### Added

- **CRUD `bank_accounts` post-onboarding** (`POST` / `PUT` / `DELETE` `/api/v1/bank-accounts`) — création, édition complète et soft-delete (archivage) accessibles depuis Administration → Comptes bancaires (Comptable+). Transition primary silencieuse atomique POST/PUT (l'ancien primary est démoté automatiquement avec audit `details_json.trigger = "primary_transition"`). Audit log à 3 actions (`bank_account.created`, `bank_account.updated`, `bank_account.archived`) cohérent CO Art. 958f.
- **Soft-delete via `archived` BOOLEAN** sur `bank_accounts` (migration `20260531000001_bank_accounts_archived.sql`, non-breaking) — préserve audit + historique de transactions. Toggle « Afficher les archivés » côté UI. Refus 412 sur archivage si transactions existent (`BANK_ACCOUNT_HAS_TRANSACTIONS`) ou si compte principal avec d'autres comptes actifs (`BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY`). Archivage du primary unique autorisé.
- **Solde calculé serveur-side** sur `GET /api/v1/bank-accounts` (champ `currentBalance: Decimal | null`) — agrégation `SUM(debit) - SUM(credit)` sur `journal_entry_lines` du `journal_account_id` lié. Affiché sur la page d'accueil (par compte + total liquidités) et sur la page Comptes bancaires. `null` si le compte n'a pas de `journal_account_id` configuré (lien plan comptable manquant).
- **Sidebar collapsible** via `<details>`/`<summary>` HTML natif (a11y intégrée) avec persistence de l'état via `localStorage` (SSR-safe). 3 groupes structurés : Quotidien (déplié par défaut), Mensuel (déplié), Administration (replié). Auto-expand du groupe contenant la route active (UX + screen reader).
- **5 pages orphelines ajoutées à la sidebar** (Administration) : Plan comptable, Exercices comptables, Comptes bancaires, Profils bancaires, Règles d'affectation. Précédemment accessibles uniquement via URL directe.
- **Guide de démarrage utilisateur** (`docs/user-guide/fr/getting-started.md`) avec section dédiée à la liaison `bank_account` ↔ plan comptable et au cas multi-comptes (sous-comptes auxiliaires 1030.001/1030.002).

### Changed

- **Widget « Comptes bancaires » homepage** : affiche les soldes calculés au lieu d'un CTA configuration. Retiré complètement du DOM si aucun compte bancaire n'existe (`{#if bankAccounts.length > 0}`). Total liquidités affiché en pied de carte si plusieurs comptes.
- **Sidebar restructurée** : entrée « Payer » (qui pointait vers `/bank-accounts` — nom trompeur car c'était la configuration, pas un flow paiement) renommée en « Comptes bancaires » et déplacée sous Administration. Items « Export global » / « Paramètres » / admin-only (« Utilisateurs », « Facturation ») fusionnés dans Administration plutôt que dispersés en groupes séparés.
- **Page `/settings` — section Comptes bancaires** : remplacement du bouton « Modifier » (qui affichait un toast `notYet()`) par un lien direct vers `/bank-accounts`. Texte d'aide explicite.

### Fixed

- **Cohérence cross-fichier du flag `archived`** (FINDING-1/2/6 Pass 3 Opus spec-validate) : la fonction repo `bank_accounts::find_primary` (utilisée par `routes/invoice_pdf.rs:83` pour le QR Bill) filtre désormais `archived = FALSE` — sans ce filtre, un primary archivé continuerait à servir d'IBAN pour les PDF de factures alors qu'il n'apparaît plus côté UI (état fantôme). Idem pour `set_journal_account_id_for_company`, `update_for_company`, `archive_for_company` — un PATCH/PUT/DELETE sur un compte archivé retourne désormais 404 anti-énumération (KF-002). 7 call sites cross-modules patchés (`bank_imports.rs:862, 1006` + `reconciliation.rs:349, 629, 1962, 2278, 2699`) — empêche la création de nouvelles `bank_transactions` ou réconciliations manuelles sur un compte archivé.
- **PATCH `/bank-accounts/{id}` (legacy 8-5a-zero) — cohérence audit log** : l'event `bank_account.updated` émis par le PATCH inclut désormais `details_json.trigger = "journal_account_link"` (cohérent avec le PUT qui émet `trigger = "full_update"`). Sans ce champ, un script audit qui filtre par trigger raterait toutes les écritures PATCH.

### Removed

- **Fonction `notYet()` dans `/settings`** (plus utilisée après remplacement du bouton « Modifier » par lien direct).

---

## [0.1.3] — 2026-05-31

Hotfix critique : déblocage des déploiements LAN strict HTTP-only (cookies session inutilisables sans HTTPS dans v0.1.2).

### Corrections

- **Cookies session sur HTTP-only LAN** (Issue #136) : avant v0.1.3, les cookies `kesh_access_token` et `kesh_refresh_token` portaient systématiquement le flag `Secure` en mode production (sauf en mode test E2E). Le browser refuse de stocker/envoyer un cookie `Secure` sur une connexion HTTP non-TLS, ce qui rendait Kesh inutilisable sur tout déploiement LAN privé sans HTTPS (domaine RFC 8375 `*.home.arpa`, NAS Synology derrière Traefik HTTP sans Let's Encrypt, etc.) : l'utilisateur pouvait se logger côté backend mais aucun cookie ne persistait, déclenchant une boucle d'erreurs 401 sur tous les calls subséquents. Le couplage à `KESH_TEST_MODE` (qui active aussi les endpoints dangereux `/api/v1/_test/*`) interdisait par ailleurs tout workaround propre. Désormais découplé via une variable d'environnement dédiée `KESH_COOKIE_SECURE` (défaut `true` — sécurité préservée). Les déploiements LAN HTTP-only peuvent passer à `false` avec warning explicite au boot ; les autres déploiements continuent en mode sécurisé sans changement.

### Ajouts

- **Variable d'environnement `KESH_COOKIE_SECURE`** : contrôle explicitement le flag `Secure` des cookies session, indépendamment du mode test E2E. Valeurs `true`/`1` (défaut) ou `false`/`0` ; toute autre valeur (`True`, `yes`, `on`, espaces, etc.) refuse le démarrage pour éviter toute ambiguïté. Documentée dans `.env.example` avec warning de sécurité explicite et alternative HTTPS recommandée. Le manuel administrateur ajoute une sous-section dédiée au déploiement LAN HTTP-only avec procédure et matrice de risque.

---

## [0.1.2] — 2026-05-31

Évolutions de l'expérience d'installation pour s'aligner sur les standards des applications self-hosted modernes (Jellyfin, Bitwarden, Vaultwarden) et URL HTTP standard.

### ⚠️ Action requise — upgrade v0.1.1 → v0.1.2

**Si vous avez changé votre mot de passe administrateur via l'UI depuis l'installation v0.1.0/v0.1.1**, vous devez **retirer `KESH_ADMIN_PASSWORD` de votre `.env`** AVANT de redémarrer en v0.1.2. Sinon, le password sera resetté au password de `.env` au prochain boot (mécanisme **Recovery break-glass** déclenché automatiquement quand un admin existe avec le même `KESH_ADMIN_USERNAME` mais un hash différent). Visible dans les logs Docker au démarrage : `🔓 Recovery effectué — RETIRER LES VARS DE .ENV`.

**Aucune action requise** si vous n'avez pas changé votre mot de passe administrateur depuis l'installation : le boot v0.1.2 détecte que le hash en base correspond toujours à `KESH_ADMIN_PASSWORD`, ne touche pas au password, et émet un simple warning (« retirer les vars de .env ») qui disparaîtra dès que vous les retirez.

### Ajouts

- **Onboarding self-service au 1er démarrage** : sur une nouvelle installation avec une base de données vide, Kesh affiche désormais un écran **« Bienvenue dans Kesh »** au lieu d'exiger une édition `.env` préalable. L'administrateur initial est créé via un formulaire web (`/setup`) — pattern conforme à Jellyfin/Bitwarden/Sonarr/Vaultwarden. L'écran de setup est automatiquement désactivé (`410 Gone`) dès qu'un administrateur existe. Variables `KESH_ADMIN_USERNAME` / `KESH_ADMIN_PASSWORD` deviennent **optionnelles** et conservent un double-usage : (a) **bootstrap déclaratif** (CI, Test, déploiements automatisés) si renseignées sur DB vide, (b) **recovery break-glass** si un administrateur existe avec ce username mais un mot de passe différent (cf. ci-dessous). **⚠️ Sécurité** : avant le 1er démarrage en production, bloquez l'accès réseau public — la personne qui touche `/setup` en premier devient administrateur. Recommandé : binder loopback `127.0.0.1` ou LAN privé en attendant la création du compte. (closes #121)

- **Recovery break-glass administrateur** : si vous perdez votre mot de passe administrateur, vous pouvez désormais le réinitialiser en renseignant `KESH_ADMIN_USERNAME` et `KESH_ADMIN_PASSWORD` dans `.env` puis en redémarrant le container. Le hash de l'administrateur correspondant est resetté en transaction atomique (avec rollback automatique si l'audit log fail), ses sessions actives (refresh tokens) sont révoquées, et un événement `admin_break_glass_reset` est enregistré dans le journal d'audit (conservation 10 ans Swiss CO Art. 958f). Un warning préventif est émis dans les logs avant l'UPDATE : « ⚠️ Recovery break-glass déclenché — si vous avez changé votre mdp via l'UI, votre mdp sera écrasé par KESH_ADMIN_PASSWORD ». Procédure complète step-by-step dans le manuel administrateur (section « J'ai oublié mon mot de passe administrateur »). **Pensez à retirer les variables `KESH_ADMIN_*` de votre `.env` après recovery** — un warning persistant le rappelle dans les logs à chaque boot tant qu'elles traînent. (closes #121)

### Modifié

- **Port d'écoute par défaut : 3000 → 80** : Kesh écoute désormais sur le port HTTP standard (80) au lieu de 3000. L'URL d'accès est `http://kesh.local` (sans suffixe `:port`), conforme à ce que les utilisateurs attendent d'une application web. Le container Docker tourne en root et peut donc bind ce port privilégié sans configuration supplémentaire ; le bind loopback `127.0.0.1` de la prod est conservé.

  **⚠️ Breaking de configuration — utilisateurs existants v0.1.1, choisissez l'une des deux procédures :**

  1. **Adopter le nouveau défaut 80** (recommandé) : retirer la ligne `KESH_PORT=` de votre `.env` si présente. Le mapping `docker-compose.{prod,dev}.yml` (`127.0.0.1:80:80` ou `80:80`) prend effet automatiquement au prochain `docker compose up`. URL d'accès : `http://localhost`.

  2. **Garder le port 3000** (si conflit port 80, ex. Synology DSM Web Station, ou mode `cargo run` natif sur Linux non-root) : conserver `KESH_PORT=3000` dans `.env` ET **éditer directement** `docker-compose.prod.yml` (ou `docker-compose.dev.yml` selon votre déploiement) pour remplacer la ligne du mapping :
     ```yaml
     # Avant :
     # - "127.0.0.1:80:80"
     # Après :
     - "127.0.0.1:3000:3000"
     ```
     ⚠️ Ne PAS utiliser `docker-compose.override.yml` pour surcharger : Docker Compose **concatène** les listes `ports:` (ne les remplace pas) — le mapping `80:80` resterait actif et échouerait sur un hôte avec port 80 occupé. L'édition directe est la procédure officielle (alignée avec le manuel administrateur, section « Changer le port d'écoute »). URL d'accès : `http://localhost:3000`.

  Le manuel administrateur (section « Changer le port d'écoute (conflit port 80, ex. Synology DSM) ») détaille les 4 options d'override (remap host, override KESH_PORT, IP dédiée macvlan, mode dev natif).

---

## [0.1.1] — 2026-05-29

Hotfix post-déploiement v0.1.0 : corrections et améliorations opérationnelles découvertes lors du premier déploiement en production sur NAS Synology. Cette release embarque les **2 stories critiques** de l'épic hotfix (logs fichier + déblocage du premier démarrage). Les stories restantes de l'épic (break-glass admin reset, port 80 par défaut) sont reportées à une release ultérieure.

### Ajouts

- **Logs fichier avec rotation** : en plus de la sortie standard (`docker logs`), Kesh peut désormais écrire ses logs dans un fichier avec rotation automatique (quotidienne, horaire, ou désactivée), conservés sur le disque et inclus dans le backup. Activé par défaut en production (répertoire `./log/`). Configurable via `KESH_LOG_FILE_PATH`, `KESH_LOG_FILE_ROTATION`, `KESH_LOG_FILE_MAX_FILES` et `KESH_LOG_FILE_FORMAT` (format lisible ou JSON structuré). (#119)

### Corrections

- **Premier démarrage débloqué (catch-22 onboarding)** : sur une nouvelle installation avec une base de données vide, il était impossible de terminer la configuration — l'utilisateur administrateur du `.env` n'était jamais créé tant qu'aucune entreprise n'existait, alors que la création d'une entreprise passe par un assistant qui exige justement d'être connecté. Désormais, au tout premier démarrage, Kesh crée automatiquement un compte administrateur **et** une entreprise provisoire, ce qui permet de se connecter immédiatement et de compléter l'onboarding. Une bannière non-bloquante rappelle, pendant l'assistant, que l'entreprise porte un nom provisoire jusqu'à la saisie des vraies coordonnées. (#120)

---

## [0.1.0] — 2026-05-27

Première version publique de Kesh. Cette release fournit un système comptable complet et conforme au Code des obligations suisse (Art. 957a et suivants) ainsi qu'à l'Ordonnance OLICo (RS 221.431), prêt pour un usage productif chez un indépendant, une PME ou un fiduciaire mono-poste.

### Ajouts

#### Comptabilité

- **Plan comptable suisse PME** intégré (Sterchi adapté) avec création et modification de comptes (numérotation libre, type Actif/Passif/Charge/Produit, comptes inactifs archivables).
- **Saisie d'écritures comptables** en partie double avec validation automatique de l'équilibre Débit = Crédit avant enregistrement.
- **Journaux comptables** : Banque, Caisse, Achats, Ventes, Opérations Diverses (OD) ; sélection automatique du journal selon le contexte.
- **Audit-trail complet** (table `audit_log`) : chaque écriture comptable créée, modifiée ou supprimée est tracée avec l'utilisateur, la date, et un snapshot direct des données — conforme à l'exigence OLICo Art. 9 (intégrité des supports modifiables).
- **Exercices comptables** : création et clôture d'exercices, blocage automatique des écritures sur exercice clôturé (impossibilité de modifier les écritures d'un exercice fermé).
- **Recherche et filtrage** des écritures par date, montant, journal, description (recherche full-text MariaDB).
- **Tooltips pédagogiques** sur les concepts comptables (Débit, Crédit, etc.) pour faciliter la prise en main par les utilisateurs non-comptables.

#### Facturation et QR Bill

- **Création de factures** avec lignes libres ou références au catalogue de produits.
- **QR Bill 2.2** conformes au standard SIX Interbank Clearing (norme suisse en vigueur depuis 2020), avec génération du Swiss QR Code intégré au PDF de la facture.
- **Téléchargement PDF** des factures validées, archivage local et envoi par email possible.
- **Échéancier des factures** : suivi des échéances, marquage manuel des paiements reçus, export CSV pour rapprochement bancaire.
- **Workflow facture** : brouillon → validée → payée (avec transition par marquage manuel ou réconciliation bancaire automatique).

#### Import bancaire et réconciliation

- **Import CAMT.053** (ISO 20022) — standard universel des relevés bancaires en Suisse et en Europe — avec parsing des transactions, références, soldes d'ouverture et clôture.
- **Import CSV multi-encodage** avec création de profils banque réutilisables (mapping colonnes, séparateurs, encoding UTF-8/Latin-1/Windows-1252, format de date, séparateur décimal). Support des particularités de chaque banque suisse (PostFinance, UBS, Raiffeisen, Banques Cantonales, etc.).
- **Détection automatique des doublons** : si le même fichier est importé deux fois, ou si les mêmes transactions apparaissent dans deux relevés qui se chevauchent, Kesh prévient et propose de fusionner ou ignorer.
- **Commit partiel** : si quelques lignes d'un fichier sont invalides (formats incorrects), les autres lignes correctes sont importées et un rapport détaillé liste les rejets ligne par ligne.
- **Avertissement de balance** : si le solde de clôture déclaré dans le fichier ne correspond pas au solde calculé, Kesh prévient et demande confirmation avant import.
- **Réconciliation manuelle** : sélection d'une transaction bancaire et d'une facture (ou groupe d'écritures) pour les marquer comme rapprochées.
- **Réconciliation automatique par règles** : définition de règles d'affectation (par libellé, montant, contrepartie) qui rapprochent automatiquement les nouvelles transactions importées. Suggestions multi-candidats si plusieurs règles matchent.
- **Split de transaction** : une transaction bancaire peut être éclatée en plusieurs écritures comptables (utile pour les versements groupés).

#### Rapports comptables

- **Bilan** : actif/passif à une date donnée avec sous-totaux par classe de compte.
- **Compte de résultat** : produits et charges sur une période avec calcul du résultat net.
- **Balance** : tous les comptes avec leurs soldes débiteurs/créditeurs, filtres par classe.
- **Journal** : chronologie complète des écritures sur une période.
- **Exports** : tous les rapports exportables en PDF (mise en page officielle) et CSV (analyse Excel/LibreOffice).

#### Export global de souveraineté

- **Export ZIP global** d'une company (Story 9-2b) : un seul ZIP contient tous les rapports comptables, toutes les factures PDF, tous les imports bancaires, le journal complet, avec un hash SHA-256 d'intégrité. Permet à tout moment de retrouver vos données dans un format consultable et vérifiable, indépendant de Kesh — garantie de souveraineté numérique.

#### Multi-utilisateurs et sécurité

- **Authentification JWT** avec refresh tokens et rotation automatique (durée d'expiration courte de 15 minutes compensée par rafraîchissement transparent côté frontend).
- **Rôles RBAC** : Administrateur (toutes opérations) et Utilisateur (saisie sans gestion des paramètres système).
- **Isolation multi-tenant stricte** : chaque utilisateur n'accède qu'aux données de sa propre société (`company_id`). Audit complet du scoping effectué Epic 7 (Story 7-1) sur toutes les requêtes API et SQL.
- **Politique de mot de passe** : minimum 12 caractères pour le compte administrateur initial (hardening Story 10-1), configurable pour les autres utilisateurs.
- **Rate limiting** sur la connexion : protection contre les attaques par force brute (5 tentatives échouées par IP avant blocage 30 minutes, paramétrable).
- **Sessions à expiration glissante** : durée d'inactivité de 15 minutes avant déconnexion automatique (paramétrable).

#### Sécurité durcie (Story 10-5)

- **Tokens en cookies HttpOnly + Secure + SameSite=Strict** : les tokens d'authentification (`access_token` JWT + `refresh_token` UUID) sont stockés dans des cookies inaccessibles au JavaScript du navigateur. Élimine la possibilité de vol immédiat des tokens via une faille XSS hypothétique (un script malveillant ne peut ni lire `document.cookie`, ni accéder à `localStorage`). Nouveau endpoint `GET /api/v1/auth/me` permet au frontend de restaurer l'identité utilisateur sans pouvoir décoder le JWT côté JS. Closes Issue [#41 [KF-002]](https://github.com/guycorbaz/kesh/issues/41).
- **Content-Security-Policy défensif** sur les réponses HTML : restreint les sources de scripts, styles, images, connexions ; bloque l'incrustation iframe (`frame-ancestors 'none'`) anti-clickjacking. Défense en profondeur même si l'app reste sans XSS connu.

> **Note pour les administrateurs** : en mode `KESH_TEST_MODE=true` (CI + dev local sur HTTP loopback `127.0.0.1`), le flag `Secure` est désactivé pour permettre les tests E2E sans certificat. En production HTTPS, le flag `Secure` est inconditionnellement actif.

#### Multilingue (FR / DE / IT / EN)

- **Interface utilisateur** disponible en **français, allemand, italien et anglais** — les 4 langues nationales suisses + anglais professionnel.
- **Messages d'erreur** localisés dans toutes les langues.
- **Messages système** (banner DB indisponible, notifications) en 4 langues.

#### Déploiement et opérations

- **Docker Compose** : déploiement standard via une image officielle `gcorbaz/kesh:latest` publiée sur Docker Hub. Documentation complète dans le manuel administrateur.
- **Synology DSM Container Manager** : support natif documenté pour le déploiement sur NAS Synology (DSM 7.2+, modèles x86_64), avec utilisation du Portail des applications DSM comme reverse proxy HTTPS (alternative simple à Nginx/Caddy/Traefik pour LAN-only).
- **Reverse proxy HTTPS** : exemples documentés pour Nginx, Caddy (Let's Encrypt automatique), Traefik (avec firewall applicatif rate-limiting + headers OWASP + plugin CrowdSec optionnel).
- **Healthcheck** `/health` DB-aware (Story 10-3) : retourne `{ status, db, version }` permettant aux orchestrateurs (Docker, Kubernetes, monitoring) de détecter l'état réel de la base de données.
- **Résilience frontend DB inaccessible** (Story 10-3) : si la base de données devient temporairement indisponible (redémarrage MariaDB, panne réseau), l'interface utilisateur reste utilisable en consultation et affiche une bannière "Base de données temporairement indisponible — réessai automatique en cours". Reprise transparente dès que la connexion est rétablie.
- **Migrations DB idempotentes** (Story 10-2) avec protection contre les downgrades silencieux corrupteurs : refus de boot si le binaire Kesh est plus ancien que la version de schéma actuellement déployée.

#### Backup et conformité légale

- **Procédure de backup `mariadb-dump`** documentée avec script bash, rotation 30 jours et hash SHA-256 d'intégrité (conforme OLICo Art. 9 al. 1 lit. b ch. 1).
- **Backup natif sur Synology DSM** (Story 10-4) : documentation complète pour Hyper Backup (incrémental versionné vers cloud ou HDD USB, chiffrement client-side AES-256, rotation Smart Recycle) et Snapshot Replication (Btrfs, recovery point-in-time < 1 minute). Stratégie 3-2-1 illustrée.
- **Procédure de restauration** documentée avec vérification d'intégrité SHA-256, arrêt propre de Kesh, restauration, redémarrage et vérification fonctionnelle.
- **Test de restauration périodique** documenté pour conformité OLICo Art. 10 al. 1 (test annuel obligatoire, procès-verbal conservé 10 ans).
- **Audit-trail des écritures** : chaque modification d'écriture est tracée avec utilisateur, date et snapshot — fournit la garantie d'intégrité requise par OLICo Art. 9 pour les supports modifiables.

#### Documentation

- **Manuel administrateur** (105 pages PDF, français) : installation, configuration, sauvegarde, mise à jour, sécurité, conformité légale suisse, dépannage. Public cible : administrateurs système, responsables DevOps, fiduciaires en self-hosting.
- **Manuel utilisateur** (français) : guide d'utilisation quotidienne pour les comptables et utilisateurs PME.
- **Brochure marketing** (français) : présentation commerciale courte pour découverte du produit.
- Versions DE / IT / EN des manuels prévues v0.2.

### Notes de cette release

- **Production prête, mais pré-1.0** : Kesh v0.1.0 est utilisable en production pour des installations individuelles ou de PME. Le label `0.x` signale que des évolutions sont encore prévues avant v1.0 (notamment TVA Suisse complète, multi-langue des manuels, et fonctionnalités avancées de fiduciaire multi-clients).
- **Pas de migration utilisateur de v0.0.x** : Kesh v0.1.0 est la première version publique. Aucune migration depuis une version antérieure n'est nécessaire ni supportée.
- **Limitations connues v0.1.0** : voir les [issues GitHub](https://github.com/guycorbaz/kesh/issues?q=is%3Aopen+label%3Aknown-failure) avec le label `known-failure`. Aucune ne bloque l'usage productif des fonctionnalités livrées.

### Licence

Kesh est distribué sous [licence EUPL 1.2](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12) (European Union Public Licence). Cette licence est compatible GPL et permet l'usage commercial, la modification et la redistribution.

---

## Conventions de versionnage

- **MAJOR** (`X.0.0`) : changement incompatible nécessitant action manuelle de l'administrateur (nouvelle migration breaking, changement d'API, refonte UI majeure).
- **MINOR** (`0.X.0`) : nouvelle fonctionnalité rétro-compatible (nouvel epic livré, nouveau module).
- **PATCH** (`0.0.X`) : correction de bug ou amélioration mineure rétro-compatible (sécurité, performance, ergonomie).

Voir la [politique de migration breaking](https://github.com/guycorbaz/kesh/blob/main/CLAUDE.md#migration-breaking-policy) pour le détail technique des migrations DB et de la protection downgrade.
