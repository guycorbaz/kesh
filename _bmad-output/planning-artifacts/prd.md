---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
  - step-12-complete
inputDocuments:
  - docs/kesh-prd-v0.2.md
  - docs/change_request.md
documentCounts:
  briefs: 0
  research: 0
  brainstorming: 0
  projectDocs: 2
workflowType: 'prd'
classification:
  projectType: Web App (SPA Svelte + API Axum)
  domain: Comptabilité & gestion PME suisse (mini-ERP)
  complexity: high
  projectContext: brownfield
---

# Product Requirements Document - Kesh

**Author:** Guy
**Date:** 2026-04-01

## Résumé Exécutif

Kesh est un logiciel de comptabilité et de gestion destiné aux indépendants, TPE et petites associations en Suisse. Gratuit et open source (EUPL 1.2), il s'auto-héberge sur l'infrastructure de l'utilisateur — NAS, VPS ou serveur local — via Docker. Les données comptables restent intégralement sous le contrôle de l'utilisateur, sans transmission à un tiers.

L'application web implémente nativement les standards suisses : facturation QR Bill 2.2, paiements pain.001.001.09.ch.03, import de relevés CAMT.053.001.04, plan comptable helvétique et TVA suisse à taux configurables avec historique. L'interface est disponible en français, allemand, italien et anglais.

Face aux solutions cloud par abonnement et aux logiciels locaux vieillissants, Kesh offre une alternative moderne, gratuite et souveraine. Les standards bancaires suisses ISO 20022 sont aujourd'hui matures et Docker démocratise l'auto-hébergement : le moment est opportun pour une solution open source qui n'existait pas jusqu'ici. L'installation via docker-compose et un onboarding assisté permettent d'être opérationnel en moins de 15 minutes.

### Ce qui rend Kesh spécial

- **Gratuit et open source** — Aucun abonnement. Code source ouvert sous licence EUPL 1.2. Seul le support est payant.
- **Auto-hébergé** — Les données comptables restent sur l'infrastructure de l'utilisateur. Aucune donnée transmise à un tiers.
- **Standards suisses natifs** — QR Bill, pain.001, CAMT.053, plan comptable PME/association, TVA configurable. Conçu pour la Suisse dès le premier jour.
- **Multilingue natif** — FR, DE, IT, EN. L'interface et les documents sont dans la langue de l'instance.
- **Pensé pour des utilisateurs sans formation comptable spécialisée** — Onboarding assisté, plan comptable pré-configuré, aide contextuelle intégrée.

## Classification du projet

| Critère | Valeur |
|---|---|
| **Type** | Web App (SPA Svelte + API REST Axum) |
| **Domaine** | Comptabilité & gestion PME suisse |
| **Complexité** | Haute — standards SIX/ISO 20022, TVA suisse, comptabilité en partie double |
| **Contexte** | Brownfield — PRD v0.2 existant, 3 change requests, 50+ décisions structurantes |
| **Licence** | EUPL 1.2 |

## Critères de succès

### Succès utilisateur

- Préparer et émettre une facture QR Bill (lignes libres ou produits/services, validation, génération PDF) en moins de 5 minutes
- Importer un relevé bancaire (CAMT.053/CSV) et réconcilier les transactions connues en quelques clics
- Clôturer un exercice et produire un bilan/compte de résultat sans assistance externe
- Être opérationnel (onboarding + première saisie) en moins de 15 minutes depuis le premier lancement
- Établir un budget annuel et suivre les écarts avec le réalisé tout au long de l'exercice
- Utiliser le logiciel sans formation comptable spécialisée grâce à l'aide contextuelle

### Succès business

- L'auteur gère sa propre comptabilité effective dans Kesh pendant au moins un exercice complet (12 mois)
- Le logiciel fonctionne de manière fiable pour une utilisation quotidienne en conditions réelles
- Des utilisateurs adoptent Kesh pour leur comptabilité effective (pas uniquement en test)
- Le projet open source attire des retours constructifs de la communauté

### Succès technique

- **Intégrité comptable** — La balance débit/crédit est toujours juste, aucun écart possible
- **Zéro perte de données** — Écritures, factures et justificatifs toujours intègres
- **Intégrité des imports** — Aucune transaction en doublon suite à un import multiple ou un chevauchement de relevés
- **Standards conformes** — Les fichiers pain.001 passent la validation du schéma XSD officiel SIX. Les QR Bill sont conformes aux spécifications SIX.
- **Performance** — Import d'un relevé mensuel (200 transactions) en < 2s, pages < 300ms
- **Disponibilité** — L'application redémarre proprement après un crash ou une coupure
- **Sécurité** — Aucune donnée accessible sans authentification valide. Mots de passe hashés (argon2/bcrypt), jamais stockés en clair.

### Résultats mesurables

