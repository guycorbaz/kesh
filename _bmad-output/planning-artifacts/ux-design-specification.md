---
stepsCompleted:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
  - 7
  - 8-in-progress
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - docs/change_request.md
  - docs/kesh-prd-v0.2.md
documentCounts:
  prd: 2
  brief: 0
  other: 1
---

# UX Design Specification Kesh

**Author:** Guy
**Date:** 2026-04-02

---

## Executive Summary

### Vision Projet

Kesh est la première solution de comptabilité suisse open source, gratuite et auto-hébergée. Face aux solutions cloud qui retiennent les données en otage (exportation difficile, dépendance à l'abonnement), aux logiciels desktop vieillissants (Crésus, Banana) et aux outils open source non adaptés au contexte suisse (GnuCash), Kesh offre une alternative moderne, souveraine et accessible.

L'application web (SPA Svelte + API Axum + MariaDB) se déploie via docker-compose sur l'infrastructure de l'utilisateur — NAS, VPS ou serveur local. Les données comptables restent intégralement sous contrôle de l'utilisateur.

### Utilisateurs Cibles

| Persona | Profil | Compétence comptable | Besoin UX prioritaire |
|---|---|---|---|
| **Marc** (indépendant) | Graphiste, Lausanne, 5-10 factures/mois | Basique | Efficacité : facturer et réconcilier vite |
| **Sophie** (association) | Trésorière bénévole, aucune formation | Aucune | Guidage : comprendre sans se perdre |
| **Thomas** (admin) | Informaticien, gère 2 instances | Technique, pas comptable | Contrôle : déployer, configurer, maintenir |
| **Lisa** (fiduciaire) | Comptable pro, 8 clients | Experte | Puissance : raccourcis, batch, efficacité maximale |

### Principes UX Fondateurs

1. **Simple par défaut, puissant sur demande** — Progressive disclosure. Mode Guidé pour les débutants, mode Expert pour les professionnels. Pré-configurations par type d'organisation qui masquent 80% de la complexité pour 80% des utilisateurs, tout en restant entièrement personnalisables.

2. **L'UX commence avant l'application** — Le `.env` documenté, le guide d'installation PDF, le port configurable, l'URL affichée au démarrage font partie de l'expérience. L'installation est un touchpoint UX.

3. **Chaque erreur est un moment d'apprentissage** — Chaque message d'erreur dit ce qui s'est passé ET ce que l'utilisateur peut faire. Jamais de message cryptique. « Écriture déséquilibrée — le total des débits (500.00) ne correspond pas au total des crédits (0.00) » plutôt que « Erreur 422 ».

4. **Respect du temps et de l'autonomie** — 15 minutes pour être opérationnel. Données sous contrôle. Export facile. Onboarding interruptible et reprenant là où on en était. Pas d'otage, y compris dans l'application elle-même.

### Défis UX Clés

1. **Deux profils extrêmes** — L'interface doit être immédiatement compréhensible pour un non-comptable (Sophie) sans ralentir un expert (Lisa). Mode Guidé / Expert comme réponse structurelle.

2. **Démystifier la comptabilité** — Aide contextuelle en langage naturel (« l'argent entre dans ce compte ») couplée à la terminologie comptable correcte. Sophie apprend, Lisa n'est pas ralentie.

3. **Flux multi-étapes complexes** — Import bancaire, réconciliation, facturation QR Bill. Mode guidé pas-à-pas pour les débutants, écran compact avec actions en lot pour les experts. Même fonctionnalité, deux présentations.

4. **Confiance dès le premier contact** — Transparence opérationnelle : logs clairs, `.env` documenté, URL affichée, backup responsabilisé, guide d'installation PDF. Le contraire du cloud opaque.

### Opportunités UX

1. **Onboarding comme différenciateur** — Deux chemins : exploration avec données de démo ou production directe. 6 étapes atomiques, chacune persistée immédiatement, interruptible et repris via bannière contextuelle. Opérationnel en moins de 15 minutes.

2. **Mode Guidé / Expert** — Choix à l'onboarding (« Comment préférez-vous utiliser Kesh ? Guidé / Expert »), modifiable à tout moment dans le profil. Impact global sur la présentation des flux, les confirmations, les raccourcis et le niveau de détail.

3. **Séparation langue comptable / langue interface** — La langue comptable (plan comptable, terminologie) est fixée à l'onboarding au niveau de l'instance. La langue d'interface est modifiable par utilisateur. Lisa peut travailler sur un client zurichois (compta en DE) avec son interface en FR.

4. **Souveraineté comme valeur UX** — Export facile, données sur l'infrastructure de l'utilisateur, backup intégré aux outils NAS existants, aucune télémétrie, documentation honnête sur les responsabilités de sauvegarde.

### Expérience d'Installation

**Fichiers livrés :** `docker-compose.yml`, `.env`, `guide-installation.pdf`

**`.env` documenté :**
- Port configurable (`KESH_PORT`) — essentiel pour les NAS avec plusieurs containers
- Mot de passe admin initial — changement forcé au premier login
- Nom du projet (`COMPOSE_PROJECT_NAME`) — évite les collisions de noms
- Variables purement techniques — la configuration métier se fait dans l'application

**Guide d'installation PDF (4-5 pages) :**
- Prérequis, installation en 5 étapes, configuration du `.env`
- Vérification (URL, healthcheck, premier login)
- Problèmes fréquents (port occupé, container qui ne démarre pas)
- Section NAS (Synology, QNAP)
- Backup : responsabilisation claire, pas de fausse promesse

**Feedback au démarrage :**
- Logs Docker : `✅ Kesh est prêt → http://localhost:{KESH_PORT}`
- Si services pas prêts : page d'attente élégante, pas d'erreur nginx

### Flux d'Onboarding

**Chemin A — Exploration :**
1. Langue d'interface (FR / DE / IT / EN — noms dans leur langue, sans texte explicatif)
2. Mode d'utilisation (Guidé / Expert)
3. Charger les données de démo → exploration immédiate
   - Bannière permanente : 🟡 « Instance de démonstration — données fictives »
   - Réinitialisation complète pour passer en production

**Chemin B — Production :**
1. Langue d'interface
2. Mode d'utilisation (Guidé / Expert)
3. Type d'organisation (Indépendant / Association / PME)
4. Langue comptable (FR / DE / IT / EN)
5. Coordonnées (nom/raison sociale, adresse, IDE optionnel)
6. Compte bancaire principal (banque, IBAN, QR-IBAN) — bouton « Configurer plus tard » disponible

**Principes d'onboarding :**
- Chaque étape est atomique et persistée immédiatement en base
- L'onboarding peut être interrompu et repris à tout moment
- Bannière contextuelle : 🔵 « Configuration incomplète — Terminer la configuration »
- Kesh fonctionne même avec un onboarding partiel — les fonctionnalités dépendantes des étapes manquantes affichent un message avec lien vers la configuration
- L'onboarding est un raccourci vers les paramètres — tout est retrouvable et modifiable dans Administration

### Correspondance Onboarding / Administration

| Onboarding | Administration |
|---|---|
| Langue d'interface | Profil → Langue |
| Mode Guidé/Expert | Profil → Mode d'utilisation |
| Type d'organisation | Paramètres → Organisation |
| Langue comptable | Paramètres → Comptabilité |
| Coordonnées | Paramètres → Organisation |
| Compte bancaire | Paramètres → Comptes bancaires |

### Bannières Contextuelles

| Bannière | Condition | Disparition |
|---|---|---|
| 🔵 Onboarding incomplet | Étapes non terminées | Toutes étapes complétées |
| 🟡 Mode démo | Données fictives chargées | Après réinitialisation |

### États Vides

Chaque liste vide affiche un message d'appel à l'action contextuel :
- En mode Guidé : explication + suggestion d'ordre + bouton
- En mode Expert : bouton d'action uniquement

### Navigation

Navigation par activité (vocabulaire utilisateur, pas technique) :
- « Facturer » plutôt que « Module Facturation »
- « Importer un relevé » plutôt que « Import bancaire CAMT.053 »
- « Voir mes comptes » plutôt que « Balance des comptes »

### Raccourcis Clavier (v0.1)

- `Ctrl+N` — nouvelle écriture
- `Ctrl+S` — sauvegarder
- `Tab` / `Shift+Tab` — navigation formulaires
- Navigation clavier complète dans les tableaux

### Impacts PRD Identifiés

- **FR76** — La langue n'est plus configurée uniquement au niveau de l'instance. Langue comptable = instance (onboarding), langue interface = utilisateur (profil).
- **FR80** — Le script de seed devient aussi un outil d'onboarding (chemin « Exploration »).
- **FR82** — Page d'accueil « minimaliste » précisée : minimaliste ET contextuelle, avec checklist de démarrage pour les nouveaux utilisateurs.

## Core User Experience

### Expérience Définissante

L'expérience définissante de Kesh est le **parcours de première adoption** : installation → configuration → première écriture. C'est un entonnoir où chaque étape est un filtre. Si l'utilisateur bloque à l'une d'elles, il n'atteindra jamais l'usage quotidien (facturation, import, réconciliation).

**Entonnoir d'adoption :**

1. **Installation** (docker-compose) — `.env` documenté, port configurable, URL dans les logs, guide PDF
2. **Configuration du plan comptable** — Pré-chargé selon le type d'organisation. En mode Guidé : « Il est déjà configuré, vous n'avez rien à modifier pour commencer »
3. **Ouverture des comptes** — Choix explicite : « Je pars de zéro » ou « J'ai des soldes à reprendre ». Pas de formulaire imposé si les soldes sont à zéro
4. **Première écriture** — Mode Guidé : assistant pas-à-pas. Mode Expert : formulaire direct avec raccourcis clavier

Chaque étape doit être franchissable sans blocage. L'utilisateur qui arrive à la première écriture validée est converti.

### Stratégie Plateforme

- **Web app uniquement** — SPA Svelte + API REST Axum
- **Desktop/laptop exclusivement** — largeur minimale 1280px, pas de responsive mobile
- **Interaction souris + clavier** — optimisée pour la saisie au clavier (Tab, autocomplétion, raccourcis), souris pour la navigation et les actions ponctuelles
- **Pas de mode offline** — connexion au serveur requise (auto-hébergé ou local)
- **Navigateurs** — Chrome, Firefox, Safari, Edge (2 dernières versions)

### Interactions Sans Effort

**Saisie d'écriture :**
- Tab entre les champs, flux naturel gauche → droite
- Autocomplétion des comptes par numéro ou nom (« 1020 » ou « Banque »)
- Validation instantanée de l'équilibre débit/crédit (feedback visuel en temps réel)
- En mode Expert : saisie rapide sans lever les mains du clavier

**Facturation QR Bill :**
- Sélection client → ajout de lignes (catalogue ou libre) → valider → PDF dans un nouvel onglet
- Minimum de clics entre l'intention et le résultat
- QR Code conforme SIX généré automatiquement

**Import bancaire :**
- Glisser-déposer le fichier (ou sélection classique)
- Prévisualisation avant import
- Détection automatique du format et des doublons
- Règles d'affectation appliquées automatiquement
- Validation en lot des transactions pré-affectées

**Navigation générale :**
- Pages < 300ms — aucun chargement perceptible
- Retour arrière naturel (bouton navigateur fonctionnel)
- Aucun cul-de-sac — toujours un chemin de retour ou une action suivante
- Recherche globale accessible partout

### Moments Critiques de Succès

| Moment | Réaction attendue | Risque si échoué |
|---|---|---|
| `docker-compose up` → URL affichée | « Ça marche ! » | Abandon immédiat |
| Choix de langue + onboarding | « C'est dans ma langue, c'est pour moi » | Doute sur la qualité |
| Plan comptable pré-configuré | « Je n'ai rien à faire, c'est prêt » | Panique face à la complexité |
| Première écriture validée | « J'ai compris le principe » | « Trop compliqué pour moi » |
| Première facture QR Bill PDF | « Je peux envoyer ça à mon client » | Retour à l'ancien outil |
| Premier import réconcilié | « Ça me fait gagner du temps » | « Autant rester sur Excel » |

### Principes d'Expérience

1. **L'adoption se joue dans la première heure** — De l'installation à la première écriture, chaque étape doit être franchissable sans aide externe. Le chemin « Exploration » avec données de démo raccourcit ce parcours.

2. **Ne jamais bloquer, toujours guider** — Aucune étape obligatoire ne doit être un mur. Plan comptable pré-configuré, soldes à zéro par défaut, aide contextuelle à chaque champ. L'utilisateur avance toujours.

3. **Le clavier est roi pour la saisie** — La comptabilité est un métier de saisie. Tab, autocomplétion, raccourcis. La souris est un complément, pas l'outil principal.

4. **Le résultat visible crée la confiance** — Chaque action produit un feedback immédiat : écriture validée → bilan mis à jour, facture validée → PDF visible, import → transactions listées. Pas d'action silencieuse.

## Desired Emotional Response

### Objectifs Émotionnels Primaires

| Persona | Émotion cible | Moment déclencheur |
|---|---|---|
| **Marc** | Efficacité sereine | Comptabilité mensuelle bouclée en 30 min |
| **Sophie** | Fierté et confiance | Bilan présenté à l'AG, correct et professionnel |
| **Thomas** | Contrôle et fiabilité | Mise à jour en 5 min, healthcheck au vert |
| **Lisa** | Fluidité et puissance | 8 clôtures d'exercice sans friction |

**Émotion différenciante :** La souveraineté — « Mes données sont chez moi, je ne dépends de personne, je peux partir quand je veux (mais je n'en ai pas envie). »

### Cartographie du Parcours Émotionnel

| Étape du parcours | Émotion visée | Design qui la soutient |
|---|---|---|
| Découverte (README, site) | Curiosité + crédibilité | « Open source, gratuit, suisse » |
| Installation | Soulagement → satisfaction | `docker-compose up` → URL en 2 min |
| Onboarding | Bienvenue + simplicité | « C'est dans ma langue, c'est pour moi » |
| Exploration démo | Compréhension + projection | « Je vois comment ça marche » |
| Première écriture | Accomplissement | « J'ai compris le principe débit/crédit » |
| Première facture QR Bill | Wow + utilité concrète | PDF pro dans un nouvel onglet |
| Premier import bancaire | Gain de temps | Réconciliation semi-automatique |
| Usage quotidien | Routine fluide | Raccourcis, autocomplétion, < 300ms |
| Erreur utilisateur | Apprentissage (pas honte) | Message clair + action suggérée |
| Clôture d'exercice | Fierté + accomplissement | Bilan/résultat en PDF, chiffres justes |
| Retour après absence | Retrouvailles faciles | Interface familière, données intactes |

### Micro-Émotions Critiques

**À cultiver :**
- **Confiance** → Chaque calcul est juste (rust_decimal). Le bilan s'équilibre toujours. Les PDF sont conformes SIX.
- **Compétence** → L'utilisateur apprend en utilisant. Les tooltips transforment l'ignorance en savoir, pas en honte.
- **Autonomie** → Tout est modifiable, tout est exportable. L'onboarding est un raccourci, pas un passage obligé.
- **Sérénité** → Pas de surprises. Pas de données qui disparaissent. Pas de mise à jour forcée. Pas d'abonnement qui expire.

**À éliminer :**
- **Panique** → Plan comptable pré-configuré, mode Guidé, pas de terminologie sans explication
- **Sentiment d'otage** → Export facile, données locales, open source, aucune dépendance externe
- **Honte** → Messages d'erreur qui disent quoi faire, pas qui blâment. « Écriture déséquilibrée » pas « Erreur de saisie »
- **Abandon** → Entonnoir d'adoption sans mur. Chaque étape skippable ou reportable. Données de démo pour explorer sans risque
- **Méfiance** → Auto-hébergé, zéro télémétrie, documentation honnête sur les backups
- **Frustration** → Performance < 300ms, navigation clavier, aucun cul-de-sac

### Implications Design

| Émotion visée | Choix UX |
|---|---|
| Confiance | Feedback visuel immédiat sur chaque action (balance en temps réel, toast de confirmation) |
| Compétence | Tooltips contextuels bilingues (langage naturel + terme comptable) |
| Autonomie | Tout paramètre modifiable après l'onboarding, export accessible partout |
| Sérénité | Confirmations avant actions destructives, bannières au lieu de modals quand possible |
| Fierté | PDF professionnels, rapports bien formatés, formats suisses corrects |
| Efficacité | Raccourcis clavier, autocomplétion, actions en lot, mode Expert |

### Principes de Design Émotionnel

1. **La confiance se construit par la précision** — Chaque montant affiché est exact. Chaque PDF est conforme. L'utilisateur n'a jamais besoin de vérifier si Kesh a bien calculé.

2. **L'apprentissage remplace la honte** — Quand l'utilisateur se trompe, le système explique et guide. Le vocabulaire des messages est neutre et orienté action, jamais accusateur.

3. **La souveraineté est un sentiment, pas juste une fonctionnalité** — Chaque interaction rappelle subtilement que l'utilisateur est maître : ses données, son serveur, ses exports, son rythme.

4. **Le calme est un luxe** — Pas de notifications intrusives, pas de clignotements, pas d'urgence artificielle. La comptabilité est stressante par nature — Kesh ne doit pas en rajouter.

## UX Pattern Analysis & Inspiration

### Analyse des Produits Inspirants

**Bexio (comptabilité suisse cloud) :**
- Facturation et paiements fluides — le « core loop » quotidien fonctionne bien
- Flux de paiement en deux temps : créer au fil de l'eau, envoyer en lot le jour de paiement — pattern naturel et efficace
- Scan de factures fournisseurs envoyé à Bexio pour traitement — bon concept, mais données chez un tiers
- Configuration dispersée dans de multiples menus — difficile de trouver où paramétrer
- Messages d'erreur cryptiques (configuration comptes bancaires)
- Saisie d'écritures plus complexe que la facturation
- Export impossible en une fois — compte par compte, sans pièces jointes. Captivité délibérée des données.
- Licence coûteuse pour un usage partiel

**Odoo (ERP modulaire) :**
- Onboarding guidé efficace — bonne première impression
- Complexité exponentielle avec les modules
- Leçon : la simplicité doit tenir dans le temps, pas juste le premier jour

**Todoist (gestion de tâches) :**
- Interface épurée, usage quotidien sans fatigue
- Chaque action est rapide et prévisible
- Stabilité de l'interface entre versions — confort de la familiarité
- Modèle de « routine fluide » à transposer au contexte comptable

**Paperless-ngx (gestion documentaire) :**
- Apprentissage local par classificateur bayésien — le système observe les classements manuels et apprend à suggérer
- OCR via Tesseract, 100% local, aucune donnée externe
- Plus on l'utilise, meilleur il devient — rétention par la valeur, pas la captivité
- Modèle directement transposable aux règles d'affectation comptable

### Patterns UX Transférables

**Pattern unifié « Sélectionner → Vérifier → Valider → Résultat » :**

Les trois flux principaux partagent la même structure :

| Flux | Sélectionner | Vérifier | Valider | Résultat |
|---|---|---|---|---|
| Facturer | Client + lignes | Récapitulatif | Confirmer | PDF QR Bill |
| Payer | Factures en attente | Montants + IBAN | Confirmer | pain.001 |
| Importer | Fichier bancaire | Prévisualisation | Confirmer | Écritures |

L'utilisateur apprend ce pattern une fois et le retrouve partout.

**Flux de paiement en deux temps (inspiré Bexio) :**

- Temps 1 (au fil de l'eau) : facture reçue → créer un paiement → ajouté à la liste d'attente
- Temps 2 (jour de paiement) : sélectionner → vérifier → générer pain.001 → upload e-banking
- Sépare la décision individuelle de l'action groupée

**Dossier inbox pour factures fournisseurs :**

- Volume Docker monté (`./inbox/`) — Marc dépose ses scans/PDF
- Kesh détecte les nouveaux fichiers → liste « Factures à traiter »
- Split view : aperçu du scan à gauche, formulaire à droite
- Extraction automatique par priorité :
  1. QR Code détecté → extraction complète (v0.1)
  2. Tesseract OCR local → extraction texte (post-MVP)
  3. Classificateur bayésien → suggestion de compte (post-MVP)
  4. Aucune extraction → saisie manuelle
- Indicateur de source : 🟢 QR Code / 🟡 IA locale / ⚪ Manuel

**Apprentissage local à la Paperless-ngx :**

- Le système observe les affectations manuelles de l'utilisateur
- Après N affectations similaires, il commence à suggérer
- Confiance croissante affichée à l'utilisateur
- 100% local — classificateur bayésien, pas de LLM, pas de cloud
- Le modèle appris fait partie des données utilisateur (exportable)

**Hiérarchie de navigation par fréquence d'usage :**

- Quotidien/hebdomadaire (accès direct) : Facturer, Paiements, Import relevé
- Mensuel (accès secondaire) : Réconciliation, Écritures manuelles, Rapports
- Rare (paramètres) : Plan comptable, Configuration, Export

**Configuration centralisée (Anti-Bexio) :**

- Un seul menu « Paramètres » avec sous-sections claires
- Pas de chasse au trésor dans N menus dispersés

**Export global en un clic (Anti-Bexio) :**

- Bouton visible dans le menu principal, pas caché dans les paramètres
- ZIP complet : CSV par table + pièces justificatives + métadonnées
- Message de souveraineté : la sortie est toujours accessible

### Anti-Patterns à Éviter

| Anti-pattern | Observé dans | Pourquoi l'éviter |
|---|---|---|
| Configuration dispersée | Bexio | L'utilisateur ne trouve pas où paramétrer |
| Messages d'erreur cryptiques | Bexio | L'utilisateur ne sait pas quoi corriger |
| Export verrouillé / par compte | Bexio | Captivité délibérée des données |
| Complexité exponentielle | Odoo | L'utilisateur perd le contrôle |
| Interface instable | Apps diverses | Perte de repères, méfiance |
| IA externe sans consentement | Apps diverses | Violation de la souveraineté |

### Stratégie d'Inspiration Design

**À adopter :**
- Flux facturation/paiement Bexio — benchmark suisse connu
- Flux en deux temps pour les paiements — naturel et efficace
- Épure et stabilité Todoist — routine quotidienne sans fatigue
- Apprentissage local Paperless-ngx — intelligence sans compromis

**À adapter :**
- Configuration Bexio → centraliser au lieu de disperser
- Onboarding Odoo → garder le guidage, contenir la complexité
- Inbox Paperless-ngx → transposer au contexte factures fournisseurs

**À éviter :**
- Export verrouillé (Bexio) → export global accessible
- Messages cryptiques (Bexio) → messages humains avec action
- Modules explosifs (Odoo) → produit unique, complexité stable
- IA opaque → transparence sur la source des suggestions

### Architecture Intelligence Locale

| Couche | Technologie | Données | Phase |
|---|---|---|---|
| Extraction QR Code | Lib Rust locale | Aucune sortie | v0.1 |
| OCR documents | Tesseract local | Aucune sortie | Post-MVP |
| Classification | Bayésien local | Aucune sortie | Post-MVP |
| Conseil comptable | LLM externe | Consentement requis | Post-MVP |

## Design System Foundation

### Choix du Design System

**shadcn-svelte + Tailwind CSS**

Système de composants copiables (pas de dépendance externe) basé sur Bits UI (primitives accessibles pour Svelte) et stylé avec Tailwind CSS.

**Stack technique :**
- shadcn-svelte (composants stylés) → Bits UI (composants accessibles) → Melt UI (primitives headless)
- Tailwind CSS pour le styling utilitaire
- Composants copiés dans le projet — aucune dépendance runtime

### Justification du Choix

| Critère | Évaluation |
|---|---|
| **Svelte-natif** | Composants conçus pour Svelte, pas un portage React |
| **Contrôle total** | Composants copiés dans le projet, modifiables librement |
| **Accessibilité** | Bits UI intègre WCAG AA (navigation clavier, ARIA, contraste) |
| **Productivité** | Tailwind CSS + Claude Code = itération rapide |
| **Licence** | MIT — compatible EUPL 1.2 |
| **Pas de lock-in** | Aucune dépendance runtime, le code est dans le projet |
| **Composants métier** | Tables, formulaires, modals, toasts, dropdowns, date pickers |
| **Pérennité** | Même si le projet upstream ralentit, Kesh n'est pas impacté |

### Approche d'Implémentation

**Phase 1 — Fondations (début du développement) :**
- Installation Tailwind CSS + configuration des design tokens
- Import des composants shadcn-svelte de base : Button, Input, Select, Table, Dialog, Toast, Tooltip, DropdownMenu
- Définition du thème Kesh

**Phase 2 — Composants métier :**
- Composants spécifiques Kesh construits sur les primitives shadcn : formulaire de saisie d'écriture, split view factures, tableau de réconciliation
- Composants de navigation : menu principal, breadcrumb, recherche globale

**Phase 3 — Raffinement :**
- Variantes mode Guidé / Expert sur tous les composants
- Thème finalisé, cohérence visuelle vérifiée sur tous les écrans

### Stratégie de Personnalisation

**Design tokens Kesh :**

- **Couleurs — langage visuel fonctionnel :**
  - Vert = positif (revenu, actif, facture payée, import réussi)
  - Rouge = négatif (dépense, erreur, facture en retard)
  - Bleu = neutre/informatif (navigation, liens, sélection)
  - Jaune/Orange = attention (avertissement, facture en attente, démo)
  - Gris = désactivé/archivé
  - Ce code couleur correspond aux conventions comptables connues — Lisa le lit sans réfléchir

- **Typographie :**
  - Police système (Inter ou similaire), lisibilité maximale
  - `font-variant-numeric: tabular-nums` obligatoire pour tous les montants — chiffres à largeur fixe pour alignement parfait des colonnes comptables
  - Police tabulaire pour les montants (chiffres alignés au centime)

- **Espacements — adaptatifs selon le mode :**
  - Mode Guidé : espacements généreux (`gap-4`), plus d'air, moins d'information par écran, boutons plus grands avec labels explicites
  - Mode Expert : espacements compacts (`gap-2`), plus d'information par écran, boutons avec icônes et labels en tooltip

- **Tableaux :**
  - Composant central de Kesh. Lignes alternées, tri par colonne, pagination, sélection en lot
  - Montants alignés à droite avec `tabular-nums`
  - Optimisé pour la lisibilité des données comptables

**Composants sur mesure à créer :**
- Formulaire de saisie d'écriture (débit/crédit avec autocomplétion)
- Split view factures fournisseurs (aperçu PDF + formulaire)
- Tableau de réconciliation bancaire
- Bannières contextuelles (onboarding, démo, erreurs)
- Indicateur d'équilibre débit/crédit en temps réel
- Générateur de PDF QR Bill (prévisualisation)

### Stratégie de Test du Design System

**Tests unitaires des composants (Vitest + Testing Library) :**
- Chaque composant shadcn personnalisé a ses tests unitaires
- Rendu correct avec différentes props et états
- Comportement mode Guidé vs mode Expert
- Gestion des cas limites (valeurs vides, texte long, montants négatifs, caractères spéciaux UTF-8)
- Validation des formats suisses (montants avec apostrophe, dates dd.mm.yyyy)

**Tests d'accessibilité :**
- Navigation clavier complète sur chaque composant interactif (Tab, Entrée, Échap, flèches)
- Attributs ARIA corrects (labels, rôles, états)
- Contraste WCAG AA vérifié sur tous les thèmes
- Axe-core intégré dans la CI pour détecter les régressions

**Tests de sécurité :**
- XSS : champs de saisie échappent le HTML (fournisseur nommé `<script>` ne s'exécute jamais)
- CSRF : token valide sur chaque formulaire de modification
- Entrées malveillantes testées sur tous les champs texte

**Tests d'intégration :**
- Formulaire de saisie d'écriture : autocomplétion, validation débit/crédit, soumission
- Split view factures : chargement PDF, pré-remplissage, validation
- Tableau de réconciliation : sélection en lot, filtres, tri, pagination
- Bannières contextuelles : affichage conditionnel, disparition
- Verrouillage optimiste : deux sessions modifient la même écriture → modal de conflit

**Tests E2E (Playwright) :**
- Structure par parcours utilisateur :
  - `marc-independent/` : onboarding, facturation, import, paiements
  - `sophie-association/` : onboarding, première écriture, réconciliation
  - `thomas-admin/` : déploiement, gestion utilisateurs
  - `lisa-fiduciary/` : opérations en lot, clôture d'exercice
- Chaque flux testé dans les deux modes (Guidé / Expert)
- Onboarding : chemin A (démo) et chemin B (production)
- Onboarding interrompu et repris
- États vides avec appels à l'action
- Navigation clavier sur les flux critiques
- Raccourcis clavier fonctionnels
- Multilingue : 4 langues sur les écrans principaux
- Import bancaire : fichiers SIX officiels, CSV multi-encodages, doublons, rejets partiels

**Tests de performance :**
- Chaque page < 300ms (Lighthouse CI)
- Tableaux avec 1000+ lignes : rendu fluide
- Génération PDF < 3s

**Tests visuels (post-MVP) :**
- Snapshots visuels des composants clés (régressions CSS)
- Vérification du rendu PDF par comparaison visuelle

## Expérience Utilisateur Définissante

### L'Interaction Définissante

**« Ça fonctionne et c'est gratuit. »**

L'expérience définissante de Kesh n'est pas une interaction spécifique — c'est la découverte que tout fonctionne, dès le premier contact, sans contrepartie financière. L'utilisateur installe, explore, et constate : facturation QR Bill, import bancaire, écritures, bilan — tout est là, tout marche, et c'est gratuit.

La démonstration qui convainc : le chemin Exploration de l'onboarding. L'utilisateur charge les données de démo et voit un Kesh fonctionnel — factures, écritures, bilan, PDF. Il peut toucher, explorer, comprendre. La conversion se fait à ce moment : « Si la démo marche aussi bien, le vrai marchera aussi. »

### Modèle Mental de l'Utilisateur

**D'où il vient :**
- Bexio / logiciel cloud → « Je paie un abonnement pour 10% des fonctionnalités, et je ne peux pas récupérer mes données facilement »
- Crésus / Banana → « Mon logiciel est vieillissant, l'interface date, les mises à jour sont payantes »
- Excel → « Je bricole, c'est source d'erreurs, je n'ai pas de bilan propre »
- GnuCash → « C'est gratuit mais pas adapté à la Suisse, pas de QR Bill, pas de CAMT.053 »

**Ce qu'il attend :**
- Que ça fonctionne comme ce qu'il connaît (Bexio), en mieux
- Que ce soit gratuit — pas de « gratuit mais limité » ou « gratuit pendant 30 jours »
- Que ses données soient chez lui, pas chez un tiers
- Qu'il n'ait pas besoin d'un diplôme comptable pour s'en servir

**Ce qu'il ne cherche PAS :**
- De l'innovation technologique
- Des fonctionnalités qu'il n'a jamais vues
- Un ERP complet

### Critères de Succès de l'Expérience Core

| Critère | Indicateur | Mesure |
|---|---|---|
| « Ça marche » | Chemin Exploration → Kesh complet en 3 min | Temps entre docker-compose up et exploration de la démo |
| « C'est pour la Suisse » | QR Bill conforme, CAMT.053, plan comptable suisse | PDF QR Bill scannable par une banque |
| « C'est dans ma langue » | Interface et plan comptable dans la langue choisie | 4 langues fonctionnelles sans erreur |
| « Je m'y retrouve » | Navigation intuitive, vocabulaire compréhensible | Sophie peut créer une écriture sans aide externe |
| « C'est rapide » | Chaque action produit un résultat visible immédiatement | Pages < 300ms, PDF < 3s |
| « C'est gratuit, vraiment » | Aucune limitation, aucun paywall, aucune surprise | 100% des fonctionnalités accessibles sans paiement |

### Patterns UX : Établis vs Novateurs

**Patterns établis (90% de l'interface) :**
- Saisie comptable débit/crédit — pattern universel de la comptabilité
- Flux facturation — identique à Bexio (benchmark suisse)
- Tableaux avec tri/filtre/pagination — pattern standard des apps métier
- Formulaires CRUD classiques — créer, lire, modifier, archiver
- Navigation par menu latéral ou supérieur — pattern SPA standard
- Notifications banner/modal — pattern UX standard

**Patterns adaptés (innovation dans le familier) :**
- Flux de paiement en deux temps — inspiré Bexio, optimisé
- Inbox factures fournisseurs avec split view — inspiré Paperless-ngx
- Mode Guidé / Expert — pas nouveau conceptuellement, mais rare dans la comptabilité
- Onboarding avec chemin démo — peu courant dans les logiciels comptables, différenciateur fort

**Patterns novateurs (éducation nécessaire) :**
- Apprentissage local des affectations (post-MVP) — concept nouveau pour les utilisateurs comptables. Nécessite une explication claire : « Kesh apprend vos habitudes pour vous faire gagner du temps »
- Extraction QR Code des factures reçues — le geste est nouveau (déposer un PDF, Kesh lit le QR), mais le résultat est familier (formulaire pré-rempli)

### Mécanique de l'Expérience Définissante

**1. Initiation — « J'installe et j'explore »**
- `docker-compose up` → URL dans les logs
- Premier écran : choix de langue (4 drapeaux)
- Chemin A (Exploration) : 3 clics → Kesh avec données de démo

**2. Interaction — « Je touche et je comprends »**
- L'utilisateur explore : ouvre une facture, voit le PDF QR Bill
- Regarde le plan comptable → « c'est déjà configuré »
- Ouvre le bilan → « les chiffres sont là »
- Essaie de créer une écriture → « ça marche comme je m'y attends »

**3. Feedback — « Le système me répond »**
- Chaque action produit un résultat visible (toast, mise à jour, PDF dans un nouvel onglet)
- Les erreurs expliquent et guident
- La balance débit/crédit se met à jour en temps réel

**4. Conversion — « J'adopte »**
- L'utilisateur réinitialise → chemin B (Production)
- L'onboarding en 6 étapes → son Kesh, sa langue, son plan comptable
- Première vraie écriture → « c'est parti »
- Le message implicite : « Si c'était aussi facile, je reste »

## Visual Design Foundation

### Système de Couleurs

**Palette — sobre, fonctionnelle, professionnelle :**

Basée sur Tailwind Slate/Blue — éprouvée, accessible, cohérente avec le langage visuel fonctionnel défini dans le design system.

| Rôle | Couleur | Hex | Usage |
|---|---|---|---|
| **Primaire** | Bleu ardoise | `#1e40af` | Navigation, boutons principaux, liens |
| **Primaire clair** | Bleu ciel | `#3b82f6` | Survol, sélection, focus |
| **Succès** | Vert | `#16a34a` | Facture payée, import réussi, balance OK |
| **Erreur** | Rouge | `#dc2626` | Écriture déséquilibrée, erreur, retard |
| **Attention** | Ambre | `#d97706` | Avertissement, démo, en attente |
| **Info** | Bleu clair | `#0ea5e9` | Tooltips, bannières info, onboarding |
| **Fond principal** | Blanc | `#ffffff` | Fond de page |
| **Fond secondaire** | Gris très clair | `#f8fafc` | Lignes alternées, sidebar |
| **Texte principal** | Gris foncé | `#1e293b` | Texte courant |
| **Texte secondaire** | Gris moyen | `#64748b` | Labels, descriptions, désactivé |
| **Bordures** | Gris clair | `#e2e8f0` | Séparateurs, contours de champs |

**Ratios de contraste WCAG AA :**
- Texte principal sur fond blanc : 13.5:1 (AA+)
- Texte secondaire sur fond blanc : 4.6:1 (AA)
- Bouton primaire (blanc sur bleu) : 7.2:1 (AA+)

### Système Typographique

**Police unique : Inter**

Police système, pas de chargement externe. Excellente lisibilité, support `tabular-nums` natif pour l'alignement des montants.

| Élément | Taille | Poids | Particularité |
|---|---|---|---|
| **Titres H1** | 24px | 600 semi-bold | Pages principales |
| **Titres H2** | 20px | 600 semi-bold | Sections |
| **Titres H3** | 16px | 600 semi-bold | Sous-sections |
| **Corps** | 14px | 400 normal | Texte courant |
| **Montants** | 14px | 500 medium | `tabular-nums`, aligné à droite |
| **Labels** | 14px | 500 medium | Champs de formulaire |
| **Petit texte** | 12px | 400 normal | Tooltips, métadonnées |

**Hiérarchie typographique :**
- Une seule famille de police — cohérence maximale
- Différenciation par taille et poids, pas par police
- `font-variant-numeric: tabular-nums` sur toutes les colonnes de montants pour un alignement parfait au centime

### Espacements & Layout

**Système d'espacement adaptatif :**

| Élément | Mode Guidé | Mode Expert |
|---|---|---|
| Unité de base | 4px | 4px |
| Gap entre composants | 16px (`gap-4`) | 8px (`gap-2`) |
| Padding sections | 24px (`p-6`) | 16px (`p-4`) |
| Marge entre sections | 32px (`my-8`) | 16px (`my-4`) |
| Hauteur ligne tableau | 48px | 36px |

**Structure de page :**

- **Header fixe** — logo Kesh, recherche globale, profil utilisateur (langue, mode), bannières contextuelles
- **Sidebar fixe gauche (200-240px)** — navigation par activité, organisée par fréquence d'usage :
  - Quotidien : Accueil, Facturer, Payer, Import
  - Mensuel : Écritures, Réconciliation, Rapports
  - Séparé : Paramètres
- **Zone de contenu fluide** — s'adapte à la largeur disponible
- **Footer discret** — version Kesh, disclaimer légal

**Dimensions :**
- Largeur minimale : 1280px
- Optimisé pour : 1440-1920px
- Pas de responsive mobile — desktop uniquement

### Considérations d'Accessibilité

- **Contraste** — tous les textes respectent WCAG AA (ratio minimum 4.5:1 pour le texte normal, 3:1 pour le grand texte)
- **Navigation clavier** — focus visible sur tous les éléments interactifs (outline bleu `#3b82f6`)
- **Zoom** — interface fonctionnelle à 200% de zoom navigateur
- **Couleur seule** — les états ne sont jamais communiqués uniquement par la couleur. Le vert « succès » est accompagné d'une icône ✓, le rouge « erreur » d'une icône ✗, etc.
- **Taille de cible** — zones cliquables minimum 44x44px (mode Guidé), 32x32px (mode Expert)
