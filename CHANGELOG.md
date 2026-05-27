# Changelog

Toutes les évolutions notables de **Kesh** sont consignées dans ce fichier.

Le format suit la convention [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et la numérotation respecte [Semantic Versioning](https://semver.org/lang/fr/).

Le contenu est rédigé en français à destination des **fiduciaires, PME, indépendants et associations** suisses qui utilisent Kesh. Pour le détail technique commit-par-commit, consulter [l'historique Git](https://github.com/guycorbaz/kesh/commits/main) du projet.

---

## [Non publié]

Aucun changement post-v0.1.0 pour le moment.

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