- Un indépendant peut gérer sa comptabilité annuelle complète dans Kesh : saisie, facturation, paiements, import bancaire, TVA, clôture
- Les PDF QR Bill générés sont conformes aux spécifications SIX et scannables
- Les fichiers pain.001 passent la validation XSD officielle SIX
- Le décompte TVA généré correspond aux montants vérifiés manuellement
- Le bilan et le compte de résultat générés correspondent aux résultats vérifiés manuellement sur un jeu de données de référence

## Parcours utilisateurs

### Parcours 1 — Marc, graphiste indépendant à Lausanne (utilisateur principal)

**Scène d'ouverture :** Marc est graphiste indépendant depuis 3 ans. Chaque mois, il paie un abonnement pour un logiciel comptable cloud qu'il utilise à 10% de ses capacités. Il envoie 5-10 factures par mois, importe son relevé PostFinance, et une fois par an il exporte ses chiffres pour son fiduciaire. Il aimerait quelque chose de simple, gratuit, et hébergé sur son NAS Synology.

**Action montante :** Marc découvre Kesh. Il copie trois lignes dans son docker-compose, lance l'application. L'assistant d'onboarding lui demande : « Vous êtes indépendant ? ». Il saisit ses coordonnées, son QR-IBAN, et Kesh installe le plan comptable adapté. En 10 minutes, il est prêt.

Il crée son premier client dans le carnet d'adresses, prépare une facture avec deux lignes depuis son catalogue de prestations (« Création logo — 1'500 CHF », « Retouches photo — 500 CHF »), et valide. Le PDF QR Bill s'ouvre dans un nouvel onglet. Le QR Code est conforme. Il télécharge et envoie par email.

En début d'année, Marc saisit son budget prévisionnel : revenus attendus par mois, charges fixes (loyer atelier, assurances), charges variables (logiciels, matériel). Chaque trimestre, il consulte le rapport budget vs réalisé et ajuste ses dépenses.

En fin de mois, il importe son relevé PostFinance (CAMT.053). Les paiements de ses factures sont automatiquement proposés pour réconciliation. Les frais bancaires et un paiement carte arrivent dans la file — il crée les contreparties en deux clics. Kesh lui propose de créer des règles pour la prochaine fois.

**Scénario d'avoir :** Un client de Marc conteste une facture validée de 1'500 CHF — le travail livré ne correspondait pas à la demande. Marc ne peut pas supprimer la facture validée. Il crée un avoir depuis la séquence dédiée, lié à la facture d'origine. L'avoir génère automatiquement l'écriture de contre-passation. Le solde du client revient à zéro.

**Scénario d'erreur :** Marc prépare un lot de paiements fournisseurs et génère le fichier pain.001. Mais il a saisi un IBAN invalide pour un fournisseur. Kesh valide le format IBAN avant la génération et signale l'erreur : « IBAN invalide pour le fournisseur Müller — paiement exclu du lot ». Marc corrige l'IBAN dans le carnet d'adresses et régénère le fichier. Les autres paiements du lot ne sont pas impactés.

**Climax :** En fin de trimestre, Marc génère son rapport TVA. Les montants correspondent. Il remplit le formulaire sur le portail de l'AFC en 5 minutes. En fin d'année, il clôture l'exercice. Le bilan et le compte de résultat sont générés en PDF. Le rapport budget vs réalisé annuel montre qu'il a dépassé son budget matériel de 15% — il en tient compte pour le budget de l'année suivante. Son fiduciaire valide les comptes sans remarque.

**Résolution :** Marc ne paie plus d'abonnement. Ses données sont sur son NAS. Sa comptabilité lui prend 30 minutes par mois au lieu de 2 heures. Il recommande Kesh à ses collègues indépendants.

### Parcours 2 — Sophie, trésorière d'une association sportive à Berne (non-comptable, association)

**Scène d'ouverture :** Sophie est bénévole comme trésorière du club de volleyball de son quartier. Elle n'a aucune formation comptable. Jusqu'ici, elle gérait les comptes sur un tableur Excel — erreurs fréquentes, pas de bilan propre pour l'AG.

**Action montante :** Le président de l'association installe Kesh sur le serveur du club. Il crée deux comptes : un admin pour lui, un comptable pour Sophie. Sophie se connecte et découvre un plan comptable « association » pré-configuré. Les tooltips l'aident à comprendre les termes comptables.

Elle commence sa première saisie d'écriture. « Débit ? Crédit ? » — elle hésite. Le tooltip lui explique : « Débit = l'argent entre dans ce compte, Crédit = l'argent sort de ce compte ». Elle saisit les cotisations des membres, les factures du gymnase, les recettes de la buvette. Pour le gymnase, elle crée le fournisseur dans le carnet d'adresses et saisit la facture reçue avec le justificatif PDF scanné.

En début d'année, Sophie saisit le budget voté par l'AG : cotisations attendues, subventions, dépenses prévues (location gymnase, équipement, tournois). Elle crée une version « Budget AG » qui servira de référence.

**Scénario d'écriture déséquilibrée :** Sophie saisit une écriture de 500 CHF au débit mais oublie la contrepartie crédit. Kesh refuse la validation : « Écriture déséquilibrée — le total des débits (500.00) ne correspond pas au total des crédits (0.00) ». Sophie comprend et ajoute la ligne crédit manquante.

**Scénario de doublons :** Sophie importe le relevé bancaire de décembre, mais elle avait déjà importé les derniers jours de décembre avec le relevé de novembre. Kesh détecte les doublons et les signale — aucune transaction en double. Elle souffle de soulagement.

**Scénario de concurrence :** Le trésorier adjoint de l'association se connecte en même temps que Sophie. Il modifie le libellé d'une écriture que Sophie est en train de corriger. Sophie sauvegarde — Kesh affiche une modal : « Cette écriture a été modifiée par un autre utilisateur. Voulez-vous recharger ? ». Sophie recharge et voit la modification du trésorier adjoint. Elle applique sa correction par-dessus. Pas de perte de données.

**Scénario de session expirée :** Sophie part en pause café. 20 minutes plus tard, elle clique sur « Sauvegarder » — une modal apparaît : « Session expirée — veuillez vous reconnecter ». Elle se reconnecte. Le formulaire qu'elle remplissait est perdu — elle devra le ressaisir. Une leçon apprise : sauvegarder régulièrement.

**Climax :** À l'AG, Sophie présente le bilan, le compte de résultat ET le comparatif budget vs réalisé générés par Kesh. Les membres voient immédiatement que les dépenses d'équipement ont dépassé le budget de 800 CHF, compensées par des recettes de buvette meilleures que prévu. C'est la première fois que les comptes sont clairs et professionnels. Le budget de l'année suivante est voté sur la base du réalisé.

**Résolution :** Sophie gère désormais les comptes de l'association avec confiance. Le président, en mode consultation, peut vérifier les soldes à tout moment sans risque de modifier quoi que ce soit.

### Parcours 3 — Thomas, administrateur système (admin/ops)

**Scène d'ouverture :** Thomas est informaticien. Un ami indépendant et une association locale lui demandent d'installer Kesh. Il gère deux instances sur son VPS.

**Action montante :** Thomas crée deux docker-compose distincts, chacun avec sa propre base MariaDB. Il configure les variables d'environnement (`KESH_ADMIN_PASSWORD`, `KESH_LANG`), le reverse proxy nginx avec TLS, et lance les containers. L'endpoint `/health` lui confirme que tout est opérationnel.

Il crée les comptes utilisateurs pour chaque instance — un admin et un comptable pour l'ami indépendant, un admin, un comptable et un compte consultation pour l'association. Il configure la politique de mot de passe (12 caractères minimum).

**Scénario de changement de mot de passe :** Le comptable de l'ami indépendant souhaite changer son mot de passe initial. Depuis son profil, il saisit l'ancien mot de passe, puis le nouveau. Kesh vérifie la politique de mot de passe (12 caractères minimum) et confirme le changement par une banner de succès.

**Scénario de mise à jour :** Thomas télécharge la nouvelle image Docker. Au redémarrage, Kesh détecte la nouvelle version et affiche « Backup recommandé avant de continuer ». Thomas fait un dump MariaDB, confirme, et les migrations s'appliquent. Mais l'instance de l'association ne démarre pas — le healthcheck `/health` retourne 503. Les logs Docker montrent une erreur de connexion MariaDB : le container de base de données n'a pas fini de redémarrer. Thomas attend 10 secondes, le healthcheck repasse au vert. Fausse alerte, mais les logs étaient clairs.

Un jour, le comptable de l'association oublie son mot de passe. Thomas le réinitialise depuis l'interface admin. Le trésorier quitte l'association — Thomas désactive son compte. L'historique de ses actions reste intact.

**Résolution :** Thomas maintient deux instances Kesh avec un effort minimal. Les backups MariaDB sont automatisés via son NAS. Les mises à jour se font en 5 minutes.

### Parcours 4 — Lisa, comptable dans une fiduciaire à Zurich (utilisateur avancé)

**Scène d'ouverture :** Lisa travaille dans une petite fiduciaire à Zurich. Elle gère la comptabilité de 8 clients indépendants et TPE. Chacun a sa propre instance Kesh installée par leur informaticien.

**Action montante :** Lisa se connecte à l'instance de chaque client avec son compte « Comptable ». Elle connaît bien Kesh — les raccourcis, les règles d'affectation. Pour chaque client, elle importe le relevé bancaire mensuel. Les règles d'affectation qu'elle a créées au fil des mois font que 80% des transactions sont pré-affectées. Elle valide en lot, traite les exceptions manuellement.

Elle prépare les paiements fournisseurs pour un client : sélectionne les factures ouvertes, génère le fichier pain.001. Le client l'upload dans son e-banking.

**Scénario d'import partiel :** Lisa importe un fichier CSV d'une banque en ligne pour un nouveau client. Le format est différent de ce qu'elle connaît — les colonnes ne correspondent pas au profil standard. Kesh rejette partiellement l'import : « 12 transactions importées, 3 rejetées — format de date non reconnu en lignes 45, 67, 89 ». Lisa consulte les lignes en erreur, crée un profil banque personnalisé pour ce format, et réimporte les 3 lignes manquantes.

**Climax :** Fin d'année. Lisa clôture les exercices de ses 8 clients. Pour chacun : rapport TVA du dernier trimestre, écriture de clôture, report des soldes, génération du bilan et du compte de résultat. Les PDF sont conformes et professionnels. Ses clients sont satisfaits.

**Résolution :** Lisa recommande Kesh à ses clients parce que c'est gratuit pour eux et qu'elle travaille efficacement. La fiduciaire propose le support Kesh comme service payant.

### Synthèse des capacités révélées par les parcours

| Parcours | Capacités clés | Scénarios d'erreur couverts |
|---|---|---|
| **Marc (indépendant)** | Onboarding, catalogue, facturation QR Bill, import CAMT, réconciliation, règles d'affectation, TVA, budget, clôture, rapports | Validation IBAN pain.001, création d'avoir sur facture contestée |
| **Sophie (association)** | Multi-utilisateurs/rôles, plan comptable association, justificatifs, aide contextuelle, budget AG, mode consultation | Écriture déséquilibrée, doublons import, verrouillage optimiste, session expirée |
| **Thomas (admin)** | Docker-compose, multi-instances, healthcheck, gestion utilisateurs, politique MdP, migration/mise à jour, désactivation compte | Healthcheck 503, changement de mot de passe |
| **Lisa (fiduciaire)** | Efficacité experte, règles d'affectation, batch paiements, profils banque, clôture multi-clients | Import CSV partiel, profil banque personnalisé |

**Correspondance tests E2E Playwright :** Chaque parcours et scénario d'erreur constitue directement un scénario de test de bout en bout.

## Exigences spécifiques au domaine

### Conformité & réglementaire

- **Code des obligations suisse (CO art. 957-964)** — Intégrité des écritures, conservation obligatoire 10 ans. Jamais de suppression automatique de données comptables, même archivées.
- **Standards SIX** — QR Bill 2.2 (conformité stricte : dimensions 46x46mm, position A4, police, croix suisse), pain.001.001.09.ch.03 (validation XSD), CAMT.053.001.04
- **TVA suisse (AFC)** — Taux configurables avec historique et dates de validité, méthode effective/forfaitaire, arrondis commercial au centime
- **Numéro IDE (CHE)** — Validation format et checksum sur le carnet d'adresses (CHE-123.456.789)
- **nLPD** — Post-MVP (export/anonymisation des données personnelles)

### Contraintes techniques

- **Arithmétique exacte** — `rust_decimal` pour tous les montants, jamais de flottants (f64)
- **Intégrité comptable** — Balance débit/crédit toujours équilibrée, validation à chaque saisie
- **Immutabilité post-clôture** — Aucune modification des écritures après clôture d'exercice, uniquement contre-passation
- **Formats suisses** — Apostrophe comme séparateur de milliers (`1'234.56`) dans tous les PDF et rapports, dates en `dd.mm.yyyy`
- **Encoding** — Support UTF-8 et ISO-8859-1 pour l'import CSV (détection automatique)
- **Sécurité** — Hash des mots de passe (argon2/bcrypt), JWT avec refresh silencieux
- **Migrations rétrocompatibles** — Les données des exercices passés doivent rester lisibles après mise à jour du schéma
- **Justificatifs sur filesystem** — Les pièces justificatives sont stockées dans un volume Docker dédié (pas en BDD). La stratégie de backup doit couvrir le dump MariaDB et le volume de fichiers. Sur NAS, les deux sont inclus dans l'outil de backup natif. Sur VPS, un script de backup documenté couvre les deux.

### Intégrations

- Formats bancaires suisses via fichiers (CAMT.053, pain.001) — pas d'API directe avec les banques
- QR-IBAN pour la facturation QR Bill
- Plans comptables suisses standards (PME, association, indépendant)
- Fichiers de test officiels SIX pour la validation en CI

## Innovation & Patterns Novateurs

### Axes d'innovation identifiés

**Innovation de positionnement :** Kesh est la première solution open source, gratuite et auto-hébergée pour la comptabilité suisse. Cette combinaison n'existe pas sur le marché actuel. L'innovation n'est pas technologique mais dans le modèle de distribution et le positionnement.

**Innovation de distribution :** Docker-compose rend l'auto-hébergement accessible à des non-développeurs. Un NAS Synology ou un VPS simple suffisent. Le « time to value » de 15 minutes est un différenciateur fort.

**Innovation IA (post-MVP) :** Intégration optionnelle d'intelligence artificielle pour assister la gestion comptable :
- Classification automatique des écritures (ex. : facture assurance → compte assurances)
- Catégorisation intelligente des transactions importées
- Détection d'anomalies comptables
- Aide à la saisie par suggestion contextuelle

**Principes IA :**
- Fonctionnalité strictement optionnelle — Kesh fonctionne parfaitement sans IA
- IA auto-hébergée (modèle local) : aucune restriction, données restent locales
- IA externe (Claude, OpenAI, etc.) : avertissement explicite que des données comptables seront transmises à un tiers, consentement utilisateur requis

### Approche de validation

- L'innovation de positionnement se valide par l'adoption : des utilisateurs choisissent Kesh plutôt qu'une solution payante
- L'innovation IA se validera en post-MVP par la précision de la classification automatique comparée aux règles d'affectation manuelles

### Atténuation des risques

| Risque | Mitigation |
|---|---|
| L'auto-hébergement freine l'adoption | Docker-compose simplifie au maximum, documentation claire |
| L'IA externe compromet la confidentialité | Avertissement explicite, consentement requis, IA locale privilégiée |
| L'IA locale manque de performance | Fonctionnalité optionnelle, les règles d'affectation manuelles restent disponibles |

## Exigences spécifiques Web App

### Vue d'ensemble

Kesh est une Single Page Application (SPA) Svelte communiquant avec une API REST Axum. L'application est conçue pour un usage desktop/laptop exclusivement — pas de responsive mobile.

### Matrice des navigateurs

| Navigateur | Support |
|---|---|
| Chrome | 2 dernières versions |
| Firefox | 2 dernières versions |
| Safari | 2 dernières versions |
| Edge | 2 dernières versions |
| Mobile | Non supporté |

### Architecture frontend

- **SPA Svelte** — navigation côté client, pas de rendu serveur
- **API REST** — communication JSON avec le backend Axum, préfixe `/api/v1/`
- **Pas de temps réel** — requêtes HTTP classiques, verrouillage optimiste pour la concurrence
- **Pas de SEO** — application derrière authentification, pas d'indexation nécessaire. Site vitrine via GitHub Pages si besoin.

### Design

- **Desktop uniquement** — largeur minimale 1280px
- **Accessibilité** — inspiré WCAG AA (contraste, navigation clavier, labels) sans contrainte stricte
- **Notifications** — banner (succès/warning/erreur), modal (erreur bloquante)
- **Aide contextuelle** — tooltips sur les termes comptables et champs techniques

### Considérations d'implémentation

- Frontend servi en fichiers statiques par nginx (production) ou Axum directement (développement)
- Authentification JWT avec refresh silencieux, expiration 15 min d'inactivité
- Internationalisation côté backend (API expose les traductions), langue configurée au niveau de l'instance
- Formats suisses : dates `dd.mm.yyyy`, montants avec apostrophe `1'234.56`

## Scoping & Développement Phasé

### Stratégie MVP & Philosophie

**Approche MVP :** Résolution de problème — le minimum pour qu'un indépendant ou une association puisse gérer sa comptabilité suisse de bout en bout.

**Ressources :** Développeur solo avec Claude Code. Le phasage en v0.1/v0.2 permet d'avoir un produit utilisable rapidement pour maintenir la motivation.

### v0.1 — MVP (cœur comptable + facturation)

**Parcours utilisateurs supportés :** Marc (indépendant) et Sophie (association) — parcours de base sans TVA ni budgets.

**Fonctionnalités :**
- Configuration système et onboarding assisté (type d'orga, coordonnées, comptes bancaires)
- Compte admin initial (username `admin`, mot de passe via variable d'env)
- Authentification multi-utilisateurs avec RBAC (Admin / Comptable / Consultation)
- Politique de mot de passe configurable, désactivation d'utilisateurs
- Verrouillage optimiste pour la concurrence
- Plan comptable suisse (PME, association, indépendant) chargeable et personnalisable
- Saisie comptable en partie double, journaux (Achats, Ventes, Banque, Caisse, OD)
- Carnet d'adresses unifié (personnes/entreprises, flag client/fournisseur)
- Catalogue de produits/services (nom, description, prix unitaire, TVA)
- Facturation QR Bill 2.2 avec lignes libres ou depuis catalogue, numérotation configurable
- Conditions de paiement configurables
- Multi-comptes bancaires
- Import de relevés bancaires CAMT.053, CSV (parseur tolérant + profils banque)
- Détection de doublons à l'import (hash fichier + vérification par transaction)
- Réconciliation bancaire : flux unique, matching automatique, création manuelle de contrepartie
- Rapports : bilan, compte de résultat, balance, journaux (PDF, CSV)
- Recherche, pagination, tri et filtres
- Notifications UX : banner (succès/warning/erreur) + modal (erreur bloquante)
- Aide contextuelle (tooltips)
- Disclaimer légal « ne remplace pas un fiduciaire »
- Healthcheck endpoint `/health`
- Rate limiting sur le login
- Script de seed rechargeable (démo/test)
- Déploiement docker-compose (Axum + MariaDB, 2 containers, reverse proxy TLS optionnel)
- Langues : FR, DE, IT, EN

### v0.2 — MVP (complétude comptable)

**Parcours utilisateurs complétés :** Tous les parcours (Marc, Sophie, Thomas, Lisa) avec tous les scénarios.

**Fonctionnalités :**
- TVA suisse à taux configurables avec historique, arrondis commercial au centime par ligne, rapport par période
- Budgets annuels par compte individuel avec montants mensuels, versions multiples, rapport comparatif budget vs réalisé
- Avoirs (notes de crédit) avec séquence de numérotation séparée
- Clôture d'exercice avec report des soldes + bilan d'ouverture
- Génération de fichiers de paiement pain.001.001.09.ch.03
- Pièces justificatives attachées aux écritures (filesystem, volume Docker dédié)
- Lettrage
- Export CSV global par table
- Versioning des parseurs/générateurs SIX
- Détection de nouvelle version au démarrage avec avertissement backup avant migration
- 3 manuels PDF embarqués (démarrage, utilisateur, administrateur)

### Phase 3 — Post-MVP

- Dashboard configurable
- Gestion de stocks simplifiée
- Import MT940 (SWIFT legacy)
- Export XLSX
- Réconciliation automatique des paiements batch agrégés
- Envoi de factures par email
- Romanche (RM)
- Reset mot de passe par email
- Export structuré pour migration Kesh-à-Kesh
- Conformité nLPD (export/anonymisation des données personnelles)
- Sous-comptes clients et fournisseurs (grand livre auxiliaire par tiers)
- IA optionnelle : classification automatique des écritures, catégorisation des transactions, détection d'anomalies

### Phase 4 — Vision future

- Évolution vers un mini-ERP pour PME suisses
- Communauté open source active avec contributeurs
- Support payant comme modèle économique
- Intégrations tierces (Open Banking, API partenaires)

### Stratégie d'atténuation des risques

| Type de risque | Risque | Mitigation |
|---|---|---|
| **Technique** | QR Bill pixel-perfect (specs SIX strictes) | Tests unitaires + validation contre specs SIX dès le début |
| **Technique** | Parseur CAMT.053 (XML complexe) | Utiliser les fichiers de test officiels SIX |
| **Technique** | Intégrité comptable (calculs débit/crédit) | Tests unitaires exhaustifs sur le moteur comptable (`rust_decimal`) |
| **Marché** | Faible adoption | Kesh répond d'abord au besoin propre du développeur — adoption externe en bonus |
| **Ressources** | Motivation solo sur la durée | Phasage v0.1/v0.2 pour livrer un produit utilisable rapidement |

## Exigences Fonctionnelles

### Configuration & Onboarding

- FR1 : L'administrateur peut installer Kesh via docker-compose en moins de 15 minutes
- FR2 : L'administrateur peut configurer l'application via des variables d'environnement
- FR3 : L'administrateur peut configurer le nom d'utilisateur et le mot de passe du compte admin initial via des variables d'environnement
- FR4 : L'assistant d'onboarding guide l'utilisateur à travers le type d'organisation, les coordonnées, et la configuration des comptes bancaires
- FR5 : Le système installe automatiquement le plan comptable et les journaux adaptés au type d'organisation choisi
- FR6 : L'administrateur peut configurer la politique de mot de passe
- FR7 : Le système affiche un disclaimer légal « ne remplace pas un fiduciaire »
- FR8 : Le système expose un endpoint de healthcheck (`/health`)

### Gestion des Utilisateurs & Sécurité

- FR9 : L'administrateur peut créer, désactiver et gérer les comptes utilisateurs
- FR10 : L'administrateur peut attribuer un ou plusieurs rôles à un utilisateur (Admin, Comptable, Consultation)
- FR11 : Le système contrôle l'accès aux fonctionnalités en fonction des rôles (RBAC)
- FR12 : L'utilisateur peut s'authentifier avec un identifiant et un mot de passe
- FR13 : Le système renouvelle silencieusement la session et la termine après 15 minutes d'inactivité (configurable)
- FR14 : L'utilisateur peut changer son propre mot de passe
- FR15 : L'administrateur peut réinitialiser le mot de passe d'un utilisateur
- FR16 : Le système bloque les tentatives de connexion après 5 échecs en 15 minutes, avec un délai de déblocage de 30 minutes
- FR17 : Le système utilise le verrouillage optimiste pour gérer la concurrence entre utilisateurs

### Plan Comptable & Écritures

- FR18 : L'utilisateur peut charger un plan comptable suisse standard (PME, association, indépendant)
- FR19 : L'utilisateur peut ajouter, modifier et archiver des comptes dans le plan comptable
- FR20 : L'utilisateur peut saisir des écritures en partie double (débit/crédit)
- FR21 : Le système refuse toute écriture déséquilibrée (débit ≠ crédit)
- FR22 : L'utilisateur peut saisir des écritures dans différents journaux (Achats, Ventes, Banque, Caisse, OD)
- FR23 : L'utilisateur peut supprimer des écritures tant que l'exercice est ouvert
- FR24 : Le système interdit la modification et la suppression des écritures d'un exercice clôturé

### Carnet d'Adresses & Contacts

- FR25 : L'utilisateur peut gérer un carnet d'adresses unifié (personnes et entreprises)
- FR26 : L'utilisateur peut marquer un contact comme client, fournisseur, ou les deux
- FR27 : Le système valide le format et le checksum du numéro IDE (CHE) lorsqu'il est saisi (champ optionnel)
- FR28 : L'utilisateur peut associer des conditions de paiement par défaut à un contact

### Catalogue Produits/Services

- FR29 : L'utilisateur peut gérer un catalogue de produits/services (nom, description, prix unitaire, taux TVA)
- FR30 : L'utilisateur peut sélectionner des articles du catalogue lors de la création d'une facture

### Facturation

- FR31 : L'utilisateur peut créer une facture avec des lignes libres ou depuis le catalogue (quantité, prix, TVA)
- FR32 : L'utilisateur peut supprimer une facture en brouillon
- FR33 : L'utilisateur peut valider une facture, ce qui lui attribue un numéro séquentiel définitif
- FR34 : Le système génère un PDF QR Bill conforme aux spécifications SIX 2.2 pour chaque facture validée
- FR35 : L'utilisateur peut configurer le format de numérotation des factures
- FR36 : L'utilisateur peut annuler une facture validée uniquement par la création d'un avoir (v0.2)
- FR37 : Le système gère une séquence de numérotation séparée pour les avoirs (v0.2)
- FR38 : Le PDF s'ouvre dans un nouvel onglet du navigateur

### Paiements (v0.2)

- FR39 : L'utilisateur peut sélectionner des factures fournisseurs ouvertes pour générer un lot de paiements
- FR40 : Le système génère un fichier pain.001.001.09.ch.03 conforme au schéma XSD SIX
- FR41 : Le système valide le format IBAN avant la génération du fichier de paiement

### Import Bancaire & Réconciliation

- FR42 : L'utilisateur peut importer des relevés bancaires aux formats CAMT.053 et CSV
- FR43 : Le système détecte les fichiers déjà importés et les transactions en doublon
- FR44 : Le système propose automatiquement des contreparties pour les transactions connues
- FR45 : L'utilisateur peut créer manuellement une contrepartie pour les transactions inconnues et réconcilier
- FR46 : Le système propose la création d'une règle d'affectation après une affectation manuelle
- FR47 : L'utilisateur peut gérer les règles d'affectation automatique
- FR48 : L'utilisateur peut éclater manuellement une transaction agrégée en sous-lignes
- FR49 : Le système extrait les sous-transactions (`TxDtls`) des fichiers CAMT quand elles sont présentes
- FR50 : L'utilisateur peut importer un relevé lié à un compte bancaire spécifique (multi-comptes)
- FR51 : Le système gère le rejet partiel d'un import avec un listing détaillé des erreurs
- FR52 : Le parseur CSV supporte UTF-8 et ISO-8859-1 avec détection automatique
- FR53 : L'utilisateur peut configurer des profils de format par banque pour l'import CSV

### TVA (v0.2)

- FR54 : L'administrateur peut configurer les taux de TVA avec des dates de validité
- FR55 : Le système applique les arrondis TVA au centime par ligne (arrondi commercial)
- FR56 : L'utilisateur peut générer un rapport TVA par période (trimestriel/semestriel)

### Budgets (v0.2)

- FR57 : L'utilisateur peut créer un budget annuel par compte individuel avec des montants mensuels
- FR58 : L'utilisateur peut gérer plusieurs versions d'un budget par exercice (initial, révisé)
- FR59 : L'utilisateur peut générer un rapport comparatif budget vs réalisé avec écarts

### Clôture & Exercice (v0.2)

- FR60 : L'utilisateur peut clôturer un exercice comptable
- FR61 : Le système reporte automatiquement les soldes vers le nouvel exercice
- FR62 : L'utilisateur peut saisir un bilan d'ouverture (soldes de départ)

### Pièces Justificatives (v0.2)

- FR63 : L'utilisateur peut attacher des fichiers (PDF, images) à une écriture
- FR64 : Le système stocke les justificatifs dans un volume de stockage dédié, séparé de la base de données

### Rapports & Exports

- FR65 : L'utilisateur peut générer un bilan, un compte de résultat, une balance des comptes et des journaux
- FR66 : L'utilisateur peut exporter les rapports en PDF et CSV
- FR67 : Les PDF générés respectent les formats suisses (apostrophe séparateur de milliers, dates dd.mm.yyyy)
- FR68 : L'utilisateur peut exporter l'ensemble des données par table en CSV
- FR69 : L'utilisateur peut rechercher des écritures par montant, libellé, numéro de facture ou date
- FR70 : L'utilisateur peut trier, filtrer et paginer toutes les listes

### Interface & Aide

- FR71 : Le système affiche des notifications banner pour les succès, avertissements et erreurs non-bloquantes
- FR72 : Le système affiche des modals pour les erreurs bloquantes (session expirée, conflit de version)
- FR73 : Le système fournit des tooltips contextuels sur les termes comptables et champs techniques
- FR74 : L'utilisateur peut accéder à 3 manuels PDF embarqués (démarrage, utilisateur, administrateur) (v0.2)

### Internationalisation

- FR75 : Le système est disponible en français, allemand, italien et anglais
- FR76 : La langue de l'interface est configurée au niveau de l'instance. Elle est découplée de la langue des données comptables (libellés, noms de comptes). L'utilisateur peut changer la langue de l'interface sans impacter les données saisies.

### Déploiement & Maintenance

- FR77 : Le système se déploie via docker-compose avec 2 containers (application + base de données). Un reverse proxy TLS est optionnel et géré par l'infrastructure de l'utilisateur.
- FR78 : Le système détecte une nouvelle version au démarrage et avertit de faire un backup avant migration
- FR79 : Le système applique les migrations de schéma automatiquement
- FR80 : Le système fournit un script de seed rechargeable pour la démo et les tests

### Modèles de Documents (v0.2)

- FR81 : L'utilisateur peut personnaliser les modèles de documents générés (factures, rapports) : logo, coordonnées, textes de pied de page

### Page d'Accueil

- FR82 : Le système affiche une page d'accueil après connexion avec accès rapide aux fonctions principales (dernières écritures, factures ouvertes, soldes des comptes bancaires)

### Comptes Bancaires

- FR83 : L'utilisateur peut configurer et gérer ses comptes bancaires (nom, IBAN, banque)
- FR84 : L'assistant d'onboarding propose la configuration des comptes bancaires avec validation QR-IBAN/IBAN

### Lettrage (v0.2)

- FR85 : L'utilisateur peut lettrer des écritures entre elles pour marquer les correspondances (facture ↔ paiement)
- FR86 : L'utilisateur peut délettrer des écritures précédemment lettrées tant que l'exercice est ouvert

### Versioning des parseurs (v0.2)

- FR87 : Le système identifie la version du format utilisé lors de l'import (CAMT.053, pain.001) et sélectionne le parseur/générateur correspondant

### Traçabilité & Audit

- FR88 : Le système enregistre un journal d'audit des actions utilisateurs sur les données comptables (création, modification, suppression d'écritures, clôture d'exercice) avec l'identifiant utilisateur et l'horodatage

### Résilience Frontend

- FR89 : Le frontend s'affiche même si la base de données est inaccessible, avec un message d'erreur explicite invitant à vérifier l'état du serveur

## Exigences Non-Fonctionnelles

### Performance

- Les pages se chargent en moins de 300ms
- L'import d'un relevé mensuel (200 transactions) s'exécute en moins de 2s
- La génération d'un PDF (facture QR Bill, rapport) s'exécute en moins de 3s
- Le système supporte 2-5 utilisateurs simultanés par instance sans dégradation

### Sécurité

- Tous les mots de passe sont hashés avec argon2 ou bcrypt, jamais stockés en clair
- Aucune donnée n'est accessible sans authentification JWT valide
- La communication est chiffrée via TLS en production (nginx reverse proxy)
- Le rate limiting protège l'endpoint de connexion contre le brute-force
- Les données comptables ne sont jamais transmises à un tiers (sauf IA externe avec consentement explicite, post-MVP)
- Les credentials de base de données ne sont jamais exposés dans les logs ou l'API

### Fiabilité & Intégrité

- La balance débit/crédit est garantie correcte à tout moment — aucun écart possible
- Aucune donnée comptable n'est perdue en cas de crash ou de redémarrage
- Les fichiers pain.001 générés passent la validation XSD officielle SIX
- Les QR Bill générés sont conformes aux spécifications SIX 2.2 (dimensions, position, contenu)
- Les migrations de schéma préservent l'intégrité des données des exercices passés
- L'arithmétique monétaire utilise exclusivement des types décimaux exacts (`rust_decimal`), jamais de flottants

### Accessibilité

- L'interface s'inspire de WCAG AA (contraste suffisant, navigation clavier, labels sur les formulaires) sans contrainte stricte de conformité
- Le zoom navigateur reste fonctionnel à 200%

### Internationalisation

- Toutes les chaînes de l'interface sont externalisées (format Fluent `.ftl`)
- Les formats régionaux suisses sont respectés : dates `dd.mm.yyyy`, montants avec apostrophe `1'234.56`
- Les PDF générés respectent les mêmes formats régionaux

### Maintenabilité

- Le code source est documenté selon les meilleures pratiques (doc comments Rust, JSDoc Svelte)
- Aucun code dupliqué (principe DRY)
- Tests unitaires sur toute la logique métier (moteur comptable, TVA, calculs financiers)
- Tests d'intégration sur les parseurs (CAMT.053, QR Bill, pain.001) avec les fichiers de test officiels SIX
- Tests E2E avec Playwright couvrant chaque parcours utilisateur
- L'API est versionnée (`/api/v1/`) dès le premier jour

### Déploiement

- L'application se déploie via une seule commande `docker-compose up`
- L'image Docker résultante pèse moins de 100 Mo
- Les logs applicatifs sont émis sur stdout/stderr (standard Docker)
- Le healthcheck `/health` vérifie la connexion à la base de données
